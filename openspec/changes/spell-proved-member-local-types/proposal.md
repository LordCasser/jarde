## Why

Jarde 已能将具名成员类家族投影为嵌套声明和 `outer.new Member()`，但同一完整源码中的局部声明仍可写成 `Outer$Member member`。冻结 `OuterSuperEffects` 因此无法 Java 8 重编，阻断已证明 `Outer.super` 桥的整类验收；这是 DT-01/DT-03 的类型拼写闭环，不应混进桥方法规则。

## What Changes

- 对已选物理定义和已证 `InnerClasses` 成员路径，将局部声明中的准确成员类型写成 Java 的 `Outer.Member` 形式。
- 保留原二进制类型用于语义检查和物理报告；无唯一源路径、泛型实参无法说明或名称有冲突时维持保守输出，不基于 `$` 字符串替换。
- 用冻结家族的原/JADX/Jarde 全类重编运行与拒绝对照验证局部、循环头等声明位置，同时复核 `Outer.super` 效果夹具。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在局部声明类型的物理成员关系与源路径已证时，源码 SHALL 使用可解析的成员类类型名且保持内部语义类型不变。

## Impact

涉及 `jarde-java` 的声明 AST/发射及既有成员源路径证据消费，`class-source` 家族输出复用这些结果；不改变 Reader、JVM IR、CLI schema 或 `Outer.super` 桥证明器。
