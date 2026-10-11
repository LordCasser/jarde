# v2 补充：Object foreach header、原 cast 与 generic refusal

本补充修正 v1 中“`for (element : iterable)`”过于含糊的描述。当前 `iterable_for_each_candidate` 的设计已经显式产出 `Object` header，并把原有 cast/绑定留在循环体；本片不需要先修 generic Signature，也不应另造 synthetic element/cast 机制。这里是源码静态审读，不是候选运行或 Java 编译结果。

## 候选中真实的类型路径

`Builder::iterable_for_each_candidate` 位于 `crates/jarde-java/src/build.rs:21224`。它从 `body.first()` 读取已经初始化的元素声明。元素值若是直接 `next()`，候选只接受它本来已呈现为 `Object` 且循环体有 cast 声明的分支；若是 `Cast(next())`，候选记下原 cast BCI。两支都精确校验 `next()` 结果的唯一 consumer；cast 分支还校验 cast 输出唯一交给原元素 Store，未 cast 分支则核对 Store 值唯一进入原 cast 输入。关键代码在 `21303–21314`、`21560–21611`。

最终改写在 `21690–21708`：它只把循环体首个元素声明内部的原 `next()` 调用节点替成 synthetic local `iteratorElement{next_bci}`，并给该 local 呈现类型 `java.lang.Object`；外层已有 Cast、原绑定声明及其类型留在 body 中。返回的 `StmtKind::ForEach.ty` 同样是 `java.lang.Object`。对这片输入，正确的归一化形状是：

```java
for (java.lang.Object iteratorElement47 : l2) {
    java.lang.String s2 = (java.lang.String) iteratorElement47;
    // 原循环体其余语句
}
```

这只是由候选 AST 变换推得的形状示意，不是本次实际生成文本。候选不会输出 `for (String s2 : rawList)`，也不会把原 cast 删除或搬进新转换规则。

输入候选要求 `iterable.presented` 是引用类型，并把 `iterator()` 的接口 owner 限定为 `java/lang/Iterable`、`java/util/List` 或 `java/util/Collection`，且呈现名称与相应平台接口相符（`21400–21440`）。它还精确要求 `iterator()Ljava/util/Iterator;`、`hasNext()Z`、`next()Ljava/lang/Object;`。因此当前 header 的迭代目标是 Java 源码可识别的 Iterable 子类型，header 元素目标类型则固定为 Object。

## JLS 约束与可编译性范围

JLS Java SE 8 §14.14.2 定义：对 Iterable enhanced-for，若表达式是 `Iterable<X>` 子类型，合成 iterator 类型是 `Iterator<X>`；否则为 raw `Iterator`。等价展开中的 header 变量类型作为 `TargetType`；raw Iterator 的缺省目标类型为 `Object`。因此 raw `java.util.List` 配 `Object` 元素变量在类型上成立；参数化 `List<String>` 配 `Object` 也成立，因为 `String` 可赋值给 `Object`。随后 body 原有的 `(String)` cast 仍然执行。这里的成立范围只说明这对 header/保留 cast 的 Java 类型关系，不证明整份 Jarde 输出可编译或语义已验收。规范：[JLS §14.14.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-14.html#jls-14.14.2)。

如改成 `for (String s2 : rawList)`，raw iterator 的 `next()` 提供 Object，header 的 String 目标会在 JLS 等价展开处承担 Object→String 转换；虽然语法仍可编译，但转换失败时点、绑定/转换表示和当前候选的 consumer 证明都需要单独审查。当前 candidate 的 Object header 加保留 cast是已实现结构所表达的路径，不能把一个假设的 String header 当成它的输出，也不能以 `List<String>` 未恢复为由拒绝现有 Object header。

## generic Signature refusal 是另一道证明

冻结基线 `UpstreamNestedLoops.java` 的输入确实写 `List<String>`，classfile 方法 descriptor 却只含 `Ljava/util/List;`；其 Signature 与真实字节码分别在 pin 的 Java 输入和 `javap` 文件中。冻结 Jarde 源码文本仍将两个参数写 raw `java.util.List`，并带有：

`ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under parameterized types`

该 refusal 的生成位于 `src/class_source.rs:7178–7180` 的 `ordinary_parameterized_declaration`；当已恢复方法没有可用 `GenericReturnCandidate` 时，`7400–7403` 直接以此原因拒绝参数化方法源声明。foreach 候选的 AST/header 改写既不产生该 proof，也不改变该拒绝逻辑。把 raw List 的 header 元素写成 Object 不要求先恢复 List 参数，所以 generic refusal 可保留且不妨碍这对 header/body 结构的类型相容性。

这不意味着 generic refusal 可以移除：完整方法的 generic source projection 仍无 same-run body proof，class/method 输出仍可能带“presentation is not claimed to compile”等既有标记。本片只确认此 generic 缺口不迫使 foreach 使用 String header，也不应将 Signature 恢复混入下一片。若以后追求 `for (String s2 : l2)` 这种更接近输入源码的 typed header，先要有 iterable 的可靠源类型及完整 consumer/cast 关系证明，并另立需求；不要从 JADX 类型修正推导 Jarde 已有证明。

## 与 JADX 的差别

JADX `LoopRegionVisitor.checkIterableForEach`（`246–338`）在消费 iterator SSA 后建 `ForEachLoop`。wrapped `next()`/cast 路径对 cast result 调 `fixIterableType`；无 wrapped cast 则对 `next()` result 调整。`fixIterableType`（`342–377`）会检查已有 iterable 泛型参数与循环变量类型是否一致，或尝试把可变 iterable 参数类型设为 `Iterable<elementType>`；已有 immutable type 冲突时，只允许 `Object` 循环变量。随后 `RegionGen`（`199–205`）直接输出 foreach variable 与 iterable。

JADX 可以用其自己的 ArgType 与 SSA rewrite 选择 typed element variable。Jarde 当前显式选 Object 且保留现有 body cast。JADX 的 fixup 既不能替代 Jarde 的 generic Signature/body proof，也不提供把 Jarde header 直接改成 String 的理由。

## 结论边界

当前 private design 应保留已有 Object-header/原 cast路线。它不需要先做 Signature 修复，也不需要新增 typed foreach/cast 逻辑。它仍受 Jarde 完整 SSA、唯一使用、effect/handler、iterator 初始化位置和 source owner gate 约束；本分析没有运行 candidate、没有观察新 Builder 最终输出、没有调用 Java 编译器。对当前被冻结基线，最多可说 raw List + Object header + 原 String cast 这一类型组合符合 JLS 结构；不能说此 foreach candidate 已通过完整流水线或整个 class 可编译。v1 的“`for (element : iterable)`”须按上面的 Object header 具体化；v1 其余 lexical owner / 外层 Assign 分片边界仍有效。
