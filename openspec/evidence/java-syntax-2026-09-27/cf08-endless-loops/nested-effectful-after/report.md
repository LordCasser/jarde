# CF-08：外层 `if` 中的带效果双出口循环修后记录

输入仍是 [`nested-effectful-baseline`](../nested-effectful-baseline/report.md) 的 `NestedEffectful.pick`，class SHA-256 为 `2b29c9a9ba3812ed5083d8e983e907f9575dcb700dc52b2a7842e7fcc4667bee`。本次 CLI SHA-256 为 `99089291149c5a3e7ed02b1b2d9b279889f5321408215975ad6dc9ee891676ab`。固定 JADX revision 和算法文件哈希由同一 `replay.py` 校验，具体值见 [`summary.json`](summary.json)。

## Region 怎么闭合

外层 `If` 的非空臂入口 BCI 9 只到循环头 BCI 11。循环证书仍检查三个自然循环块、唯一 latch、BCI 17 的 `Push/Invoke/Store/Transfer` 效果臂、BCI 35 的直接 `break`、SSA φ 和 BCI 44 的两个正常前驱。嵌套证书额外检查外层分支、另一臂、父边界 BCI 50，以及 BCI 44 只经一段直线尾部到 BCI 50。Region walker 从 BCI 9 停在循环头时，只继续这份证书；随后在同一臂消费 BCI 44，并在父边界停止。

[`regions.json`](regions.json)记录外层 `If` 唯一持有 BCI 0/4/9/11/17/26/35/38/44，BCI 50 由后续 `Straight` 持有。Rust 测试检查了这些块、循环源码、两个 `break`、一次 `cost(7)`，以及 BCI 11/17/26/35/38/44/50 的源码映射。外层空值臂的无条件跳转 BCI 6 属于 BCI 4 基本块的 Region，但现有 source map 没有单独为这条结构性跳转生成文本段；这是来源层面的已知区别，不影响本次七个指定物理位置的映射。

## 运行与拒绝

冻结脚本以 `--require-jarde` 重编原始、固定 JADX、Jarde 的**完整类源码**，三者 `javac --release 8 -g:none` 均为退出码 0。`java -Xverify:all` 对空引用、空数组、命中、未命中依次均输出 `-1:0 / 8:1 / 3:0 / 8:1`；[`Jarde 源码`](source/jarde/cf08nested/NestedEffectful.java)的 `pick` 无 `@bytecode`。对应编译、运行及 Region 原始日志保存在本目录。

[`NestedEffectfulNegatives.java`](../../../../../tests/fixtures/cf08-nested-effectful/cf08nested/NestedEffectfulNegatives.java) 给出额外循环入口、异向出口、绕过尾段、内部汇合第三入边和异常边五个 Java 8 负例。它们的 class 用 `java -Xverify:all` 实际加载执行，日志见 [`negative-verifier.log`](negative-verifier.log)；定向 Rust 测试逐方法确认保留 `@bytecode`、无 `while (true)`，且物理循环头仍在 Region 记录中。预算耗尽和预先取消返回空源码、空 source map。未改 `build.rs` 的跨引用局部门。

`cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check`、`openspec validate recover-nested-effectful-loop-arm --strict` 均通过。固定 `TestNotIndexedLoop` 另跑原脚本，[记录](red-gap/summary.json)显示 Jarde 源码仍有 `@bytecode` 且 Java 8 编译因缺少返回语句失败；其构造和虚调用等差距未纳入这份证书。root 的独立验收仍需在合入前记录。
