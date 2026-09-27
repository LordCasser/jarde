# CF-08：外层 `if` 中的带效果双出口循环基线

这是已验收 [`EffectfulExits.pick`](../effectful-after/report.md) 的最小嵌套变体。`pick` 先处理空数组引用；非空臂内保留同样的整数循环、`cost(7)` 效果出口和命中 3 的直接出口，循环后执行一次 `result + calls`，再与空值臂汇合返回。输入位于 [`input/cf08nested/NestedEffectful.java`](input/cf08nested/NestedEffectful.java)，Runner 位于同目录。它没有 `TestNotIndexedLoop` 的内层长度分支、`File` 构造或虚调用，因而只验证“已验收的带效果双出口循环放进一个有边界的 `if` 臂”这一片。

固定输入由 `javac --release 8 -g:none` 产生的 class SHA-256 是 `2b29c9a9ba3812ed5083d8e983e907f9575dcb700dc52b2a7842e7fcc4667bee`。[`javap` 记录](baseline/javap.log)显示外层 BCI 0 的空值分支至 BCI 4 或 9；自然循环为 BCI 11/26/38；越界执行 BCI 17–23 的 `cost(7)`、写局部和退出；命中从 BCI 35 退出，两者在 BCI 44 汇合；BCI 44–49 只在非空臂执行 `result + calls`；BCI 50 是外层共同返回。固定 JADX 提交 `2fb1b16386941660fda07e9017285aec40fcb37f` 及 CF-08 测试和算法文件哈希由 [`replay.py`](replay.py)逐项核对。

[三方记录](baseline/summary.json)中，原 class 和固定 JADX 的**完整类源码**均重编成功，`java -Xverify:all` 对 `null`、空数组、命中和未命中输出逐行同为 `-1:0 / 8:1 / 3:0 / 8:1`。本次 Jarde CLI SHA-256 为 `38b0a725d81db18e01e62676718f82a7d978f13830477be83dd6df77015b1a20`。其 [`Region 证据`](baseline/regions.json)仅将 `[0,4,9]` 归给外层 `If`、`[50]` 归给返回，把 `[11,17,26,35,38,44]` 列为 `jre_region_uncovered_blocks`。完整 [Jarde 源码](baseline/source/jarde/cf08nested/NestedEffectful.java)随后因 `local 1 crosses a quoted fallback region` 只给 `pick` 留下 `@bytecode`；[`javac` 记录](baseline/jarde/javac.log)是缺少返回语句。这是实测拒绝，不能把 Jarde 计作已运行的第三方等价结果。

当前 `region.rs::effectful_dual_exit_loop` 在进入其已验收的 CFG/Code/SSA 证书前，直接排除带 `frame.boundary` 的候选。此样例的循环在外层 `If` 臂中，父边界是 BCI 50；其两条真实出口仍先在 BCI 44 汇合。下一个最小实现片应只允许这个有界臂形态，并以现有 `Region::Loop`、`LoopBreak`、`Frame::loop_body`、区域覆盖和 SSA 证明 BCI 44 尾部及 BCI 50 各自拥有一次。局部引用诊断发生在未覆盖 Region 之后，不是放宽 `build.rs` 局部安全检查的理由。正门槛是本样例三方完整 Java 8 源码均编译、验证运行四行相同、`pick` 无 `@bytecode` 且逐 BCI 有唯一 Region/来源。负门槛应包括额外循环入口、两出口不同目标、绕过 BCI 44 的路径、BCI 44 额外入边、异常边和预算/取消原子停止；这些负例尚未在本基线实施。

固定 [`TestNotIndexedLoop` 差距](../remaining-gap-architecture-2026-09-27.md)继续是红门槛。它除外层/内层 `if` 与 slot 2 的跨臂合流外，还含 BCI 25–35 的 `new File("h")` 构造，以及 BCI 38–55 的 `getName`/`equals` 调用；现有带效果循环证书只接受一条 `Push/Invoke/Store/Transfer` 效果臂且排除体内调用。本样例通过后不能宣称该固定类已恢复，应单独扩展和验收其物理形态。

复跑前构建当前 Jarde CLI 到独立 Cargo target；输出目录必须不存在或为空：

```sh
cargo build -p jarde-cli --target-dir /tmp/jarde-cf08-nested-target
python3 openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/nested-effectful-baseline/replay.py \
  --jarde /tmp/jarde-cf08-nested-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out /tmp/jarde-cf08-nested-replay
cargo clean --target-dir /tmp/jarde-cf08-nested-target
```

默认模式要求上述**当前拒绝基线**；实现后用新的空输出目录并加 `--require-jarde`，要求 Jarde 完整类无 `@bytecode` 且四行验证运行一致。原始编译、JADX、Jarde 生成源码、区域和 Java 8 编译/运行日志均保存在 [`baseline/`](baseline/)。
