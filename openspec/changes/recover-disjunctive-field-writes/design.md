## Context

见 [proposal](proposal.md)与[三方实测](../../evidence/java-syntax-2026-09-25/short-circuit-shared-true/analysis.md)。现有 `Region::ShortCircuitValue` 已能在普通嵌套 `If` 前一次认领两分支、两生产者及一个静态字段消费块。`recover-conditional-field-writes` 的 builder 只证明共享 false：外层跳转和内层跳转均到 0 生产者，外层顺序边到内层；因此 `javac` 的 `a || rhs()` 虽被安全引用，`assign` 仍为 fallback。

冻结 OR 方法的实际边为外层 `ifne 10`，顺序边到 RHS；内层 `ifeq 14`，顺序边到 true 生产者 10，跳转边到 false 生产者 14；10/14 再汇入唯一 `putstatic 15`。JADX `IfRegionMaker.mergeIfInfo` 在可合并条件路径上区分 AND/OR，`TernaryMod` 校验两臂共用 Phi，这是算法线索。Jarde 不采用仅靠后继路径相等的宽松合并：已有同方法双分配点和非规范 `Z` 反例表明“能编译”不足以证明值和类身份等价。

## Goals / Non-Goals

**Goals:** 在既有短路 Region 的受限静态 `Z` 字段写入中，按 CFG 实际边极性证明共享 true 形状；保持右侧按需求值、字段唯一写入、JVM 最低位和物理来源；失败时完整引用。

**Non-Goals:** 不恢复源码字面 `||` 的拼写，不新增逻辑运算 AST，不推广到任意布尔表达式、异常处理区、循环、实例字段或多消费 Phi。不修 `recover-conditional-field-writes` 尚未关闭的独立异常边拒绝矩阵。

## Decisions

1. **在同一证明内参数化共享生产者。** 从 canonical 正常边和分支操作数确认外层顺序边到内层，外层跳转边恰好到 true 或 false 生产者；内层顺序边到 true、跳转边到 false；两个生产者各自唯一到消费块。共享 false 保持原检查；共享 true 要求 true 生产者的前驱恰为外层与内层、false 生产者前驱恰为内层。其它外部入口、异常边、回边或不明方向一律拒绝。证明记录“外层跳转产生 true”这个事实，而不是从 `ifne`、地址次序或源变量名推断 OR。沿用当前 `O(E + I + Phi)` 预收费和深度界限。另一方案是新建 `OrRegion`；它会重复块所有权、Phi 和字段证明，故不采用。
2. **Phi 和唯一消费仍按物理前驱绑定。** 保持 1/0 生产者、栈深、Phi 输入与两生产者出口一一对应、唯一 `putstatic Z` 读者、字段身份及无额外可观察指令的现有检查。共享 true 的 true 生产者有两个入边，但只产生一个 SSA 值，不能因入边数为二就复制调用或写入。异常边或第二消费者须先拒绝，不让部分折叠计划进入 builder。只调整合法的 CFG 前驱集合，不放松值或字段证据。
3. **复用 `Conditional` 保持求值位置。** `test_expr(branch_bci, false)` 表示该分支的顺序边条件。共享 false 的表达式保留为 `outerFallthrough ? (innerFallthrough ? 1 : 0) : 0`；共享 true 则为 `outerFallthrough ? (innerFallthrough ? 1 : 0) : 1`。于是左侧直接为真时不会执行 RHS。现有字段消费处的 `% 2 != 0` 将精确整数结果转为 Java `boolean`；不把任意整数 Phi 偷换成 `true`/`false`。`Conditional` AST 已足够表达行为，先不增 `LogicalOr`。条件、两个生产者、写入和消费块后缀的来源继续原子绑定；构造任一步失败，整个 Region 回退完整引用。
4. **用完整类执行而非语句形状验收。** 由冻结源再编译字节一致性，原/JADX/Jarde 类各自 Java 8 重编，临时 runner 在 JVM 验证下逐行比较字段和 `calls`。误极性补丁、第二写入/消费者、非 1/0、字段描述符和预算停止分别约束保守路径；共享 false 原正例必须维持行为。将 JADX 的 OR 合并顺序当作参考，不把其 `getUseIn()` 等启发式或本单例通过当作普遍证明。

## Risks / Trade-offs

- [外层分支极性看反后提前执行 RHS] → 使用 decode target 与 canonical 正常边双重绑定，并以左真 `calls=0`、左假 `calls=1` 做运行断言。
- [两个入边的 true 生产者被复制] → Phi 按生产者出口值配对，发射只有一次字段赋值，来源覆盖两条入边而不重复执行值。
- [放宽共享 true 连带误接纳非 1/0 或第二消费者] → 只改变准许的 CFG 前驱集合，其余 SSA、字段及失败原子性检查保持；聚焦负例全部重新恢复而非仅给旧 Region 跑 proof。
- [源码可编译但仍丢效果] → 运行时比较原 class 的字段值和计数；fallback 文本即使能编译，也不算行为通过。
