# 同一局部槽生命周期：三方基线与保守边界

## 可重复入口

运行 `./openspec/changes/separate-reused-local-lifetimes/evidence/replay.sh`。脚本以 `javac --release 8 -g`、`-g:none` 编译两份现有 `ReuseAfterForEach`，编译本目录中的边界 fixture，使用 `java -Xverify:all` 执行 runner，并用 `javap -c -l -v` 保存/核对字节码结构。脚本在 `/tmp` 创建独立 `CARGO_TARGET_DIR`，构建当前 `jarde-cli`，临时建立外部只读 SSA dumper crate；退出时删除临时目录和 Cargo target。它还重放 frozen pre-fix Jarde 文本的编译失败，并验证当前 Jarde 与 JADX 完整类的重编运行结果。

本次工具版本：OpenJDK `javac/javap 23.0.1`、JADX `1.5.6`。Fixture 源码/runner SHA-256：`SlotReuseBoundaries.java` `4d4979707ab3a915024ef5bdd0d3b311da446b90fb87c3063950a4ecf9d0ca19`；runner `2772a9368b004f5cf4a28a072e06b1b0cb5e479c36443f9ab457337251cdd840`。`-g` / `-g:none` fixture class SHA-256 分别为 `f53bd7543241eebb0a9933e736396c711f62919c22b71e1a48c17c38f1bbf7b5` / `e2b174e41766a41525401d46c1738371d8f835ced88718997d5d4fe0de2c37cb`。`javap` 的完整 verifier、LVT、StackMapTable 和指令记录在 [有调试信息](logs/boundaries-g-javap.txt) 与[无调试信息](logs/boundaries-no-debug-javap.txt)；SSA dumper 源码为 [ssa_dump.rs](tools/ssa_dump.rs)。

## ReuseAfterForEach：已证的分段正例

输入复用既有 [`ReuseAfterForEach.java`](../../../evidence/java-syntax-2026-09-24/foreach-cache-declarations/ReuseAfterForEach.java) 和 runner，源码 SHA-256 `d89a811487925e7445d25261a92a70646906d8cb357d4c52607c08e099aeba31`。`-g` / `-g:none` 原 class SHA-256 为 `ad6ffab7c9fdb00cd733486d09414ab47421483ee6fa0f241bcba4ee8088cfd4` / `3fdc5853f0bb1e64071c1bf0b809d851e74fb41a576b0d273fe5085cd1fd548b`；相应 `javap` 在 [有调试信息](logs/reuse-g-javap.txt) 与[无调试信息](logs/reuse-no-debug-javap.txt)。两个原 class 都通过 `java -Xverify:all`，runner 均输出 `22`、`4`、`NullPointerException`。

有调试表 class 的 `sumThenReuse([I)I` 局部访问由 Jarde 实际 `SsaTable` 逐条核对，完整记录在 [reuse-g-ssa.txt](logs/reuse-g-ssa.txt)；`-g:none` 记录在 [reuse-no-debug-ssa.txt](logs/reuse-no-debug-ssa.txt)，值流和边相同。槽 2 的前段 `v9` 是 BCI 3 写出的 `[I` 引用，读取在 BCI 4（`arraylength`）和 BCI 16（`iaload` 的数组操作数）；槽 2 后段 `v29:Int` 在 BCI 36 写入、在 BCI 37 和 42 读取。两组 def-use 不相连。循环头 BCI 10 的槽 2 phi `v3` 输入为 `[v9, Itself]`，并已被 SSA 归约为 `v9`；因此前段**确实跨越循环回边**，但只是一个自引用平凡 phi。不能把它描述为“槽 2 没有 phi”，也不能靠 BCI 先后本身声称生命周期分离。

槽 3 前段 `v12:Int` 在 BCI 6 写入，唯一指令读取是 BCI 12；循环头 BCI 10 的 phi `v4` 同样是 `[v12, Itself]` 并归约到 `v12`。循环退出后，槽 3 在 BCI 40 写入后段 `v33:Int`，于 BCI 44 读取。它与前段没有 def-use 连接，但两者都是 `int`；本证据不要求把它们按类型拆开，`-g` 下第二段 LVT 名为 `second`。循环累加器槽 1 与归纳槽 4 则有非平凡 loop phi，不能连同槽 2/3 的平凡 phi 混为一谈。

canonical 正常边为 `B0→B10`、`B10→B16`、`B10→B33`、`B16→B10`；方法没有 exception table 或 exception edge。当前完整类恢复给出 region 顺序 `straight(B0) → loop(B10,B16) → straight(B33)`，见 [region-order.json](logs/reuse-region-order.json)。这个区域顺序与 SSA 的前段循环存活及退出块后段写入共同构成两生命周期分段证据。空/正常/null 输入的异常边界仍由原 runner 对照；null 在 `arraylength` 处抛出并向外传播。

### 三方重放状态

预修复基线 Jarde CLI SHA-256 `dd7c3089e4bf36d3408d5b395c57d945162f0bfefc4edec783d79a51feaa046d` 的完整文本被冻结在 [baseline Jarde -g](three-way/baseline-jarde/g/ReuseAfterForEach.java) 和[baseline Jarde -g:none](three-way/baseline-jarde/no-debug/ReuseAfterForEach.java)。两份都把槽 2 整体写成 `int[]`；`javac --release 8` 各在同一方法报三处数组/整数冲突，错误记录见 [g](logs/baseline-jarde-g-javac.stderr) 与[无调试信息](logs/baseline-jarde-g-none-javac.stderr)。这是本 change 的冻结失败基线。

共享工作树在取证期间已经出现 root 的分段实现，当前 CLI SHA-256 `e9e7eda6a8b7fde710aea172452def222aecf00ffe450e929bb8e91f8854c921` 的文本因此单独保存为[当前 Jarde -g](three-way/current-jarde/g/ReuseAfterForEach.java)和[当前 Jarde -g:none](three-way/current-jarde/no-debug/ReuseAfterForEach.java)。两份均用 Java 8 重编并 `-Xverify:all` 运行，和原 class 一样输出 `22 / 4 / NullPointerException`；运行记录在 [g](logs/current-jarde/g-runtime.stdout) 与[无调试信息](logs/current-jarde/no-debug-runtime.stdout)。当前输出中后继局部分别由有证名 `first` 和稳定合成名 `local2_2` 呈现。

JADX 的两份完整文本保存在[有调试信息](three-way/jadx/g/ReuseAfterForEach.java)与[无调试信息](three-way/jadx/no-debug/ReuseAfterForEach.java)。它们及同包 runner 均以 Java 8 重编、`-Xverify:all` 执行并得到相同三行输出。JADX 把输出放在 `defpackage`，复现时只为 runner 加同包声明。JADX 源码 SHA-256 为 `3d449a0c6a90ba7abbc09a6072669c9531d6b8fadaec5decdbfc2fa69e58c6e2` / `483f00a7f0a54934a390b19452e8582b6e8b5b00b4fd704b92676d5c348b2878`。无调试信息的 JADX 使用值传播把后段计算内联；这只能证明 Java 语义可恢复，不能证明 Jarde 应折叠或删除任何槽声明。

## 最小边界 fixture：类型、边和宽槽

Fixture 完整源码见 [SlotReuseBoundaries.java](fixtures/SlotReuseBoundaries.java)，runner 在[此处](fixtures/SlotReuseBoundariesRunner.java)。`-g` 与 `-g:none` 两份都通过 `javac --release 8`、`java -Xverify:all`，输出相同：`11 / 7 / 9 / 8 / 3 / 8 / 4294967300`。带 debug class 的实际 SSA 记录分别为 [sameType](logs/sameType-ssa.txt)、[exclusiveBranch](logs/exclusiveBranch-ssa.txt)、[loopBodyReuse](logs/loopBodyReuse-ssa.txt)、[handlerReuse](logs/handlerReuse-ssa.txt) 与 [category2Adjacent](logs/category2Adjacent-ssa.txt)。

- `sameType` 在槽 2 先后写 `int`（BCI 5，用于 7）与 `int`（BCI 13，用于 15），值链不相连且无 phi。LVT 给它们不同名字 `first/second`；`-g:none` 没有名字仍表达同类型赋值。它证明同类型连续写入无需为了 slot reuse 分段，LVT 名字也不是必须分段的证据。
- `exclusiveBranch` 在槽 2 的互斥分支分别写数组引用（BCI 12）和 `int`（BCI 22），各自在本分支读取（BCI 13/23），join BCI 25 没有槽 2 phi，join 后不再读该槽。两次写入的字节码顺序不代表同一执行路径上的先后生命周期。当前 Jarde 对 `-g` 与 `-g:none` 均能按分支 Region 恢复；隔离方法用 Java 8 重编、`-Xverify:all` 执行，结果 `7 / 9` 与原 fixture 相同。这里的证明来自控制流分支范围，不是把两个 BCI 划成连续生命周期，也不是 LVT 名 `local` 本身。
- `loopBodyReuse` 每轮在槽 2 写数组引用、读取后再写 `int`、读取，回边为 `B6→B2`。槽 2 在 loop header 为 Top，因此没有槽 2 phi；同轮 def-use 断开不代表全局线性顺序，下一轮会重复执行这两个写入形状。当前 Jarde `-g` 能用已有 LVT 分别恢复两段并通过隔离方法执行；`-g:none` 的矛盾门拒绝槽 2 两项读写，虽然剩下文本能编译，执行却输出 `0` 而非原结果 `8`，所以它不是成功恢复。不能把 BCI 大小当作回边证明。
- `handlerReuse` 槽 2 的数组写入 `v21` 与 handler 捕获写入 `v29:Throwable` 走不同边；异常边进入 block B31，正常/异常后续在 B38 汇合成槽 2 phi `v12=[v21,v29]`。虽都属引用类型且后续未读该 phi，本例明确展示 handler 与 phi 的连通性。当前 Jarde 的两种调试模式均报告 explanation-only，隔离方法因缺返回语句而不能编译；这是 protected-region 覆盖边界，不能伪称分段成功。`javap` 给出的 exception-table range 是 `[2,28) → 31`。
- `category2Adjacent` 在局部槽 0 先写/读 `int[]`，随后 `lstore_0 @19` 存放 `long`，它占槽 0 和 1；`tail:int` 位于槽 2，并在 `long` 仍活跃时读取。Slot 1 是 category-2 值的第二半，不能作为独立 source local。当前 Jarde `-g` 的隔离方法重编执行与原结果 `4294967300` 相同；`-g:none` 将 slot 0 的 `[I` 与 `long` 矛盾写入报告为 explanation-only，未生成有返回语句的方法。宽度与 debug 名称共同影响可用证据，单看 slot index 不足以证明段。

方法级编译与执行结果完整保存在脚本输出 [replay.stdout](logs/replay.stdout)。边界 class 整体含 explanation-only 方法，故该检查逐一抽出方法文本并配套 runner 验证，不声称整个边界类都能从 Jarde 输出重编。证据支持的准入范围是：`ReuseAfterForEach` 的 Ref→Int 跨越 loop 头的平凡自 phi，前段访问留在 loop Region、后段从 exit Region 开始，当前实现对 `-g` 与 `-g:none` 均通过完整类三方重编运行；分支正例由互斥分支各自完整的 SSA 值链、canonical CFG 不返回前段及后续 Region 放置共同支撑。无调试信息的 loop 回边复用、handler phi 和 category-2 宽槽若现有事实不能给出完整声明/用途证明，必须拒绝相关恢复范围；同类型连续赋值维持既有语义，无需为了槽复用强拆。

Java 9 TWR 夹具 `Held.use(Ljava/io/Reader;)I` 的 slot 2 SSA/异常边另记在 [Held-use-ssa.txt](logs/Held-use-ssa.txt)。`v13:Int`（BCI 6）与 `v18:Throwable`（BCI 17）各自只有同值的平凡 phi，彼此不合并；异常边将 handler 栈上的 Throwable 送进 handler，覆盖 slot 2 后再沿 suppression/throw 路径使用。root 已分别关闭分段推断和冲突门，确认两者都保持既有 explanation-only 输出；这是已有异常区债务，不将其误报成本 change 的回归。
