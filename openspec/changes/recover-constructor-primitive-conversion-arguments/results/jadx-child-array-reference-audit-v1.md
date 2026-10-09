# JADX 子数组 initializer 与协变参考审计

只读检查了 `/Users/lordcasser/workspace/testzone/jadx` 的源码和集成测试，并对 `ReplaceNewArray.processNewArray`、`verifyPutInsns` 与 `TestMultiDimArrayFill` 做了 scoped Atlas 查询后再按当前文件行号复核。Atlas 打开的项目当时没有预建索引；查询只覆盖指定文件，以下结论来自文件本身，不代表运行了 JADX 或任何测试。

## JADX 中可复用的组合形状

`jadx-core/src/main/java/jadx/core/Jadx.java:166-172` 把 `ReplaceNewArray` 放在 `ModVisitor` 与 `CodeShrinkVisitor` 之后、region 建立之前。visitor 在 `ReplaceNewArray.java:40-67` 逐 basic block 扫描，并在改写后 shrink、再次迭代，最多 101 轮。对某个 `NEW_ARRAY`，`processNewArray`（`ReplaceNewArray.java:77-181`）要求长度是常量且非零；一维 primitive 数组可用 `len / 2` 作为最少 store 数，其余形状要求覆盖长度（第 87-95 行）。它从该数组 SSA variable 的 use-list 收集 `APUT`，要求存储索引为常量且小于长度；同一索引的后续写会截断收集，第 109-141 行。

收集成功后，visitor 按索引构建 `FilledNewArrayNode(elemType, len)`，用 `elemType` 表示父数组的组件类型，并把各个 `APUT` 的值作为 initializer 参数；缺少的槽位填零，第 143-168 行。替换位置取最后一个 store，并在有其它 reader 时向前移至第一个 reader，第 170-180 行。`verifyPutInsns` 还要求所有被收集的 stores 属于同一条 instruction list（第 184-203 行），并拒绝元素值再次读取父数组。`FilledNewArrayNode.getArrayType()` 是 `ArgType.array(elemType)`（`jadx-core/src/main/java/jadx/core/dex/instructions/FilledNewArrayNode.java:8-23`）；codegen 在 `InsnGen.java:705-725` 输出 `new <array-type>{...}`。这些部分提供了一个有用的组合模型：每层数组有自己的 filled-array 节点，子数组值可以成为父 initializer 的元素。

最明确的集成例是 `TestMultiDimArrayFill.test`：输入同时含 `new int[][] { {1}, {2}, {3}, {4, 5}, new int[0] }` 和另一个 `int[]`；断言输出保留为嵌套 initializer 与显式空数组（`jadx-core/src/test/java/jadx/tests/integration/arrays/TestMultiDimArrayFill.java:12-30`）。它证明当前测试预期包含这种具体的、同型 primitive 子数组组合。`TestArrays3.test` 则断言 `Object[]` 可保存一个已有 `byte[]` 值（`jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrays3.java:12-28`），但该元素是参数局部别名，不是 inline 子数组；它不覆盖 `Base[][]`/`Derived[]` 或其它 reference-array covariance。`TestNewArrayOfArrays.test` 检查的是部分维度分配和 `multianewarray`，不是 child store 组合（`.../TestNewArrayOfArrays.java:11-38`）。在当前 `jadx-core/src/test/java` 与 `src/test/smali` 的数组查询中，没有找到 `Base[][] { new DerivedA[] ... }` 这类用户类子数组协变用例。

## 不能照搬的门槛

JADX 这一 rewrite 是针对已有 DEX `NEW_ARRAY`/`APUT` IR 的局部规范化。`processNewArray` 用父 `ArgType` 直接建立 typed `FilledNewArrayNode`，代码里没有调用 Java class-header 或逐 `APUT` 的 source assignability proof；这不能替代 Jarde 生成可独立编译 Java 文本所需的赋值证明。尤其不能把“DEX 中父组件类型与 child 实际数组类型不等但 verifier 已允许该值”泛化成任意 Java initializer 可呈现。

其它门槛也属于这个 optimizer 的适用域，不能作为 Jarde 的宽松条件：长度必须是非零常量；index 必须是常量；重复 index 会停止采集；所有 stores 必须落在同一 basic block；一维 primitive 数组允许只覆盖一半并用零补齐。它按索引排序 stores，而 `firstNotAPutUsage` 只负责调整新节点位置；代码还留有 `TODO: check that all args already assigned`（`ReplaceNewArray.java:170-177`）。相邻的 `TestArrayFill2.test2` 明确带有 `@NotYetImplemented`，目标是保留 `a++` 后再用 `a * 2` 的求值次序（`jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrayFill2.java:25-37`），所以不能把这个路径当作 side-effect 顺序已全面证明的正例。

这些限制与 Jarde 目前需要的证明方向不同：Jarde 已有 exact child value/parent-store 身份、同 block、唯一 reader 与闭合 interval 检查；不能为了借用 JADX 的简单组合算法而放宽这些 ownership 和 effect 条件。JADX 的 primitive 半覆盖、按常量 index 排列或单 block 检查都不构成 child-array covariance 的类型证明。

## 与当前 Jarde 类型证明的对应关系

当前 `crates/jarde-java/src/build.rs:13215-13251` 已从 parent `aastore` 的 stored `ValueId` 找到 consumer 正好为该 store 的 child 候选，并由 child `NewArray` 的 element/rank 形成完整 `child_type`。过早截断点是 `component != child_type`：这里把类型相等当作结构候选成立条件，发生在 Builder 呈现元素之前。结构层已有 parent/child 的唯一 reader、producer/consumer 身份与 interval 检查（`build.rs:13700-13767`）；完整 parent chain 和其 Sites 一起提交（`build.rs:12735-12815`）。这些是 ownership 不变量，应保留。

Java 类型判定已有正确入口，不需要借用新类型表：Builder 在 render initializer 元素时将准确 `store_bci` 传给 `array_initializer_element`（`build.rs:25302-25366`、`26680-26747`）。它先接受 null、完全相同类型和 `Object`，再调用 `initializer_reference_widens`。该 helper 使用 Java release 对应的 scalar/platform 闭表、数组形状规则和相同 BCI 上完全匹配的 snapshot `source`/`target` proof；对 reference leaf 只提升相同 rank，不接受 rank 差或 primitive leaf 转换（`build.rs:29402-29471`）。现有单元测试明确覆盖 `Integer[][] → Number[][]` 等价秩路径、方向/rank/primitive 拒绝和 `Child[] → Parent[]` proof 的错 BCI/错 target 拒绝（`build.rs:33563-33649`）。

snapshot proof 的生产者位于 `src/facade.rs:25162-25215`：扫描真实 `aastore` opcode `0x53`，从同一 BCI 的 SSA operands 取得 source/target array types，核对 rank 后，沿当前选中的 snapshot class-header chain 产生带完整数组类型拼写和 store BCI 的 `ProvedSnapshotHierarchyWidening`。它只在 class-source assembly context 中提供给 Builder；method-only 请求不伪造该事实（`src/facade.rs:40795-40819`；载体定义在 `crates/jarde-java/src/report.rs:266-280`）。因此可复用的分工是：结构证明确定 child 是哪个 parent store 的元素；Builder 再在同一个 store BCI 上决定该 Java 赋值是否可证明。类型证明缺失时，整个方法保持 fallback。

这些结论是静态源码与测试断言的审计，没有执行 JADX、Java、Cargo 或测试套件。
