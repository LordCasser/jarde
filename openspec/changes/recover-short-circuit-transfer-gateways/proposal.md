## Why

[冻结的三元嵌入 OR 类](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)把 `result = (gate ? extra : left) || other || rhs()` 编译成五个测试、两个 1/0 producer、一个 `putstatic Z`，其中 BCI 8 是从 `extra=true` 通向共享真 producer 的 `goto`。Jarde 目前安全地整体引用，但整类运行 32/32 行与原 class 不同。JADX 1.5.6 虽输出可编译条件，仍有 16/32 行结果或 RHS 调用次数错误。当前私有短路值图只接受测试或常量 producer，无法把这条无效果转接边纳入闭合证明。

## What Changes

- 在现有有界 `ShortCircuitValue` 图中识别可证明的单入口、单出口、无独立效果的前向 transfer-only gateway，按真实物理边归并逻辑后继，并保留 gateway 指令的 owner 和来源。
- 继续要求精确正常前驱、异常边、SSA 测试表达式、1/0 producer、唯一同槽 Phi 与唯一 `putstatic Z` 消费证明；值的真伪由解码 taken/fallthrough 和 producer 常量决定。
- 对冻结完整类做 32 路径原/JADX/Jarde 重编与 JVM 对照，以原 class 为语义准则；证明不足时仍整体引用。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 闭合短路值图内的纯转接节点可参与静态布尔字段写入的结构化恢复，不得改变调用次数、求值顺序或来源归属。

## Impact

局限于 `jarde-java::region` 的私有图认领与 `jarde-java::build` 的图/SSA证明及表达式发射；复用既有 `Conditional`、字段身份与预算接口，不新增公开 AST、IR 或全局 pass。字段图、直接返回与调用实参消费者均作回归，字段报告 `presented` 债务单列。
