# EM-27 验收：保留 `String(char[])` 对象身份

基于 `origin/main` `33f15fde5847ff150bf6689fc990866afd08bc13`，仅闭合 Java 8 直接返回、准确 `java/lang/String.<init>([C)V`、唯一常量 `char[]` 初始化链。数组证书由 `report` 在字段计划之后、`init::sites` 之前调用一次 `ArrayInitializers::prove` 建立；`init` 只读查询同一证书，builder 接收其所有权并沿原有 `NewArray`/`New`/`Return` 路径输出。`init::sites` 不扣预算，数组、region、builder、发射仍共享同一预算与停止路径。固定 `javap` 的直接形态为 BCI 0 `new String`、3 `dup`、4–21 数组初始化、22 `String([C)V`、25 `areturn`；先存局部的控制形态为 BCI 0–17 数组初始化、18 `astore`、19–24 构造、27 `areturn`。

源码验收命令：

```sh
CARGO_TARGET_DIR=/tmp/jarde-em27-target cargo build -p jarde-cli
python3 openspec/evidence/java-syntax-2026-09-27/em27-string-concat/replay.py \
  --jarde /tmp/jarde-em27-target/debug/jarde-cli \
  --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx \
  --jadx-checkout /Users/lordcasser/workspace/testzone/jadx \
  --out openspec/evidence/java-syntax-2026-09-27/em27-string-concat/acceptance
```

`replay.py` 检查固定 JADX revision `2fb1b16386941660fda07e9017285aec40fcb37f` 及七份测试哈希。运行使用 `rustc/cargo 1.98.1`、`javac 23.0.1 --release 8 -g:none`、OpenJDK 23.0.1 `java -Xverify:all`；CLI SHA-256 为 `70b59c36303ab0e73ce8da9d568c4a8364a3c59ff750f24115cf6a8b79e62dcd`。完整命令结果、源码和哈希见 [acceptance/summary.json](acceptance/summary.json)。原/JADX/Jarde 三方全类重编和验证运行退出码均为 0。Jarde 与原 class 八行输出一致，末两行均为 `false`；固定 JADX 两处均为 `true`。Jarde 的直接方法输出 `return new java.lang.String(new char[]{'a', 'b', 'c'});`，先存局部仍输出显式构造。

定向夹具 [Probe.java](../../../../tests/fixtures/em27-inline-string/Probe.java) 由上述 `javac --release 8 -g:none` 编成 `Probe.class`（SHA-256 `630892b11ab219ebc8433d01275719145108c584535ff78a72a4a72bb3b8cc07`）。其结构化来源表核对见 [acceptance/probe-check.json](acceptance/probe-check.json)：直接方法覆盖外层分配 0、复制 3、数组分配 5、三处 `castore` 11/16/21、构造 22、返回 25，及各元素生产者。错描述符、错 owner、元素调用副作用和额外数组读取均为物理回退，报告保留 `jre_new_interleaved_effect` 与 BCI 0/3 标记。额外数组写入与第二消费者的局部数组控制形态仍分别保留独立数组语句和显式 `new String(local0)`。测试还覆盖父子区间或消费者错配、handler 边界、Java 7 profile、数组证明低 `ir_items` 与取消、输出低 `output_bytes` 与取消；停止时无部分文本或来源表。构造站点自身只读证书且不扣预算，所以没有构造站点内部预算停止点。

检查命令均成功：

```sh
CARGO_TARGET_DIR=/tmp/jarde-em27-target cargo test -p jarde-java --quiet
CARGO_TARGET_DIR=/tmp/jarde-em27-target cargo test -p jarde-java class_source --quiet
CARGO_TARGET_DIR=/tmp/jarde-em27-target cargo check --workspace
cargo fmt --all --check
openspec validate recover-proved-inline-string-char-array --strict
```

保留的边界：只证明常量 `char[]` 的单块直接返回；`byte[]`、charset、别名/突变内联、额外可观察效果、其它构造器及 J11 `invokedynamic` 不在本切片内。上述形态按原有物理回退或局部数组路径处理。
