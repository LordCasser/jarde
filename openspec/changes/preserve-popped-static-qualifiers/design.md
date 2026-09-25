## Context

见 `proposal.md` 与 `../../evidence/java-syntax-2026-09-22/static-field-qualifier/summary.json`。现有 `discarded_evaluations` 对 `produce; pop; invokestatic` 将 `pop` 归后续调用的限定符，`call_expr` 因而递归渲染其生产值；但 `produced_value_reaches_a_reader` 只把 `renders_the_value_it_reads` 接受的指令当读者，普通 `pop` 是 `Other`，于是生产者调用先被独立发射。`quoted_bcis` 仅附限定符 `pop`，没有跟踪若已延期却最终拒绝的限定生产者。现有普通 `invoke; pop` 正确表示丢弃调用结果，不能粗暴把全部 `pop` 当读者。

## Goals / Non-Goals

**Goals:** 对已认领的 `pop` 建立一次拥有者关系：可独立发射的紧邻调用用现有弃值调用语句，其它可证明的限定值嵌入目标静态调用；拒绝时物理来源及效果完整。边界限定为同块相邻形状及当前可证明 Java 类型/成员。

**Non-Goals:** 推断源码选择了字段限定访问还是方法限定访问；不引入一般 `pop` 节点、跨块解释、类层级解析、静态字段语法糖或其它赋值恢复。`receiver().staticField = rhs()` 与 `staticField = receiver().rhs()` 若生成同一已证明指令序列，源写法无从区分，验收依据实际语义和来源。

## Decisions

1. **沿用现有 `DiscardedEvaluations` 计划，优先复用弃值调用。** 若 `pop` 唯一消费紧邻 `Invoke` 的返回值且调用可作为 Java 语句呈现，沿用计划已有的 `discards` 路径：`receiver(); Owner.rhs()` 按原字节码先后执行，生产者不再嵌入目标调用。审计中 `receiver().rhs()`、`receiver(); return rhs()` 以及三种静态字段写法各自生成完全相同的 Code；无法也无需恢复源码选择。仅对不能独立成为语句的限定值，保留计划的 `qualifiers` 路径，由目标静态调用承载并让其确切 SSA 生产者延期。直接跳过所有 `pop` 前调用会丢效果；新建 ownership pass 与已有计划重复。
2. **只发布可表示的表达式限定调用。** 对仍走 `qualifiers` 的局部/字段等值，限定表达式需有已证明引用类型，且其 Java 静态成员选择须能匹配原 Methodref；当前无层级证明时至少检查确切目标属主和可拼写名称。接口静态方法不能用实例表达式限定，必须采用独立效果路径或拒绝。异属主同名隐藏会静默改绑，异属主异名可能直接不编译；两种已用 JVM 验证补丁实测。强转丢弃和非引用值保持保守边界；现有桥擦除例外仅沿已证明规则。
3. **补全失败消费者的来源追溯。** `quoted_bcis` 在有被认领限定符时，除了静态调用与 `pop`，还对 `pop` 丢弃的确切值使用已有延期生产者追溯；这样承载调用后来因参数、类型或预算失败，不会留下一段没有生产者的解释。迭代深度、预算、取消和去重沿用同一追溯函数，不建全图扫描。
4. **仅在测试中执行自写 Java。** 使用 `javac --release 8 -g:none`、JVM 验证及完整生成类重新编译/运行；产品不执行目标代码或获取目标依赖。无需外部库，当前 SSA 与池属主事实已经提供限定范围所需关系，核心/CLI 分层和 parse/方言/verification/源码编译报告保持原语义。

依据：[JLS 8 §15.11.1](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.11.1) 与 [§15.12.4.1](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.12.4.1) 均要求表达式限定的静态成员先求值限定表达式再丢弃结果。限定表达式**返回 null 本身不会抛 NPE**；只有它的求值过程或后续别的操作才可能抛异常。现有 `DiscardedEvaluations` 注释里对“qualifier 为 null 的 NPE”的措辞在实现时应顺手校正，同一局部语义范围内完成。

## Risks / Trade-offs

- 将普通丢弃调用误当限定符 → 只有计划已绑定具体 `pop`/下一静态调用且类型/目标合法时延期，普通 `invoke; pop` 和异属主对照必须原样。
- 成功路径只去掉独立调用，却在拒绝路径丢掉它 → 用非法/无法呈现目标的合法 JVM 边界检查 BCI 生产者、`pop`、调用、最终消费者均可追溯。
- 只比较字段值掩盖重复调用 → 原 class/JADX/Jarde 完整类用计数及生产者抛错对照；单独检查正文和 source map。
- 与位运算并行编辑 `build.rs` → fixture 与规划先行，生产按 root 移交窗口串行实施，合并后复跑相邻测试。
