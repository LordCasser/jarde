## Why

DT-25 的固定 Java 8 无捕获样例中，Jarde 已能从 `LambdaMetafactory` 呈现 lambda 语法和 0/1/2 参数，却仍把 `lambda$...` 编译器 helper 当普通成员输出，并在 lambda 中调用它。JDK 8 源码没有该调用目标；重新编译会与 javac 为 lambda 生成的同签名 synthetic helper 冲突。冻结三方证据确认原源码和 JADX 均可编译执行，而 Jarde 输出不能重编。

## What Changes

- 对准确绑定到当前类私有静态 synthetic lambda helper 的 LambdaMetafactory 站点，在完整方法体可被已有源码投影证明时，将 helper 方法体内联进 lambda。
- 仅在完整类范围证明 helper 的所有引用都属于这些已投影站点、且每个相关 helper 都可安全内联时，原子省略 helper 声明；证据中断或有任一引用/方法体无法证明时，不省略该 helper 集合。
- 第一片只覆盖同类、无捕获、编译器生成的简单直线 helper 与原始类型 0/1/2 参数 SAM。捕获 lambda、方法引用、泛型适配和更复杂方法体留待独立审计。

## Capabilities

### New Capabilities
无。

### Modified Capabilities
- `java8-recovery`：增加编译器生成 lambda helper 的有证源码投影与整类安全省略契约。

## Impact

影响 `jarde-java` 的 lambda 站点计划、同类 helper 方法事实读取与 `class-source` 汇编/投影；不改 classfile/IR 原始事实、CLI/API schema、依赖或 `jarde-jvm`。冻结审计证据位于 `evidence/java8-lambda/`，表明 DT-25 存在一个已证差距，不代表整个单元已完成。
