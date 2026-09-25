# 增强 for 投影后的失用声明：架构定位

现有[数组增强 for 独立验收](../../../changes/project-proved-enhanced-for-loops/verification-root-array.md)与[安全包装读取验收](../../../changes/project-proved-wrapped-array-elements/verification.md)均记录了真实的 `for (int … : …)` 投影；后者还记录失去用途的顶部 `int local3; int local4;`。它们不影响 Java 8 重编或运行，却是可见的源码质量缺口。本文件最初是下一步设计定位；当时未声称已在当前工作树重建 CLI 或完成三方重放，当前复验记录见文末。

`crates/jarde-java/src/build.rs::declarations` 在 AST 生成之前，按 `LocalVariable` 身份和 RegionPath 的 SSA 使用集决定前置声明。`declare_at` 把它写成没有初值的 `StmtKind::Declare`，来源锚在该变量首次写入的 BCI。随后 `array_for_each_candidate` 证明长度缓存、归纳下标、数组读取与元素绑定，成功时将 `for` 的初值/更新及紧邻的数组长度赋值并入增强 `for` 的 `OriginSet`。因此原先为了计数循环而前置的缓存/下标声明在 AST 中滞留；这不是声明规划最初的错误，而是后续投影改变了活跃用途。

修复接缝应在**成功投影的同一次提交**内：候选已经持有长度 `Store` 的槽及 BCI、`ForHeader` 的归纳槽和 `init` 来源，可由 `reuse.variable_at(slot, bci)` 得到两个真实 `LocalVariable`，再与 `Declarations::at_region` 的同一身份、声明锚点及 `NameTable` 名称交叉核对，只删除这两个没有初值且被整个投影消费的声明。`array_for_each_candidate` 已检查长度值除循环测试外没有其它用途、归纳 phi 的使用只在测试/元素读取/更新与允许的 join 内；证明失败仍保留原计数循环与声明。不可仅按 `local3` 字符串、槽号或全文正则删除：同槽可承载多个词法变量，且拒绝路径仍可能读取声明。

删除前需确认其 BCI 已包含在新 `ForEach` 的来源集合；当前候选对 `length_stmt`、`init`、`update` 逐项调用 `stated_by_statement` 并合成来源，但要由测试精确核对。紧预算/取消时不能发布部分删除；essential/all 正文一致。后续三方重放应复用现有数组正例与 `indexAfterLoop`、错配数组、包装读取顺序负例，另加同槽复用且后继仍需声明的 Java 8 fixture。这个清理只处理成功数组投影认领的两项缓存，不展开为全局“删除所有未使用局部”优化器。

新增 [ReuseAfterForEach.java](ReuseAfterForEach.java) 与 [ReuseAfterForEachRunner.java](ReuseAfterForEachRunner.java) 给出了身份边界。`javac --release 8 -g` 的 `sumThenReuse` 在 BCI 3 写入合成数组别名槽 2、BCI 6 写入合成长度缓存槽 3、BCI 8 写入归纳下标槽 4；循环结束后 BCI 36 又把 `first` 写入槽 2，BCI 40 把 `second` 写入槽 3。LVT 中 `first` 的槽 2 生命周期从 BCI 37 开始，`second` 的槽 3 从 BCI 41 开始；两者与缓存恰好复用槽号，却必须在投影后保留各自声明。`-g`/`-g:none` 原 class 在 `java -Xverify:all` 下均依次输出 `22`、`4`、`NullPointerException`。

fixture SHA-256：源码 `d89a811487925e7445d25261a92a70646906d8cb357d4c52607c08e099aeba31`；runner `6cfcfe99ebfeeeb85ab22f48e12003665a302824eec9515d191decaf02a96bf5`；`-g` class `ad6ffab7c9fdb00cd733486d09414ab47421483ee6fa0f241bcba4ee8088cfd4`；`-g:none` class `3fdc5853f0bb1e64071c1bf0b809d851e74fb41a576b0d273fe5085cd1fd548b`。

已用本机 JADX 1.5.6 分别重放两个 class：`-g` 几乎原样恢复增强 `for` 与 `first`、`second`，源码 SHA-256 为 `3d449a0c6a90ba7abbc09a6072669c9531d6b8fadaec5decdbfc2fa69e58c6e2`；`-g:none` 也恢复增强 `for`，把后继两个局部化为 `int i3 = i + 1; return i + i3 + i3 + 2;`，源码 SHA-256 为 `483f00a7f0a54934a390b19452e8582b6e8b5b00b4fd704b92676d5c348b2878`。两个 JADX 完整类及同包 runner 均用 `javac --release 8` 重编，`java -Xverify:all` 均与原 class 的三行输出一致。后一种文本依赖值传播，因此不能当作 Jarde 可以按槽删掉后继声明的证据。

同一构建的 Jarde CLI 对两个 class 都投影了增强 `for`，但 `-g` 文本在方法头声明 `int[] first; int second; int local4;`，循环后却写 `first = sum + 1; second = first + 2;`；`-g:none` 同样声明 `int[] local2` 后赋给它整数。以正确文件名重编各完整类，`javac --release 8` 各报三处数组与整数类型冲突，因而不能运行 Jarde runner。Jarde 源码 SHA-256 分别为 `f05b6c3fab4f692cddddc06fb261ed1220719e35ca182580232e94fad0e4e97c`、`53b5e381bfaab5d748fcb2f1af889fe496ff10160c88f5d0759bbad5c0cff9c2`。这不是缓存声明清理能解决的：`reuse::plan` 对一个槽只有一个 LVT 名称时会把整个槽记成 `Whole`，即使该名称仅在循环后生效；无 debug 时也默认单槽单变量。槽 2 的数组别名和后继整数因此被绑定成同一个 `LocalVariable`，`declarations::decide_types` 选择了数组类型。须另立局部生命周期/类型一致性修复，不能把它塞进 [缓存声明清理 change](../../../changes/prune-proved-foreach-cache-declarations/tasks.md)。后者只应确保不会进一步删掉槽 3 后继需要的声明，并在其已有可编译数组正例上完成行为验收。

本地 JADX 的思路可借，但无需照搬 Dex 结构：`jadx-core/.../visitors/InitCodeVariables.java` 为每个 SSA definition 建 `CodeVar`，仅将 phi 连通的 SSA 变量合并，并拒绝一组中多个不同的 immutable type；`debuginfo/DebugInfoApplyVisitor.java` 将 LVT 名称/类型贴到对应 SSA 变量前先通过 `TypeUpdate` 检查，冲突时拒绝 debug 名称；`regions/variables/ProcessVariables.java` 再按 `CodeVar` 的 assign/use 位置定声明。Jarde 已有 SSA 值、phi、帧类型与 `LocalVariable`，可在 `reuse::Plan` 现有身份证明上考虑“无 LVT 覆盖的前缀”和“不同类型且定义-用途不跨越的后继”分段；同一个 SSA/phi 连通分量不得拆，无法证明时应回退而非让一份 `int[]` 声明承担后继 `int`。这仅是独立 change 的方向，仍需研究异常边、跨块 phi 与 source scope 才能定实现准入。

## 2026-09-24 当前工作树复验

本次复验在实现完成后重新编译了 CLI：`/tmp/jarde-foreach-cache-impl-target/debug/jarde-cli`，SHA-256 `ab24a57446e35b0622885efb53b12e0175f4bab6cc2db47d871c4609c3af55ed`。工具为 `javac 23.0.1`（OpenJDK `23.0.1+11-39`，所有编译指定 `--release 8`）、JADX `1.5.6`。直接数组输入使用[既有源码](../../java-syntax-2026-09-22/enhanced-for/IntArrayForeach.java)和 runner；安全包装输入使用[既有源码](../array-wrapped-binding/ArrayWrappedBinding.java)和 runner。直接数组源码/runner SHA-256 分别为 `0b29cc718a614df7e5a4831370f02f1ff6c2af73c8db0232f5e3d9b9f622ed8c` / `91a45e23e43fa02d6fa9c1385b76e28fb4e832ff851ecf7e1711c11fb66cc8f1`；安全包装源码/runner 为 `c3ae78d1a80a205fe9cde417568e3ee54230493b48debdbd3db2be2924933d38` / `9907e757c9fd6f4e91e392707e1695854c1cb7b3005e5a5b43159c3be2ce51f0`。

两个原始源码与 runner 分别以 `javac --release 8 -g` 和 `-g:none` 编译；原 class 直接用 `java -Xverify:all`，JADX 对各 class 单独反编译后与同包 runner 重编，Jarde 用当前 CLI 的 `class-source --input <class> --class <name> --policy single-class --release 8 --format text --evidence essential` 输出完整类再与 runner 重编。JADX 生成物位于 `defpackage`，因此仅给 runner 加上相同 package 声明。六组 generated class/runner 均编译成功；Jarde 输出里的四个正例方法都写增强 `for`，不再带缓存 `local3` / 下标 `local4` 的无值声明。Rust 集成测试也对直接与安全包装正例逐项断言这两项声明不出现。

| 完整类输入 | 原 class SHA-256 (`-g` / `-g:none`) | JADX 源 SHA-256 (`-g` / `-g:none`) | Jarde 源 SHA-256 (`-g` / `-g:none`) | 规范化运行输出 SHA-256 |
| --- | --- | --- | --- | --- |
| `IntArrayForeach` | `a799f59d61988191d93e562fa02e1e0ef8a0a83e044f9dfe6fdf396b587c611f` / `067bf3c5b231811f97d19e9137c7d4019eb2849f8cce91706285120db7d55b58` | `126e3ae61088f0fcc699f737b3c3558cabcb86feabb1ba04d22163fb301f8f95` / `3bda59fe8ca15c73540d1e414d9ca98a347356f0ee3e20c6a440731b87805016` | `0bad4b54176b224c6765aa0cd320362e24fe73ffac5a5e48584e93d752bc57ea` / `6d9024ca27bce5b87ce82fc23a85c3644be1c6398923c9e36f20c78c933d2465` | `866dda7144ee96dd0462b8cb5e8d695c87cfeefcdcb1c0bb58808938f6aa4048` |
| `ArrayWrappedBinding` | `2081422d77ba8bb0b4abfd443b5b00a2de5f0278d2695c3a178283bb3a5a9961` / `0f024e6ebb40b16a0da82f2058374f715e233b13457bdb952abc236767d1d9ca` | `f375c56aeb1e560d30b988c40f3cb7918e033dc202ede72f31dfb6f3380c052e` / `ef24805500d3b961b03890088cf4e17d8ac2974c24a71c7edf8ab0a38e0b8a87` | `c1f83ddcbd26c611aba9204f98cf59d12dff008edc9d614a0ad0ef6cbf066cb4` / `b4ba774f2dcb8b1bc12a4cf86f620340283bc923db07b83f48de5b56f43b6062` | `693225e312826469439f7f23d7b9206d22226c307796be23bf0065c3b026625a` |

原始 stdout 的 SHA-256 依次为：直接数组 `-g` / `-g:none` 的原 class 与 JADX 都是 `ad63f502464a1cc3e09c6ce0cfa3d7a8787917964581757675f4b438e0246e9e`，Jarde 都是 `f6f49a74f89a6ee5bf5db3d253965483c70dda11926b3e44bbb6d6544caa72a4`；安全包装 `-g` 的原 class/JADX/Jarde 为 `babf4756c6e1621a0f08ba0cf7dcbbdac933fba30379dfa524a27fa23e35d4dd` / `5ca910a0e3a21208c1793e4c6fa5033ae285c40dc68722e115cb9646ca92197f` / `be88fe4fe452a92212deefac555474eda56f143365bec32c1f0ee38ac5c62c78`，`-g:none` 为 `a61c5de0d1dbcd2150d6275226386498b543c944a8d833978461f0803c943875` / `5ca910a0e3a21208c1793e4c6fa5033ae285c40dc68722e115cb9646ca92197f` / `be88fe4fe452a92212deefac555474eda56f143365bec32c1f0ee38ac5c62c78`。

规范化只替换 JVM 自动生成的 `NullPointerException` 详细文本（局部名会因重编而从 `<local2>` 变为 `<local3>`）；异常类型、调用计数、数组副作用和其余输出均保留。直接数组三方输出一致。安全包装 Jarde 与原 class 一致；JADX 的完整探针 runner 在 `tick-first`、`call-mutate-array`、`read-after-effect` 上仍分别输出 `4`、`1003`、`3`，原 class/Jarde 为 `92`、`1091`、`91`。这是已有安全边界负例，两个能投影的正例 `array-first`、`wrapped-only` 在三方均输出 `4,calls=1,array=[3]`。

拒绝边界重放了 `ReuseAfterForEach` 的源码和 runner，`-g` / `-g:none` class SHA-256 仍为 `ad6ffab7c9fdb00cd733486d09414ab47421483ee6fa0f241bcba4ee8088cfd4` / `3fdc5853f0bb1e64071c1bf0b809d851e74fb41a576b0d273fe5085cd1fd548b`，两个原 class 的 `java -Xverify:all` 输出均为 `22`、`4`、`NullPointerException`，stdout SHA-256 均为 `cb653a450c8c0a381e6e4fc7c445ede68fe78c6971b587f8a398aedcf7dfe31a`。`javap -c -l -p` 的 `-g` 输出 SHA-256 为 `d88e9901a887908be64b4692929cfd4de0bc0c3c136688eae557400654c86408`：BCI 3/6/8 分别写 slot 2 数组别名、slot 3 长度缓存、slot 4 归纳下标；BCI 36/40 又将后继 `int` 写到 slot 2/3，LVT 中后继 `first` / `second` 的可见范围分别从 BCI 37 / 41 开始。当前 Jarde 输出继续保留 `int second;` 和 `second = first + 2;`；新增集成回归在测试中编译同一份 `-g` 源码并断言这两项存在。

这一拒绝类完整 Jarde 源重编依旧失败：`-g` 的 SHA-256 为 `22461e5c20287ff9f873ab273ab5785d688010f6da2de5ac9e94b9fdc365c8ee`，`-g:none` 为 `c91d1d35cf7387c782ff63167a17f0f204f1adb5f1a2a098b19e171ef521cf19`。两者各有三处 javac 类型错误，都是 slot 2 的数组别名被命名/定型为 `int[]`，后继整数却写回同一源码局部；`-g` 源仍有 `int second;`，无调试信息的输出仍有后继 `int local3;`。这确认了本清理没有进一步删掉 slot 3 的后继声明，也确认基线 slot 2 生命周期/类型缺陷没有被本项修复。`ArrayForeachRefusal` 的 `indexInBody`、`differentArrays`、`escapedElement`、`indexAfterLoop` 继续由 [`p3_array_foreach`](../../../../tests/p3_array_foreach.rs) 断言保留计数循环；wrapped 负例与转移边界也在该测试和 [`p3_wrapped_array_foreach`](../../../../tests/p3_wrapped_array_foreach.rs) 覆盖。

复现命令主体：

```sh
javac --release 8 -g -Xlint:-options -d "$OUT/original" "$E/$CLASS.java" "$E/${CLASS}Runner.java"
javac --release 8 -g:none -Xlint:-options -d "$OUT/original-none" "$E/$CLASS.java" "$E/${CLASS}Runner.java"
java -Xverify:all -cp "$OUT/original" "${CLASS}Runner"
javap -classpath "$OUT" -c -l -p ReuseAfterForEach
jadx --no-res --single-class "$CLASS" -d "$OUT/jadx" "$OUT/original/$CLASS.class"
jarde-cli class-source --input "$OUT/original/$CLASS.class" --class "$CLASS" --policy single-class --release 8 --format text --evidence essential --output "$OUT/jarde/$CLASS.java"
javac --release 8 -g:none -Xlint:-options -d "$OUT/jarde-classes" "$OUT/jarde/$CLASS.java" "$E/${CLASS}Runner.java"
java -Xverify:all -cp "$OUT/jarde-classes" "${CLASS}Runner"
```
