## Why

[冻结的 Java 8 三方执行对照](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-array/analysis.md)表明，混合短路布尔值作为 `boolean[]` 的元素写入值时，JADX 的完整类在正常/null/越界 24 路径与原 JVM 一致，当前 Jarde 则整体引用 `one`，虽然可编译却 24/24 行行为不符。当前可见首个阻断是 Region 所有权重叠；数组消费者也尚未纳入短路值证明。必须先分开定位这两个边界。

## What Changes

- 先证明当前 `jre_region_ownership_overlap` 的准确成因，保留重复物理 owner 的原子拒绝规则；只有归属可以在现有 Region 结构中唯一闭合时才尝试数组消费者。
- 在既有有界短路值图上，限定真实 `bastore` 对已证明 `[Z` 数组的单次写入：Phi 是第三个值操作数的唯一直接 use，数组和下标两个操作数按 SSA 与实际求值顺序证明可呈现、仅求值一次。
- 复用现有 `IndexAssign`、Boolean 值适配、source map 和预算合同；完整类须通过 Java 8 重编，并在正常/null/越界 24 路径保持值、调用次数与异常时点。身份、类型、边、求值或 owner 不明时整体引用。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对独立闭合且类型、操作数、异常顺序可证明的混合短路 `boolean[]` 写入，恢复一次数组元素赋值；其它数组或不闭合图保守拒绝。

## Impact

预期落点限于 `jarde-java` 的既有 Region/SSA 短路值证明与数组写入发射接缝；不新增公开 AST、IR、全局优化或通用类型推断。`bastore` 也写 `[B`，故必须由数组值证明 `[Z`，不能从 opcode 猜测。实例字段与局部变量消费者各由独立 change 处理。
