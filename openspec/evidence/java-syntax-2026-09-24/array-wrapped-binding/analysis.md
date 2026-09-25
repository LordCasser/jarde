# 数组元素读取被表达式包裹时的增强 `for` 边界

本探针覆盖 Java 8 下 `a[i]` 不再是元素局部的独立初始化式时，Jarde 与 JADX 的数组循环投影差异。输入包括 `a[i] + tick()`、`tick() + a[i]`、`consume(effect(), a[i])`、先有循环体副作用再读数组、一次迭代读取两次，以及 null 与抛异常路径。探针编译为单个待反编译 class；runner 只作为源文件加入三方重编执行。

## 当前结论

当前工作树 Jarde 对直接绑定形式保守：`array_for_each_candidate` 先要求循环体第一条语句为 `Declare { value: ExprKind::Index }`（[`build.rs`](../../../../crates/jarde-java/src/build.rs#L5422)），随后证明读取数组和边界长度引用同一捕获、元素局部用途受限、元素 load 只流向该局部等。因此 `a[i] + tick()`、`tick() + a[i]`、调用实参里的 `a[i]` 和副作用后的 `a[i]` 都留在计数 `for`。冻结的既有 `effectInBinding` 正是 `captured[i] + tick()` 形状；本探针也确认该限制仍在当前反编译文本中。

关键运行反例是：原源码的 `mutateAndTick() + a[i]` 求值为 `92`，JADX 输出的 `for (int i : array)` 因为提前读取而求值为 `4`；原源码的 `consume(mutateAndTick(), a[i])` 返回累积值 `1091`，JADX 得到 `1003`；先 `mutateAndTick()` 再单独读 `a[i]`，原源码读到 `91`，JADX 读到旧值 `3`。完整 Jarde 重编类三例均与原 class 一致。相对地，`a[i] + tick()` / `a[i]` 已先读出、之后才调用 `tick()`，JADX 与原 class 的 `array-first=4`、`wrapped-only=4` 相同；这正是能安全提到绑定位置的求值顺序。故最小准入条件应是元素 read 之前没有 observable effect，而不是笼统地只要求一次 array read。

本地 JADX `2fb1b1638694` 对前述表达式更积极。其 `checkArrayForEach` 允许无 result 的 wrapped `AGET`，找到 wrapper 后用合成迭代变量替换并隐藏旧 `AGET`（[`LoopRegionVisitor.java`](../../../../../../testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java#L178)）。此路径仅核对数组长度与元素读取来自同一数组，并未证明读取移到循环体入口后仍与原表达式中的副作用保持相同顺序。JADX 对 `consume(mutateAndTick(), a[i])` 与 `mutateAndTick(); value = a[i];` 都生成增强 `for`，令每轮的元素读取先于 `mutateAndTick()`。完整类重编后两条路径都把 `3` 当作元素值；原 class 则在副作用写入后读到 `91`。这是由本探针执行差异直接确认的 JADX 语义缺陷，不只是代码检查推断。

`a[i] + tick()` 中元素读取原本先于 `tick()`，JADX 的绑定顺序与它相同。`tick() + a[i]` 与 `consume(tick(), a[i])` 则要求先调用再读；本次无修改数组的 `tick` 让最终数值相同，但从 JADX 生成的增强 `for` 可以确认元素读取已被提到调用之前。关于这一点，“发生了顺序变化”是源码/字节码结构事实；若调用能改写数组，值会变化是基于 Java 左到右求值规则的推论，并由紧邻的写数组变体实测印证。两次读取夹着写操作的 `twoReadsWithMutation` 被 JADX 和 Jarde 都保留为计数 `for`，未把两次读折成一次。

null 输入都在读取 `length` 时先抛 `NullPointerException`，调用计数为 0；已有数组上的 tick 抛异常时，三方都报告同一异常且计数为 1；`tick-first` 先修改数组再抛异常，三方都留下 `[91]`。它们说明此组探针中长度捕获与异常路径一致；不能据此推出所有元素读取位置都等价。循环通过 `i < captured.length` 且只访问该同一局部数组时，正常元素 load 的边界/空指针异常已被循环条件和长度读取排除。真正的危险在先前调用或其它写操作可修改元素值，而非本探针中的 load 自身抛异常。

## 最小可行的 Jarde 扩展证明

可把“唯一、简单的元素声明”扩大为“唯一元素 load 的表达式消费者”，但必须按 Java 求值顺序证明而后提交投影：

1. 复用现有循环证明，保留 `i = 0`、`i < length`、`i++`、数组引用单次捕获、长度与读取来自同一数组值、循环边与 handler 约束。SSA 中归纳变量除测试、更新外只能流向该次 `ArrayLoad`；拒绝数组或索引逃逸、额外读取、额外 consumer、循环转移未闭合及元素局部越过循环生存期。
2. 允许该 load 的唯一 SSA use 是一棵可写出的表达式子树，而不是要求 AST 根必须是 `Index`。记录它在 Java 左到右求值序列中的位置。准入的最小充分条件是本次数组读取之前没有 observable effect；将它提成 `for (T element : captured)` 后，数组读取恰好成为每轮最先执行的 observable action。`a[i] + tick()` 满足；`tick() + a[i]`、`consume(effect(), a[i])` 和先执行过副作用语句的 shape 拒绝。数组读取之后的调用、副作用可保留在原表达式及其顺序中。
3. 若表达式在数组读取之前包含除法、转换、解引用等可能抛异常表达式，只有在已有数组与索引证明足以证明元素 load 不抛，并且不移动任何有副作用操作时才可考虑放行；最小闭环可以一律拒绝这类前缀。遇到两次 `ArrayLoad` 或 index 的额外使用一律拒绝，避免合并具有独立读取时点的值。
4. 先以不可变 candidate 记录并验证 SSA/AST use、顺序、类型、异常处理器、来源和预算；全部通过后才原子地产生元素变量并认领/折叠原指令。任何拒绝都维持既有计数循环及相同 origins，不能先隐藏读取再尝试回滚。

前三条中的“要求 load 早于所有有副作用前缀”是保守充分条件，而非所有语义等价情形的必要条件。未来要放宽必须有明确的别名/写效果证明；当前没有依据把未知调用当纯函数。只需支持任意 `Expr` 子树的这条小扩展，不需要构建通用表达式重写框架。

## 可复现输入与结果

源码和本地冻结产物如下；SHA-256 对应本目录文件：

| 文件 | SHA-256 |
| --- | --- |
| [`ArrayWrappedBinding.java`](ArrayWrappedBinding.java) | `c3ae78d1a80a205fe9cde417568e3ee54230493b48debdbd3db2be2924933d38` |
| [`ArrayWrappedBindingRunner.java`](ArrayWrappedBindingRunner.java) | `9907e757c9fd6f4e91e392707e1695854c1cb7b3005e5a5b43159c3be2ce51f0` |
| [`ArrayWrappedBinding.class`](ArrayWrappedBinding.class) | `0f024e6ebb40b16a0da82f2058374f715e233b13457bdb952abc236767d1d9ca` |
| [`ArrayWrappedBinding.jadx.java`](ArrayWrappedBinding.jadx.java) | `ef24805500d3b961b03890088cf4e17d8ac2974c24a71c7edf8ab0a38e0b8a87` |
| [`ArrayWrappedBinding.jarde.java`](ArrayWrappedBinding.jarde.java) | `107634107ba832a5ad891e0eb4250b09be73813c452c63f6eeefc8bb7e5f0b38` |
| [`ArrayWrappedBindingRunner.jadx.java`](ArrayWrappedBindingRunner.jadx.java) | `19f3d1e58312befe8f47596ba4e940b15f09307d2c4420beb73645f243e36586` |
| [`original.execution.txt`](original.execution.txt) | `a61c5de0d1dbcd2150d6275226386498b543c944a8d833978461f0803c943875` |
| [`jadx.execution.txt`](jadx.execution.txt) | `5ca910a0e3a21208c1793e4c6fa5033ae285c40dc68722e115cb9646ca92197f` |
| [`jarde.execution.txt`](jarde.execution.txt) | `a61c5de0d1dbcd2150d6275226386498b543c944a8d833978461f0803c943875` |

工具为 `javac`/OpenJDK `23.0.1+11-39`（`--release 8`）、JADX `1.5.6`（源码 commit `2fb1b1638694`，本地 source path `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java`），以及当前 CLI `/tmp/jarde-iterable-remaining-target/debug/jarde-cli`（SHA-256 `35f7b4845b6386fa16b8aa0b20dc406e88a64d7d8804c17c70cd43950936ac18`）。原始 class 用 `javac --release 8 -g:none -Xlint:-options` 从上列源文件构建。JADX 用 `jadx --no-res --single-class ArrayWrappedBinding -d <output> <class>` 反编译。Jarde 用 `jarde-cli class-source --input <class> --class ArrayWrappedBinding --policy single-class --release 8 --format text --output <file>` 输出完整 class。

原 class、JADX 完整类和 Jarde 完整类分别与 source-only runner 用 `javac --release 8 -g:none -Xlint:-options` 重编；JADX runner 加 `package defpackage;` 以匹配其类源码。三次编译均成功，再分别用 `java -Xverify:all` 执行。原始与 Jarde 输出逐字节一致（两份执行文本 SHA-256 相同）；JADX 的两个具体差异为：

| 方法 | 原 class / Jarde | JADX | 原因 |
| --- | --- | --- | --- |
| `plusTickFirst` | `92`，数组变成 `[91]` | `4`，数组变成 `[91]` | 原式先调用 `mutateAndTick()` 再读元素；增强循环先缓存 `3` |
| `callMutateThenArray` | `1091`，数组变成 `[91]` | `1003`，数组变成 `[91]` | 增强循环在 `mutateAndTick()` 前缓存元素 `3` |
| `readAfterEffect` | `91`，数组变成 `[91]` | `3`，数组变成 `[91]` | 增强循环把循环体后续的读取移到此前面的调用之前 |

JADX 完整输出还将 `plusArrayFirst`、`plusTickFirst`、`callTickThenArray` 与 `wrappedOnlyRead` 输出为增强 `for`，而 Jarde 对全部探针方法保留可执行计数 `for`。编译与运行记录保存在本机临时目录 `/tmp/jarde-array-wrapped-binding/`；冻结的三方完整 class 源与关键 stdout 已复制到本证据目录。临时 Cargo target 未创建；不需要额外构建。

实际构建/执行命令如下（`ROOT` 是仓库根目录）：

```sh
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-24/array-wrapped-binding"
TMP=/tmp/jarde-array-wrapped-binding
mkdir -p "$TMP/original" "$TMP/jarde" "$TMP/classes-jadx4" "$TMP/classes-jarde4"
javac --release 8 -g:none -Xlint:-options -d "$TMP/original" "$EVIDENCE/ArrayWrappedBinding.java"
javac --release 8 -g:none -Xlint:-options -cp "$TMP/original" -d "$TMP/original" "$EVIDENCE/ArrayWrappedBindingRunner.java"
java -Xverify:all -cp "$TMP/original" ArrayWrappedBindingRunner > "$EVIDENCE/original.execution.txt"
jadx --no-res --single-class ArrayWrappedBinding -d "$TMP/jadx3" "$TMP/original/ArrayWrappedBinding.class"
"/tmp/jarde-iterable-remaining-target/debug/jarde-cli" class-source --input "$TMP/original/ArrayWrappedBinding.class" --class ArrayWrappedBinding --policy single-class --release 8 --format text --output "$TMP/jarde/ArrayWrappedBinding.java"
javac --release 8 -g:none -Xlint:-options -d "$TMP/classes-jadx4" "$TMP/jadx3/sources/defpackage/ArrayWrappedBinding.java" "$EVIDENCE/ArrayWrappedBindingRunner.jadx.java"
java -Xverify:all -cp "$TMP/classes-jadx4" defpackage.ArrayWrappedBindingRunner > "$EVIDENCE/jadx.execution.txt"
javac --release 8 -g:none -Xlint:-options -d "$TMP/classes-jarde4" "$TMP/jarde/ArrayWrappedBinding.java" "$EVIDENCE/ArrayWrappedBindingRunner.java"
java -Xverify:all -cp "$TMP/classes-jarde4" ArrayWrappedBindingRunner > "$EVIDENCE/jarde.execution.txt"
```
