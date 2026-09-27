## Why

EM-01 的固定 `Generic.A<T> implements Comparable<A<T>>` Java 8 正例仍使 Jarde 完整根源码不能编译。物理 child 已可读取，但无构造使用点的成员关系、嵌套位置中的泛型 Signature 作用域，以及编译器桥方法三道证明尚未在同一个源单元闭合。固定 `Shape.I`/`Shape.A` 已恢复，故这是 EM-01 `multi` 回放的最后一个已知根级缺口。

## What Changes

- 在现有 class-source member family 中，把“根与静态 child 的声明关系”同“某个根方法的构造使用点”分开证明；无使用点时允许准确的单 child 泛型声明，仍检查唯一物理定义、双向 `InnerClasses`、完整成员和根使用闭包。
- 在根源码的词法上下文解析并核对 child 类、字段、方法 Signature，发布 `A<T> implements Comparable<A<T>>`、`T value` 与 `compareTo(A<T>)`，保持物理 child 独立报告的原样证据。
- 只在 `compareTo(Object)` 的 flags、Code、参数 cast、单次 typed 调用和返回都与泛型声明的编译器桥相符时，于根源码省略物理 bridge；否则整组拒绝。
- 将原/JADX/Jarde 完整 `multi` Java 8 源码和外部 API/bridge consumer 重编，以 `java -Xverify:all` 对照行为、泛型反射与桥方法；畸形签名、错关系/桥和预算/取消负例保守拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证的静态泛型成员类可在根源码中保持类、字段、方法泛型作用域及桥方法语义。

## Impact

沿用 reader Signature parser/erasure proof、现有 class-source 家族状态、来源与原子投影；新增的是私有的词法上下文与桥再生证书，不新增 JVM IR、公共 API、CLI 参数或依赖。[架构分析](../../evidence/java-syntax-2026-09-28/em01-generic-member/architecture.md)记录当前拒绝门及 JADX 参考。非 Java 8 lowering、多个泛型 child、复杂继承/bridge 另验。
