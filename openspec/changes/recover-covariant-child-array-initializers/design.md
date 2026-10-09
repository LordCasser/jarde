## Context

动机见proposal。前片候选CLI `/private/tmp/jarde-constructor-primitive-conversions-cli-v1` SHA `3e4b615e0131d041ecc47c72c0131a3bbd0f5fc9042d8f91b2aa21f26baf6a95`，产品代码a738a894及属性文档修正8f624ea64已推送；后者CI37968418985已实际四job/48steps全部success，root产品blob/冻结CLI身份核验通过，前置门禁解除；当前仍无本片产品改动。

前片完整legacy manifest SHA `dc8a09f2b9d34bd50f2e8b43f5fbcfda23e2f7bfdca58c9970cf81be962528eb`，direct六类双真实javac均0/2，剩余numberGridDirect/collectionGridDirect/ownGridDirect正文拒绝。准确输入、raw输出与JDK身份沿用前片全部记录，不能改写。详细架构和验收依据见前片results/next-child-array-covariance-plan-v1.md与jadx-child-array-reference-audit-v1.md。

## Goals / Non-Goals

**Goals:** 结构层闭合child/parent唯一消费与ownership，呈现层使用实际类型和准确store的赋值证明；原完整六类家族双JDK成功，所有负边界保持。

**Non-Goals:** 不把verifier栈引用类别当Java赋值证据，不扩大泛型声明、构造白名单、别名或effect规则，不增加迭代rewrite、hierarchy walker或类型服务。parse/dialect/runtime selection、结构证明、Java正文与完整编译/执行分别记证据。

## Decisions

### 1. 去掉类型相等早截断，保留已有结构前提

build.rs:13215-13251已按stored ValueId选中consumer等于准确parent store的child。`component != child_type`在此把Java兼容性当作ownership必要条件。最小修改是解除这一相等条件，保留child真实NewArray producer形状、child sources所在区间、深度与公共meter检查；如果child_type只用于这个条件，复用原NewArray operation guard即可，不添加字段传递已能从原事实取得的类型。

保留array_initializer_reader的同block、唯一reader、顺序与interval_is_expression，以及完整parent/Sites共同提交、arrays→sites单次move。不得为协变提前提交child或放宽extra reader。所有这些前提都与类继承无关，因此结构候选可以成立而Java正文仍拒绝。

### 2. Builder使用既有准确store证明

array_initializer_element已经收到与element ValueId配对的store_bci。render_value仍按child实际数组类型生成Expr，initializer_reference_widens处理精确类型/Object/null、闭合数组形状、平台scalar/interface同秩reference lifting以及同BCI且source/target完全匹配的snapshot proof。沿用这些路径，不把父组件名写到child上制造兼容。

facade从真实aastore的SSA operands生成所选snapshot class-header chain证明；ownGrid外层store24/45分别需要DerivedA[]→Base[]（DerivedA→Mid→Base）和DerivedB[]→Base[]。method-only不带family证明，缺失Mid或错误pair仍不能生成完整Java。现有纯helper类型单测不重复建立，补实际initializer/完整报告边界。

### 3. 参考JADX的组合表示，保留本引擎的效果约束

JADX ReplaceNewArray/FilledNewArrayNode及TestMultiDimArrayFill证明每层数组节点可作为上一层元素。静态审计未找到用户类reference-array covariance的对应集成用例；其primitive半覆盖、按索引TreeMap排序、多个reader时移动位置以及argument时序TODO不构成本片效果证明。借用分层组合思路，不迁移这些放宽条件。

已有Jarde Runtime/platform事实及snapshot输入足够。外部类型库无法替代本次SSA/物理store的精确证明；新增依赖与维护/许可负担没有收益。不复制JADX代码，不新增pass、AST或自定义类型表。

### 4. 完整家族是验收分母，collection拒绝先验证根因

两腿均包含Base、DerivedA、DerivedB、LocalInterface、Main、Mid，共12个完整class-source输出。以全部生成源在显式空classpath/sourcepath下重编，runtime只用新classes并-Xverify:all；逐字核对原exit/stdout/stderr。目标为完整direct2/2，不能删grid、借原helper或只编Main。继续复跑旧factory2/2、18矩阵16/18、完整数值家族2/2，BigDecimal两腿保持单独失败分母。

collectionGrid现有ordinary_generic_source_unproved与body fallback同时出现，class_source的既有完整正文proof门槛可解释这一相关性，目前未证明独立Signature问题。先只接通child组合，再检查完整输出；若正文已完整恢复而Signature仍阻塞，root凭新证据分析并单独登记，不默默放宽泛型gate。此时本片完整家族验收仍未完成。

## Risks / Trade-offs

- [结构成功冒称Java成功] → 对照body quality/representation/diagnostics、完整生成源编译、运行和双流；NewRecord不替代正文。
- [错误store/type证明被借用] → 实际initializer的错BCI/source/target与missing-Mid控制，所有不明项使整方法fallback。
- [求值或ArrayStoreException被改动] → extra reader、非顺序store、interleaved effect、primitive invariance与verifier-valid不兼容store控制；不执行故意变义mutant。
- [child或Site部分提交、预算失效] → 公共depth/meter及pending共同提交/Stop回归，不添加feature预扫或重复验证。
- [collection存在额外泛型阻塞] → 完整新报告确认后独立记录，保持原泛型gate，本片不提前验收。
- [磁盘增长] → root串行Cargo、jobs1/incremental0、20GiB停建线；验证完清本项目target，保留CLI与全部原始证据。

对抗审查输入见results/adversarial-plan-audit-v1.md。负控制必须锚定真实child retained ValueId、其唯一parent store及相应组合区间，先确认结构路径，不能由无关不支持opcode的早拒冒充。本片primitive/rank负控至少一条实际child候选抵达Builder再因赋值不明拒绝；wrong-store/pair沿用已有helper边界，missing-Mid验证实际完整family。

既有snapshot_hierarchy_widenings_presented在facade.rs:25314-25324捕获BudgetExceeded/Cancelled并返回空facts；随后相同Budget进入正文恢复，但不能只凭最终类型fallback推断原始Stop原因/位置已传播。该现有策略另记results/snapshot-proof-stop-debt-v1.md，本片不顺带重做facade。结构公共meter及正文构建Stop以各自真实停止控制验收，空类型facts只证明缺证时不生成成功Java。

## Migration Plan

当前前片CLI/源码和输入已冻结，可先完成只读基线复核；前片确切SHA CI全部成功后才启动Luna限定实现。root负责所有Cargo/rustfmt/Git、双真实JDK重放与对抗验收；产品与fixture在门禁运行期间冻结。新结果使用不可覆盖版本目录，旧失败/streams/hash保留。无外部API或数据迁移，相关架构债务另片处理。
