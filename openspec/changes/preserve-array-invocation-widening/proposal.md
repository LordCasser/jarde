## Why

调用参数已经保留了完整的数组 descriptor，但 `invocation_argument` 对引用仅允许同型或目标 `Object`。普通 javac 将 `int[][]` / `String[]` 传给 `Object[]`，或将 `String[][]` 传给 `Object[][]` 时，JVM 方法目标明确且不需运行时检查；jarde 当前逐个拒绝，整类无法编译。即使放行，仍须固定原 Methodref 的重载目标，避免生成的 Java 选择更具体的方法。

## What Changes

- 在现有调用参数消费入口，证明一个封闭的数组上溯子集：相同组件、数组组件的递归上溯，以及任何数组到 `java.lang.Object`、`java.lang.Cloneable`、`java.io.Serializable` 的内置关系；原始类型组件不能冒充引用组件。
- 对已经证明的非同型数组参数用现有 Cast 固定 Methodref 的目标参数静态类型，保持重载目标、一次求值和来源。
- 未证明的外部继承/接口关系仍拒绝；不扩大赋值、返回或任意引用转换，不引入 classpath resolver。
- 在已验收的共享延迟值顺序实现之上串行实施，并以完整类重编和运行验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：对已证明的数组引用上溯在调用参数处保留真实方法目标。

## Impact

限定在 jarde-java 既有 `invocation_argument`、小型数组形状判据及测试。`../../evidence/java-syntax-2026-09-22/array-reference-conversion/` 的 28 项核心输入原 class/JADX 完整执行相同，当前 jarde 三处引用且整类 javac 失败；独立目标选择样本显示无 `checkcast` 的局部上溯和更具体重载并存。追加的 source-only marker 接口样本中，`int[][]→Cloneable[]`、`String[][]→Serializable[]`、`int[]→Cloneable` 的原 class/JADX 三行一致，旧 Jarde 三处引用。此前 `preserve-invocation-argument-types` 已交付同型/Object/数值边界，本案只扩展有证明的数组关系。
