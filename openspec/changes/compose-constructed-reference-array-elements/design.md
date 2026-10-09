## Context

动机见 proposal.md。类型事实与 Runtime-selected header walk 由相邻 EM-18 类型片负责；本项是构造与数组的现有证明组合。源审计及完整基线位于 ../recover-heterogeneous-array-init/results/em18-constructor-composition-audit.md 和 ../recover-heterogeneous-array-init/evidence/baseline-20261009.tar.gz。

`report.rs` 当前先建立 ArrayInitializers，再运行 init::sites；此顺序用于 constructor arguments 内的数组和 varargs，不能无差别前移 sites。`init::verify` 验证 new/dup/init 与唯一外部 consumer，但其 renders_its_reads 不接纳 aastore；数组的 collect_expression_bcis 又不接纳 Allocate。渲染层已支持 Sites::site_of → new_expr 以及 NewArray initializer，无需新增 AST。

## Goals / Non-Goals

**Goals:** 在一份已闭合 fresh-array candidate 内，准确组合每个构造 element 的同一 SSA instance、constructor interval 和 paired store；只在双方完整成功后共同发布归属。保留构造参数与元素效果次序、预算、handler覆盖和失败BCI。

**Non-Goals:** 不解析一般 alias，不扩大通用 Allocate 白名单，不以存在构造器记录推断数组合法，不取消既有 inline-array constructor arguments，不混入周边 concat/field 问题。

## Decisions

### 1. 在现有 candidate 内复用构造器验证，原子提交

数组 candidate 遇到可能的构造元素时，使用现有 init verifier 检查准确 allocation、dup、匹配init、参数闭包与唯一实例consumer；临时许可的consumer必须正是此candidate配对的aastore，且store的value为该site完整instance。把验证后的完整构造interval当一个原子element依赖，不把Allocate当普通无副作用producer。

candidate结构整体成功后才由既有Sites/ArrayInitializers保留此站点与ownership。若后续元素或结构拒绝，不得发布只满足一半的结构证明；既有refusal必须保留被消耗生产者的BCI。复用现有记录及局部candidate数据，不引入全局构造registry或额外扫描pass。

受限 verifier 的许可必须绑定 `(store BCI, stored ValueId)`，由SSA use-def/copy事实核对它是该构造的准确实例；`Site.instance` 是BCI集合，单凭BCI在集合中不等于存值身份已证明。现有 `Site.expression` 保留完整构造/参数区间，`Site.owned` 保留构造scaffolding。数组candidate局部暂存这些现有Site，直到父子数组链闭合和排重成功再交给普通 `init::sites`；后者接收已经提交的站点，并在既有allocation census中跳过其重复验证，仍保留一个verified allocation记录和一个materialized new记录。不得为了传递事实复制完整ArrayInitializers、提前全方法构造扫描或增加第二套registry。

构造参数中的child array必须从本轮已闭合的candidate/既有只读数组事实取得，不能先把未闭合parent发布给普通扫描来解依赖环。实现前明确这个受限读取接口与Site单次移交点，不能以全计划clone或半提交规避借用关系。

赋值兼容仍在Builder呈现阶段消费相邻类型片的准确store证明；结构计划不是Java成功证明。类型或后续呈现失败时可以保留真实结构证据，但必须拒绝完整initializer/body，保留生产者来源，不能输出半份Java初始化器或可独立执行的悬空构造表达式。这里不要求把既有结构证据平面改成呈现成功平面。

备选“只开放aastore reader”不能证明数组链；备选“泛化Allocate expression”绕过init；备选“全量提前构造扫描”破坏constructor argument数组依赖并引入重复扫描。均不采用。

### 2. 保持同一准确实例、次序和效果边界

构造器的outside_readers必须精确等于paired store；额外alias、pop、第二次消费、块/handler跨越或不属于参数依赖的效果均拒绝组合。数组闭包仍验证length、ascending physical index、dup/index/store单次边、唯一finalconsumer、完整handlercoverage。两侧原有深度/analysis预算必须收费且poll，不能循环扩大候选。

array owns自身scaffolding，site owns构造run；`element_sources`和derived provenance可以引用构造/参数BCI，不要求来源集合互斥；实际statement ownership必须唯一，且每个效果只呈现一次。element仍经相邻类型片的兼容判据，不插入elementcast，不改变runtime数组分量。

root核对到普通 `init::sites`/`verify` 当前不接Budget，计费主要在record materialize；不能假称已有逐读取预算。此组合新增的candidate verifier调用必须接ArrayInitializers的共享预算，在新增scan/use/递归读取中poll/charge并返回准确StopReason，而不是只给Site.expression长度估算费用。普通construction全路径计费是既存债务，不默认扩入本项；若复用必需的内部预算接缝影响它，必须明确该必需影响及实际P5费用，不增加全局计费服务。

### 3. 最小完整验收

使用已有冻结direct-new输入与新的简洁完整family：CharSequence、Collection、Throwable、同snapshotBase/interface。构造参数和两个元素带可区分mark效果，original/JADX/Jarde完整source隔离重编，-Xverify:all双流和effect次数/次序一致，source-map覆盖每个new/dup/init/store且无重复站点。

负例覆盖extra consumer、非fresh/乱序/重复索引、handler/block边界、参数间无关效果、错误storedinstance与后半element拒绝。区分第二元素结构失败后的零提交与第二元素类型拒绝后的完整正文拒绝，不伪造丢弃真实结构记录。保留constructor参数内varargs/char[]/nestedarrays和现有new/arrays测试，不只测试新shape。

完整direct fixture的boxedDirect还含五个primitive argument conversions，当前在构造区间以interleaved-effect拒绝；它们不是引用分量兼容或paired-store身份的证据。保留完整家族与拒绝，不移除成员；纯组合完整正例另用无primitive转换的引用构造参数。若要处理primitive转换，独立取证/规划，不能悄悄扩展当前构造表达式白名单。

## Risks / Trade-offs

[两份证明的依赖环或重复计费] → candidate局部验证与共同提交，保留既有编排入口；验证前后usage与P5账本。

[失败后丢生产者或出现半初始化器] → 对第二元素故意失败的控制审查所有raw来源、marker及owned BCIs。

[完整main因周边语法仍失败] → 原失败照实记录；使用从创建起就完整的独立family验收该组合，不剥离原类成员，也不宣称EM-18整个单元追平。

## Migration Plan

先冻结当前类型片及existing constructor-array baseline；实现局部proof seam，root对抗验收后串行全门禁和精确SHA CI。无格式迁移或外部依赖；若正确性门禁失败，保持原拒绝行为并修复此组合，禁止扩大白名单遮掩。
