# ShiftCore 的永久基本证据切片

该切片从 2026-09-22 `shifts/root-core` 的 719-byte `ShiftCore`（SHA-256 `db5a3053fb7b38ddf22cff1e9ec93048d41ab29e94d5809c0f1f611f8ac86bba`）挑出确定性语法边界，包含十个方法、11 个移位指令（`nested` 有两条）：long/int 三种方向、short/char 左操作数的一元整数提升、`<<` 与 `>>>` 同优先级左结合，以及左右输入都来自 helper 调用的目标表达式。没有 `& | ^ ~` 位运算、long 距离输入、`l2i`、分支、异常预期或字段写入。类之外的 helper 和 runner 都是源码；仅 `ShiftSlice.class` 是编译冻结的输入。

## 冻结结果

`source/ShiftSlice.java` 以 `javac --release 8 -g:none` 编译为 664-byte class，SHA-256 为 `1036af7d36484406373dc63259fb2bf3232cd5fcf289b22b488815eac90a438a`。`outputs/javap.txt` 记录完整方法 Code；可见 `ishl/ishr/iushr/lshl/lshr/lushr`，short 左操作数和 char 左操作数都从 JVM 的 int 形状加载并执行整数移位；返回 descriptor 为 int。嵌套方法按顺序发出 `ishl`、`iushr`。原 class 通过 `java -Xverify:all`，完整输出保存在 `outputs/original.txt`。

JADX 1.5.6 对该 class 的完整类输出保存在 `outputs/jadx.java.txt`。仅移除其自动生成的 `package defpackage;` 后，整个恢复类和源码 helper/runner 以 Java 8 编译并通过 `-Xverify:all`；`outputs/jadx-diff.txt` 为空，输出逐字一致。

可用的 Jarde CLI `/tmp/jarde-instanceof-replay/jarde-cli` SHA-256 为 `336fda92b93df11a33299cb015df7b925b4ade22970727a21ff09b4b32156b71`。完整类及报告在 `outputs/jarde.java.txt` 与 `outputs/jarde-report.txt`。CLI 返回 0 但源码中有 21 个 `@bytecode` 拒绝标记；完整 Java 8 编译失败（`outputs/jarde-javac.log`），所以没有 Jarde JVM 输出，也不声称语义对照通过。失败位置是十个 shift 方法以及它们依赖的返回值消费；这是当前冻结 CLI 的拒绝事实。

Root 又从 `shifts/root-core` 的源码独立重编旧 719 B class，SHA-256 仍为 `db5a3053fb7b38ddf22cff1e9ec93048d41ab29e94d5809c0f1f611f8ac86bba`；原 class 的 `-Xverify:all` 723 行输出 SHA-256 为 `f34b05eb6a7ec009b2a6ce621f0479d5a9eb56e4c40ebdf350572408213a317d`。JADX 完整类重编、执行仍是 723/723 行相同；当前 CLI SHA-256 `336fda92b93df11a33299cb015df7b925b4ade22970727a21ff09b4b32156b71` 输出 29 个 `@bytecode`，完整类 javac exit 1。此次重放的计数与哈希固定在 [`root-census.json`](root-census.json)，旧 2026-09-22 baseline 的 CLI 哈希不同，但拒绝数量与结论相同。小切片及 root-core 都是实现前的 RED 基线，并不表示移位语法已恢复。

Root 独立以同一 `javac --release 8 -g:none` 命令重编三个源码文件，三份 class 的 SHA-256 与冻结文件分别逐字相同；`ShiftSliceRunner` 的 `-Xverify:all` 56 行与冻结 `original.txt` 相同。`javap` 中 11 条真实移位指令与十个源码方法对应，未把嵌套表达式的两次位移误计成一次。两组基线已就绪；实现、负例与完整恢复验收仍待后续任务。

## 重放命令

从仓库根目录执行：

```sh
javac --release 8 -g:none -d openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/source/ShiftSlice.java openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/source/ShiftSliceHelper.java openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/source/ShiftSliceRunner.java
shasum -a 256 openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class
javap -classpath openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original -c -p ShiftSlice
java -Xverify:all -cp openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original ShiftSliceRunner
jadx --no-res -d /tmp/shift-slice-jadx openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class
/tmp/jarde-instanceof-replay/jarde-cli class-source --input openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class --class ShiftSlice --policy single-class --release 8 --format text
```

JADX/Jarde 的编译对照均需额外提供冻结的 `source/ShiftSliceHelper.java` 和 `source/ShiftSliceRunner.java`。JADX 输出的 `package defpackage;` 需在编译副本中移除；Jarde 拒绝类预期编译失败。本目录保留相关生成输出与 javac 日志供核验。
