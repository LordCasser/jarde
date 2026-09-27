## Why

[EM-11 三方证据](../../evidence/java-syntax-2026-09-27/em11-overload-binding/report.md)中，Java 8 的 `call((List<String>) new ArrayList<String>())` 选择 `call(List<String>)`。原源码与固定 JADX 的完整类均可重编并验证运行；Jarde 在 `invocation_argument` 处没有 `ArrayList → List` 的安全引用上溯证据，拒绝该调用，留下不可编译的完整类。

## What Changes

- 对已由选定输入或受约束的 Java 8 平台类型事实证明的引用上溯，在调用参数位置写出目标参数类型的 cast，固定物理 Methodref 的源级重载绑定。
- 证明必须覆盖实参呈现类型到目标类型的关系与当前可见的同名候选；未知继承、未经证实的向下转型、泛型参数兼容性与运行时检查不得从 descriptor 猜测。
- 以现有 EM-11 全源码样本验收 `ArrayList → List`、继承同名方法以及 `null`/数组回归；保留一次求值和来源。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：调用实参的已证明引用上溯须保留真实重载目标，缺证据时继续拒绝。

## Impact

限定在现有调用实参组装及其可读取的有界类型层级事实，不改变 JVM IR、物理 Methodref、CLI 协议或加载策略。不在本 change 中放宽 `generic_call_binding_unproved` 方法 `Signature` 投影门；固定 JADX 的 `@NotYetImplemented` 精确文本断言与 Smali 合成访问器也不是本项验收目标。
