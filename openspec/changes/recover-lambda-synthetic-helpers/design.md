## Context

见 `proposal.md` 与 `evidence/java8-lambda/baseline.md`。现有 `lambda@1` 已沿站点的 invokedynamic、BootstrapMethods 和 MethodHandle 链验证工厂、SAM/实现描述符、参数数目及适配；`ExprKind::Lambda` 可呈现 lambda，但 AST 仍以普通表达式表示 body。完整类源码会逐个输出物理成员，所以当前也把 `ACC_SYNTHETIC` 的 `lambda$...` 方法输出。DT-25 实测显示这会产生与 Java 编译器生成符号冲突的完整源码。

可复用的窄模式已存在：`lambda::array_helper_candidates` 从同轮 BSM 找同类 helper 候选，随后要求精确 helper 标识与完整方法体证明；`report.rs` 的 `ClassSourceRecovery::array_constructors` 将单方法事实交给类级适配器，结合全类引用 census 构造原子 array-helper projection。新逻辑应沿用 same-run 身份、事实与投影提交方式，不把 helper 名称当所有权证明。

## Goals / Non-Goals

**Goals:** 对本审计证明的无捕获同类 lambda helper，证明实现句柄所有权、完整且受支持的方法体、SAM 参数对应和整类无其它引用后内联其计算并省略声明；任何未完成证明都拒绝省略。

**Non-Goals:** 捕获值/实例 lambda（DT-26）、静态/实例/构造器方法引用（DT-27）、泛型 SAM/签名推断、bridge、其它编译器命名约定、控制流/异常/复杂副作用 helper、任意私有 synthetic 方法隐藏、改变物理 class facts 或令方法级 recovery 假装拥有全类视图。

## Decisions

1. **以站点事实绑定 helper，不以名称识别。** 只接受当前 `PhysicalDefinitionId` 的完整类读取中，lambda 计划里的精确 implementation owner/name/descriptor 和 bootstrap/CP/use-site 相互一致的私有静态 synthetic 声明。`lambda$` 前缀仅是候选过滤条件；其他 bootstrap、owner、handle kind、参数/返回描述符不一致即拒绝。保留站点计划已有的 descriptor adaptation；不借助泛型签名或名字推导参数。
2. **只在完整 helper 体可直接表示时内联。** 首片沿用已有方法 AST、effect、来源和 emitter，只允许完整单一路径、无额外可观察指令的原始值直线表达式；参数必须按 SAM 顺序一一对应到 helper 入参。zero-arg 常量、单/双 `int` 入参的简单运算是冻结正例。分支、异常表、调用、捕获/receiver、额外参数、未消费来源或任何 fallback 均不是内联证明。不能在 lambda 体中复制外部执行时点的副作用。
3. **省略权由整类 census 持有。** 先遍历所选物理类所有方法体及相关 CP/handle 使用，要求每个候选 helper 的所有引用都落在精确匹配且成功投影的 LambdaMetafactory implementation handle 上；普通 invocation、method handle、缺失/歧义引用或未读完整都让候选 helper 保留。类源码适配器在同一提交边界收集所有候选，只一次提交完整的内联与声明隐藏集合。单方法 API 只输出其方法内容，不能据一个站点隐藏类级成员。
4. **沿用物理来源和预算停止。** 每个站点保留 invokedynamic/use-site 与 bootstrap implementation CP 证据；lambda body 的表达式继承 helper 指令 origin，声明省略保留 helper 的 physical method 身份作为 derivation/evidence。全类成员/方法体/指令读取及 AST 投影分别按既有预算计数和轮询。停止发生在提交前时不输出半成品；已完成的原始读取前缀和停止原因按既有 report planes 报告。
5. **JADX 仅作形态参考，Jarde 的 proof 决定接受范围。** 固定版本 `CustomLambdaCall` 对解析到的同类 synthetic method 设置 `DONT_GENERATE` 并启用 inline，`InsnGen.makeInlinedLambdaMethod` 将方法指令写入 lambda；其 `TestLambdaStatic`/`TestLambdaArgs` 给出 0/1/2 参数形态。Jarde 不复制该标志即隐藏的策略，而增加 helper body 与全类引用证明及预算边界。

## Risks / Trade-offs

- [候选相同但另一普通调用仍可达] → 仅精确 BSM 反向引用不足以证明可隐藏；需要全类引用 census，未知或停止时保留成员。
- [表达式在 helper 执行时与 lambda 创建时错位] → 只把 helper 表达式写在 lambda body 内，并仅准入 SAM 调用时才求值的、无外部副作用简单表达式；捕获和副作用形态另立范围。
- [helper 部分恢复仍能产生看似完整源码] → 要求完整 Code、无 fallback 的可投影 AST 和 class-wide helper 集合原子提交；缺任一条件就不隐藏。
- [全类 census 增加读取和预算消耗] → 复用类源码已具备的成员读取与项目边界，按现有 budget 计量；不在单方法路径隐式读取更多方法。
