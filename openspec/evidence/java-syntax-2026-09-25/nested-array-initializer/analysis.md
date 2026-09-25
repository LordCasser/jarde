# 多维嵌套数组初始化与 JADX 的求值顺序反例

## 冻结的 Java 8 输入

[`NestedArrayInitializer.java`](NestedArrayInitializer.java) 同时含 `new int[][] {{element(1), element(2)}, {element(3)}}` 的带副作用内层元素和 `String[][]` 字面量。`javac --release 8 -g:none -Xlint:-options` 产生 52.0 class，SHA-256 为 `c1ad514b609bc1eefeb0f5de73e7e622bd7fa07b34a31749a18e77fd9d74f24a`；[`javap.txt`](javap.txt)显示外层 BCI 1 `anewarray [I`，每个内层 `newarray int`、`iastore` 后再由 BCI 23/36 `aastore` 写进外层数组。原 class 经 `java -Xverify:all` 输出 [`[[1, 2], [3]]:3:123`](original-run.txt) 和 `[[a, b], [c]]`。三个 `element()` 仅按 1→2→3 求值一次。

安装的 JADX 1.5.6 对同一 class 写出[完整类](jadx-output/sources/defpackage/NestedArrayInitializer.java)，对内外层均投影为 `new T[]{…}`，仅移除它给无包类加入的虚构 `package defpackage;` 后以 Java 8 重编；[`jadx-run.txt`](jadx-run.txt)与原类两行逐字相同，重编 class 还与原 class 逐字节相同。实施前 root 独立构建的 CLI 对同一 class 输出[旧完整类](jarde-nested.java)和[旧全量报告](jarde-nested-report.json)：`dynamic()`、`literal()` 均是 explanation-only，写出的方法缺少 return，Java 8 编译给出[两个错误](jarde-nested-javac.txt)。因此这是确认的 Jarde 基线缺口，不是仅凭旧一维规格推断。

## 同一 JADX 折叠算法的可执行错误

[`JadxOrderingControl.java`](JadxOrderingControl.java) 是独立边界：先 `result[1] = mark(1)`，再 `result[0] = mark(2)`；[`ordering-javap.txt`](ordering-javap.txt) 的调用/存储位置分别为 BCI 7/10 与 14/17。冻结 class SHA-256 `aeae53c70a3bab1e9e341829ad9cd2f55a18915b98d526a962654823aa7d4b15`，原 JVM 在验证模式输出 [`[2, 1]:12`](ordering-original-run.txt)。JADX 1.5.6 输出 [`return new int[]{mark(2), mark(1)};`](ordering-jadx-output/sources/defpackage/JadxOrderingControl.java)；同样只移除虚构 package 后重编执行为 [`[2, 1]:21`](ordering-jadx-run.txt)。数组值一致，**调用顺序已错**；[逐行差异](ordering-jadx-vs-original.diff)保留这一反例。

本地 JADX checkout `2fb1b163` 的 [`ReplaceNewArray.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/ReplaceNewArray.java) 在 `processNewArray` 用 `TreeMap<Long, InsnNode>` 收集/排序 `APUT`，再按索引顺序给 `FilledNewArrayNode` 添加实参；`verifyPutInsns` 只核对这些 store 同块且值未复用数组，没有证明其物理执行顺序与索引顺序相同。`InsnGen.filledNewArray` 依该实参顺序发射源码。这个源码事实与上述运行差异吻合：将两次有副作用的 `mark` 从物理 1→2 改为源码 2→1。不能把 JADX 的可编译输出或仅相同的最终数组值当作初始化器安全证明。

root 同一 CLI 对排序反例输出[完整类](jarde-ordering.java)和[全量报告](jarde-ordering-report.json)：保留 `local0[1] = mark(1); local0[0] = mark(2);`，Java 8 重编成功，[验证运行](jarde-ordering-run.txt)仍是 `[2, 1]:12`。这里 Jarde 当前规则比 JADX 正确；扩展初始化器不能退化这个边界。

修前 Jarde 的 [`ArrayInitializers::prove`](../../../../crates/jarde-java/src/build.rs) 仅准 `dimensions == total_dimensions == 1`，并按 `0..N-1` 顺序/同一 SSA 数组身份核验 store；[既有设计](../../../changes/recover-array-initializers/design.md)明确将多维嵌套与倒序索引排除。外层 `anewarray [I` 的 `element=int,total_dimensions=2` 以及内层 `newarray int` 的类型事实已经存在；`array_element` 能算出外层 store 的组件 `int[]`。阻断曾是证明计划的 rank 1 门槛、最终消费者没有 `ArrayStore`、父元素依赖扫描不能把已证明的子初始化链作为一个表达式，以及 `NewArray` 的带初始化器呈现只允许 rank 1。现已沿当前逐层物理顺序、SSA 身份、异常处理集合、预算与完整来源证明自内向外组合；没有引入全局别名分析或另一套 AST。内层创建值必须是父 `aastore` 唯一元素值，父子来源互不重复认领，也没有复制 JADX 的索引重排。

## 实施后的 root 独立对照

root 重新构建 CLI，取得[嵌套类报告](jarde-after-nested-root-report.json)、[完整源码](jarde-after-nested-root.java)以及[乱序报告](ordering-jarde-after-nested-root-report.json)。原/JADX/Jarde 三份 `NestedArrayInitializer` 完整类均用 Java 8 重编成功，`java -Xverify:all` 的[两行](nested-jarde-run.txt)与原/JADX 相同，而且三个重编 class 均与冻结原 class 逐字节相同（SHA-256 `c1ad514b609bc1eefeb0f5de73e7e622bd7fa07b34a31749a18e77fd9d74f24a`）。`dynamic` 的 28 个、`literal` 的 25 个真实指令起点分别全部有 source map，无多余 BCI；两方法均 structured、零 fallback。

乱序反例的原/Jarde class 也逐字节相同（SHA-256 `aeae53c70a3bab1e9e341829ad9cd2f55a18915b98d526a962654823aa7d4b15`），验证运行仍为 `[2, 1]:12`；JADX 重编 class SHA 为 `b7591a04901b623d936e340ef2cbec855dbff720b8ae64118e6575fbcddbd46a`，运行 `[2, 1]:21`。新增[子数组额外使用](../../../../tests/fixtures/p3-nested-array-initializers/NestedArrayExtraUse.java)经验证模式执行为 `[[4]]:4`，Jarde 保留先创建并读取子数组长度、后写入父数组的顺序；[完整源码](extra-use-jarde-after-nested-root.java)与原 class 重编字节也相同。CLI SHA 见 [hashes](jarde-after-nested-root-hashes.txt)；OpenSpec 验收见[独立记录](../../../changes/recover-nested-array-initializers/verification-root.md)。
