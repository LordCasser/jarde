## Why

已验证的 Java 8 函数式工厂若立即作为调用接收者，当前 Jarde 会输出 `((int p0) -> ...).apply(n)` 或 `Math::abs.applyAsInt(n)`。这两种文本没有函数式目标类型，`javac` 拒绝；完整类虽然零引用，仍不可编译。冻结的数组构造器引用与普通方法引用反例中，原 class 和 JADX 均能编译执行且逐行相同。

## What Changes

- 对有工厂 descriptor 证明的即时 lambda/方法引用调用接收者，在现有调用表达式里显式保留其函数式目标类型，使完整源码能被 Java 8 编译。
- 保持调用的真实 receiver、参数、求值顺序、异常和来源；缺少目标类型或无法证明它与调用 owner 相容时明确拒绝，不把零引用当作可编译证明。
- 用直接调用与先存入局部再调用的三组完整类，比较原 class、JADX、Jarde 的编译和 `-Xverify:all` 执行；手写加 cast 的机制实验只用于论证方案。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：函数式工厂的结果立即调用接口方法时，恢复的 Java 表达式具有可编译的目标类型并保留原调用语义。

## Impact

主要影响 `jarde-java` 的现有 `Call` 接收者构造和 `Cast`/emitter 分组；复用 LambdaMetafactory 工厂 descriptor、表达式类型与来源，不新增 AST 类别、pass、泛型系统、外部库或 JVM reader 事实。`preserve-lambda-descriptor-adaptation` 处理函数体内的动态参数适配，本项只处理工厂结果在外层作为即时调用接收者的 Java 目标类型；合成 helper 命名冲突另案处理。证据见 `../../evidence/java-syntax-2026-09-22/immediate-functional-receivers/`。
