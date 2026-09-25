## Why

Java 8 中通过带副作用的接收者执行赋值 `target(nullReceiver).result = (a && b()) || c()`，为现有混合短路值图提供了一个 `putfield Z` 实例字段消费者。冻结的原始类与 JADX 1.5.6 输出均可编译，且 16 条路径的结果一致，包括接收者为 null 的情形。根代理提供并明确记录 SHA-256 的稳定版 Jarde 快照因 BCI 20 的 Region 所有权重叠而引用了 `one(ZZ)V`；该方法没有发射语句。在改变消费者边界前，必须先查明这次实际观测到的 Region 拒绝原因。

## What Changes

- 规划一项独立且有条件的私有短路消费者证明扩展：仅允许实例字段写入；字段值必须是短路 Phi 的唯一值消费者，并且接收者和值两个操作数都必须与字段计划及 SSA 栈形状匹配。
- 只有独立证明了接收者的所有权、类型和单次求值，才复用现有 `FieldAssign` 与 emitter。保留 Java 中先求值接收者、再求值 RHS，以及 RHS 求值后 `putfield` 才因接收者为 null 而抛错的顺序。
- [Region/SSA 跟踪](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/region-trace.md)已确认第一个失败点是消费者锚点未包含 `putfield`：候选退出后，通用 `if` 路径确实重复认领 BCI 20。只允许完成闭合图与双操作数证明的专用候选先认领一次；不得放宽或绕过 Region 所有权校验。
- 失败时必须原子回退并保留完整字节码来源。范围不扩展到静态字段、实例字段读取、复合更新、数组写入、局部变量存储或调用消费者。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：只有在 Region 所有权、字段身份、操作数绑定和接收者求值均得到证明后，才允许已证明的混合短路布尔 Phi 写入一个实例 `putfield Z`。

## Impact

后续实现仅限于 `jarde-java` 私有 Region/build 消费者证明，以及现有字段计划和 `StmtKind::FieldAssign` 的使用。不计划新增公开 AST、IR 或 pass。冻结的源文件、类文件、Runner、原始/JADX/Jarde 快照输出和当前边界均记录在[证据报告](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/analysis.md)中。Jarde 证据来自局部值变更前的稳定二进制；最新源码的结果仍须独立复测。
