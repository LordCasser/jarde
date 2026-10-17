# Design

## Context

动机与当前产品见[proposal.md](proposal.md)。本父项跨多个已交付叶片，仍需统一实际取证与验收口径；当前更新只补齐父级设计和证据，不新增生产识别规则。详细方法、BCI、原始收据与限制见[instrumentation.md](instrumentation.md)。

## Goals / Non-Goals

**Goals:** 沿现有`analyze_method_ir`、Facts/SSA、guard、Region、Builder和来源路径证明每个有界形。return表达式先求值一次并保存，再执行cleanup；正常/异常路径清理次数、异常对象身份和cleanup抛错优先级各有证据。完整类保持原有方法集合，原样重编后只以重建classpath运行。

**Non-Goals:** 不从catch-all或相似字节序列推断任意finally，不恢复未证明的finally自身return/throw，不用死handler的不可观察性替代可抛调用的异常证明；不重定义原始exception edges、公共AST/Region或原P5预算。JADX是外部对照，不是可信语义oracle。

## Decisions

### 复用既有表示，每片只增加必要证明

已维护的Rust代码已有Finally、SavedReturn、私有continuation、真实SSA/effects和受限工作账本，足以承载本组。无需新依赖、库或许可证变更。通用第三方反编译器不能提供本引擎所需的物理身份、原子ownership和停止账本，不能替代私有证书；JADX1.5.6仅作固定版本实验工具。loopless复用受限尾部，noCall证明iinc/phi，CallReturn保存调用结果，transfer片只投影已证明不可能同步抛错的handler。替代方案“放宽旧guard/按诊断同名合并”缺少各自正常与异常出口证据，拒绝采用。

### 区分输入解析、JVM验证与源码恢复

真实javac8与javac23 `--release 8`各自产生Java8 class；reader读取/IR分析成功只说明能分析。生成报告为Java/Structured和完整物理BCI是源码质量/来源证据；完整文本被真实javac接受是编译证据；Java8/23 `-Xverify:all`运行及逐字oracle是限定输入的行为证据。WAR另经真实加载根`WEB-INF/classes`访问，不用JAR路径拼接代替枚举。它们不能彼此替代，也不能由某一fixture推断整个JAR/WAR。

### 控制证书与提交的边界

有界扫描使用既有账本，预算/取消/期限必须按既有原因和usage返回。缺证保留原fallback、生产者、来源与拒绝。Builder成功后才允许原子投影；真实For节点、ownership和最后一次付费poll全部通过才修改正文与Region。正常边不改，handler来源不冒充循环gateway。完整证据与essential模式正文须相同。

### 当前对照基线和验收范围

原patrol是历史，不改写其原始输出。父级当前矩阵写入本change；当前05e CLI对6整类32方法的24命令重筛、固定JADX的21命令、叶片JAR/WAR与独立driver均引用真实归档。纯Python独立解析32个Code/724个物理指令实例，核来源无缺失/多余且方法身份正确。前一b4a6 CLI至当前05e的28个非transfer方法记录文本不变；transfer4记录允许预期去fallback。EX.finReturn前后文本相同且仍fallback，FinallyCompletion两完整4114B输出各3个missing return，均是红项，不计新行为成功。

## Risks / Trade-offs

- [固定main和有界driver只观察部分输入] → 明确记录观察集合，语义要求另由SSA、effects与负控制支撑；不宣称动态覆盖不可观察handler或所有异常。
- [历史将fallback内if/else叫作已恢复] → 同时记录报告质量、完整类javac结果与是否真正运行rebuilt产物。
- [叶片不同源码/CLI或父CI混记] → 使用产品30afa的实际4945源码Git blob/mode、冻结CLI、独立本地/CI收据；后续文档head只通过相同源码绑定，不称再次执行了产品CI。
- [未来改动破坏对照] → 以当前产品固定的原始class、文本、oracle与真实双javac重编为基线；未证明形保留具体拒绝，不以删方法或修生成Java通过门禁。

## Migration Plan

父级先提交设计、instrumentation与新红项便携证据；已交付功能仍由各叶片归档定义。父剩余验收逐项独审后才勾选、同步唯一delta、正常归档。后续主动抛异常/覆盖完成需独立有界取证与变更，不借本父项扩权；发现不满足证明时保持既有拒绝并保留失败raw。
