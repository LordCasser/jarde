# Fixture and evidence record

本记录只覆盖任务 1.1/1.2 的 fixture 准备和已有证据重放，不勾选 `tasks.md`。主类和合法拒绝
边界分别记录，边界样本不计入 B/C/S 的 147 项核心 oracle。

## 1.1 核心正面 fixture

永久语料位于 `tests/fixtures/p3-narrow-array-stores/`。其中只有
`v8/NarrowArrayStores.class` 是永久 class；其余 Java 文件供生成/重放使用。README 固定了
Java 8 编译方式，`run_fixture.py` 在系统临时目录重编译源码、重做补丁、严格 JVM 验证原始和
patched class、运行三方源码阶段并在退出时清理输出。

复放命令：

```sh
python3 tests/fixtures/p3-narrow-array-stores/run_fixture.py \
  --jarde-cli /tmp/jarde-cli-deferred-interim-948a
```

原始类 SHA-256 为
`3a720dffd2907a2d027280a89568368d5e4e14f8b138fc37f3b0a6972aebdb80`，697 字节；patched 类为
Java 8 major 52、760 字节，SHA-256
`a3464f4b62da257e4cd4ff70970a05360475e51502a938c675392b334b359803`。10 个 Code 方法中，
6 个目标方法各只变更 descriptor 和唯一的 store opcode：直接 store BCI 3，producer store
BCI 7；Code 长度、max stack 与 max locals 均保持原值。其余三个 `ordinary*` 方法提供窄局部
回读、普通常量和同型写入对照。补丁报告和 Code 哈希由脚本逐次生成；永久 class 必须与重放
字节逐字节相等。

原始 source class 与 B/C/S patched class 均通过 `java -Xverify:all`，runner 各产生 147 行，
内容摘要分别为 SHA-256 `66aeef507937680f8a423293f3c4a9c76502dbb42bf726815ae34e0c031a1046`
和 `dbd5b1d08239b72ff59a50fad806a4ce55183d7a441a1f64e8a642dab0bb47fe`。两边输出不相等是
预期：source class 仍是 `int[]/iastore` 生成输入，patched class 才执行 B/C/S 窄化；patched
输出是目标语义 oracle，而不是原始 source class 的 `int[]` 数值。

使用 SHA-256 为
`948a6f9b7db009fbb89a792f83328c9ab63e42ac0cb62e691d97bec28c2c8ba0` 的 frozen Jarde CLI 重放：
完整 class-source 返回 0，输出 101 行、9 个 `@bytecode` 拒绝标记（BCI 3、7、5 各三次）；
完整源码 Java 8 编译、`-Xverify:all` 执行均返回 0，但运行的 147 行与 patched oracle 行行不同，
不能称为语义恢复通过。JADX 输出 SHA-256
`4882fe07e156aa2d5d4ceb1766ddbab81e71d3e9e583aa173bd4b40c91561961`；完整 JADX 源码 Java 8
编译返回 1，报告六处 int 到 byte/char/short 的窄化编译错误，未运行 JADX 结果。

中间实现 CLI SHA-256 `982ae786d64470eb339ebe6b839ff348e8c820e28e7a8da9ee9dc71f59187bf4`
早于最后一处 opcode/element 一致门；最终复核使用
`/tmp/jarde-narrow-array-target/debug/jarde-cli`，SHA-256
`cb0fc223553b7c82b023632fc67513412a48cbb8af31a3bc10d1fa561e41c315`。核心 147 项完整源码有
0 个 `@bytecode` 拒绝 marker，Java 8 javac 和 `-Xverify:all` 运行均通过；运行输出与 patched
JVM oracle 逐行相同，0 个差异，SHA-256
`dbd5b1d08239b72ff59a50fad806a4ce55183d7a441a1f64e8a642dab0bb47fe`。该输出在 final CLI 重放
中原样等于 patched oracle。

## 1.2 合法拒绝边界和求值顺序对照

三个 verifier-valid source-only 样本由 `run_fixture.py` 临时编译并在 `-Xverify:all` 下执行：

- `NarrowArrayStoreBooleanRunner`：核心类副本中仅将 `storeByte` descriptor 从 `([BII)V` 换成
  `([ZII)V`，仍为同一 `bastore` 和五字节 Code。17 个 int 值都按 JVM 低位语义得到 false/true。
  这确认数组已证明为 Z 的一般 int 操作数边界，不代表 B/Z 未知。
- `NarrowArrayStoreBooleanOperandRunner`：直接 boolean 操作数的 source store 将 descriptor 从
  `([ZIZ)V` 改为 `([BIZ)V`，`bastore` 和五字节 Code 不变；false/true 分别写 0/1。输入合法、
  但 Java 源码不支持 boolean 写入 byte[]，应保留来源完整的拒绝。
- `NarrowArrayStoreUnknownRunner`：`unknown()V` descriptor 不包含 B/Z；方法局部数组引用为
  verifier null type，Code 执行 `aconst_null; iconst_0; iconst_1; bastore`。生成的
  `NarrowArrayStoreUnknownElement.class` SHA-256 为
  `c575d237ba8e5a238075f12919fd11760d88db4a6bba48b3965fd1bacd05e1fc`。源码编译、
  `java -Xverify:all` 及 runner 均通过，原行为为 `NullPointerException`。这个样本才是恢复时
  不携带 B/Z 元素事实的候选，不计入 147 正例。

历史 frozen CLI 在上述 unknown class 上没有输出 `@bytecode` 拒绝来源，而将数组局部恢复成
`Object` 并生成 `local0[0] = 1`；生成源码 javac 返回 1。中间 CLI `982ae7…` 和最终 CLI
`cb0fc2…` 都在 unknown store BCI 5 输出拒绝 marker，完整源码 javac 返回 0。最终全证据报告的
source-map primary BCI 为 `[0, 1, 4, 5, 6]`，marker 指向实际 store@5。该 refusal 为 no-op：
source-only runner 在 `-Xverify:all` 下退出 1，因为生成方法正常返回而没有原类应有的
`NullPointerException`。这是来源可见的拒绝，不是行为等价恢复。

最终 CLI 对已证明为 Z 的一般 int 操作数和直接 boolean 到 B 的合法边界分别在 BCI 3 生成
拒绝 marker；两者完整源码 javac 返回 0，runner 都因拒绝 no-op 与原 JVM store 值不符而退出 1。
它们只验证拒绝路径，不计入核心 147 项执行对照。

数组、下标和值生产者 source-only control 有 8 项顺序断言：成功 `AIV` 各调用一次；数组异常
只到 `A`，下标异常到 `AI`；RHS 异常与 null/OOB 时都先到 `AIV`，且调用各一次、失败时原数组
值不变；producer 成功后 null/OOB 分别抛 `NullPointerException` 和
`ArrayIndexOutOfBoundsException`。这些证据固定 Java 8 求值顺序，但本目录的 control 用普通
`int[]` source 类运行，不能代替 B/C/S patched class 的恢复对照。后续独立的
`p3-narrow-array-store-order-e2e` fixture 已补齐该端到端验收，详见
[verification-order.md](verification-order.md) 和 [verification-root.md](verification-root.md)。

## 实际重放命令

本轮执行了：

```sh
python3 tests/fixtures/p3-narrow-array-stores/run_fixture.py
python3 tests/fixtures/p3-narrow-array-stores/run_fixture.py \
  --jarde-cli /tmp/jarde-cli-deferred-interim-948a
python3 tests/fixtures/p3-narrow-array-stores/run_fixture.py \
  --jarde-cli /tmp/jarde-narrow-array-target/debug/jarde-cli
CARGO_TARGET_DIR=/tmp/jarde-narrow-array-fixture-target \
  cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
```

`run_fixture.py` 的三次复放中带 CLI 的两次都运行完整 Jarde class-source/javac/java；JADX 在
当前环境可用时一并重放。最后一次使用上面记录的最终 CLI SHA。
manifest 生成器全量扫描了当前共享工作树并更新 `corpus-fingerprint.json`，因此产生的 dirty
记录包含当时工作树中的其它在途文件；全量结果已另存
`/tmp/jarde-narrow-array-corpus-fingerprint-regenerated-20260925.json`。这次没有编辑
`tests/p5_corpus_fingerprint.rs`，也没有执行 corpus-fingerprint verify。私有 Cargo target
`/tmp/jarde-narrow-array-fixture-target` 已删除。manifest 必须由主代理审查和收束。

## 任务状态

- 1.1：永久主 class、精确 descriptor/store patch、JVM 147 项、JADX 阶段和最终 Jarde
  零拒绝/逐行相同结果均已有可重放证据。
- 1.2：合法 Z、boolean 操作数与 unknown/null verifier 输入已构造。unknown 负例现有实现输出
  来源 marker 并保持可编译拒绝，但 no-op 与原异常不等价。真实 B/C/S 求值顺序由后续独立
  fixture 完成端到端对照；root 已验收。

本文件最初交付时未改 `tasks.md`；root 汇总全部证据后已将 8 项任务勾选完成。
