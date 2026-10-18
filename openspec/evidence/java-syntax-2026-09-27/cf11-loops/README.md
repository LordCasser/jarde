# CF-11 嵌套与顺序循环审计（2026-09-27）

Jarde 固定基线：`02f90beb742c6273966f2b117ce528a334f9d912`，分支 `codex/cf11-nested-sequential-loop-audit`。JADX 固定 checkout：`/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，工作区干净。

## 固定 JADX 断言和代码

下列 SHA-256 均从上述固定 checkout 逐文件核验。

| 文件 | SHA-256 | 断言范围 |
|---|---|---|
| `TestNestedLoops.java` | `4897fd18a15fce6e0abb40dc8afc6c7a3fa9f16e617f3c7e5e88072f6a37309e` | 只检查生成文本中 `for` 两处、`if`、加法、减法各一处；没有 `check()` 行为断言。 |
| `TestNestedLoops2.java` | `795ea73a0acfc5695f9e67c456746919f889d52026695f406d88f9693469b6db` | 只检查 `for` 和 `while` 各出现一次；没有 `check()`。 |
| `TestNestedLoops4.java` | `b84dbf9988e1343f6d26f06345a4e8ba8b17993b4dc33419c86d9e9194968cca` | 文本只检查 `break;`、`return 0;` 各一次；`TestCls.check()` 调用 `testFor` 并断言结果为 12，是本组明确的语义断言。IntegrationTest 有可用 source compiler 时先跑源类 check；只有识别到 check 并启用编译或 decompiled check 时才对反编译类执行运行检查。 |
| `TestSequentialLoops2.java` | `adb34f80f7d55cd3309ed7708f0fdb46768cf32e4004dbd13d4e278530306720` | 检查两处 `while`、`break`、`return c`、两个 `<= 127`；无 `check()`。`testNoDebug` 另测无调试信息时的文本形状。 |
| `LoopRegionMaker.java` | `a77bedda10bdb0d36f3979c27c1fc9471da726b36f34ae14d505f6f659c6b13e` | 循环 region / header / exit 构造。 |
| `RegionMaker.java` | `f93147242c63aca4f0f27739b9de23bf93c8f83bbd25b48d12478b98a4c29b77` | 循环入口遍历和 out-block 后续 region 遍历。 |

实现侧，`RegionMaker` 对带 `LOOP_START` 的 block 选择相应 LoopInfo 并交给 `LoopRegionMaker`；多循环信息时会选择以当前 block 为 start 的循环。`LoopRegionMaker` 在候选条件块中跳过 handler、非条件块和属于内层 loop 的条件；检查 header、嵌套 loop 的外层出口关系及可归并 exit。没有可识别 header 时退化到 endless-loop 结构。顺序循环由当前 loop 的 out-block 继续遍历，分别构成后续 region。以上是源码读到的约束，不代表所有 irreducible CFG 均已覆盖。

## 三方可执行切片

`input/LoopShapes.java` 含两层普通计数 `for` 的 `nested` 和两个顺序计数循环的 `sequential`，不含异常、多出口、label 或数据相关无限循环。输入 SHA-256：`893cda08bea7def616c68129bf34cbeb952ba25c09807066af2f0773c0eda2b3`。

原始 class、JADX 完整源码、Jarde 完整 class-source 输出分别按 Java 8 编译；三者以 `java -Xverify:all` 执行，输出逐字一致：

```
40
25
```

JADX 与 Jarde 均保留嵌套的两个循环。Jarde 把 `nested` 呈现为嵌套 `for`，把 `sequential` 呈现为两个 `while`；本闭环以行为对照为准，不要求循环头的源码拼写一致。输入、class、JADX 源码、Jarde 源码和三方运行输出均保存在本目录。

复现（在仓库根目录）：

```sh
javac --release 8 -g -Xlint:-options -d openspec/evidence/java-syntax-2026-09-27/cf11-loops/original openspec/evidence/java-syntax-2026-09-27/cf11-loops/input/LoopShapes.java
jadx -d openspec/evidence/java-syntax-2026-09-27/cf11-loops/jadx openspec/evidence/java-syntax-2026-09-27/cf11-loops/original/LoopShapes.class
cargo run -q -p jarde-cli -- class-source --input openspec/evidence/java-syntax-2026-09-27/cf11-loops/original/LoopShapes.class --class LoopShapes --policy single-class --release 8 --format text > openspec/evidence/java-syntax-2026-09-27/cf11-loops/jarde/LoopShapes.java
javac --release 8 -g -Xlint:-options -d openspec/evidence/java-syntax-2026-09-27/cf11-loops/jadx-classes openspec/evidence/java-syntax-2026-09-27/cf11-loops/jadx/sources/defpackage/LoopShapes.java
javac --release 8 -g -Xlint:-options -d openspec/evidence/java-syntax-2026-09-27/cf11-loops/jarde openspec/evidence/java-syntax-2026-09-27/cf11-loops/jarde/LoopShapes.java
java -Xverify:all -cp openspec/evidence/java-syntax-2026-09-27/cf11-loops/original LoopShapes
java -Xverify:all -cp openspec/evidence/java-syntax-2026-09-27/cf11-loops/jadx-classes defpackage.LoopShapes
java -Xverify:all -cp openspec/evidence/java-syntax-2026-09-27/cf11-loops/jarde LoopShapes
```

Jarde 现有相关回归：`CARGO_TARGET_DIR=/tmp/jarde-cf11-target CARGO_INCREMENTAL=0 cargo test --test p3_loop_test_values --test p3_loop_transfers`，`p3_loop_test_values` 4/4、`p3_loop_transfers` 5/5 通过。它们覆盖条件/转移边界，不等同于固定 JADX 测试覆盖。

审计结论仅限这个首片：三方完整源码重编、验证运行一致，未发现该窄嵌套/顺序形态的差距。JADX 四个 fixture 中三个主要是文本断言；不把它们文本相似本身当作 Jarde 通过证据。更复杂 break/continue、共享状态、不可约图和更深嵌套仍待独立验收。

主线验收：在已合入 CF-06 的 `1422af58` CLI 上重放同一原始 class，Jarde 完整源码 SHA-256 仍为 `1c449e92d3dc9a9069c41b8d9c92b2f6748c6b16b171b72ea6c701c90481de28`，与本目录归档完全一致；三份归档 class 的 `-Xverify:all` 输出均为 `40 / 25`。

## 固定四项当前实测账本（2026-10-12）

四项现在都已经真实编译并采集完整类三方结果，不能再将代表样本的40/25当作四个固定fixture通过。

| 固定用例 | 当前实际结果 | 下一边界 |
|---|---|---|
| TestNestedLoops4 | child-loop side-exit窄片11/11接受，产品b155877ee自身CI已接受，完整三方同原 | 其它复杂出口不因此全部关闭 |
| TestNestedLoops2 | outer for/inner while已恢复，产品02ecd63ff本地7/9、自身CI待；完整双JDK同原 | raw List泛型声明独立 |
| TestSequentialLoops2 | 原/JADX十行同原；Jarde两循环和clinit正确，尾部new条件值缺BCI88/94/95/96，完整类编译失败无runtime | 同IR已确认outside_readers跳过Phi operand/Sites无读者拒绝；后续Builder未实测，最小所有权方案待审 |
| TestNestedLoops | [当前29命令基线](../../java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/README.md)，原八case与JADX四腿/Jarde四腿完整类行为同原，来源完整 | Jarde仍两while/raw List，增强for/Signature呈现首拒尚待内部实测 |

四固定输入的loop恢复、泛型声明和构造器条件值分开记账；持续目标与整个CF11/71/612完成数不变。

2026-10-12：for-condition窄片9/9，产品02ecd63ff自身CI38154998619四jobs/52steps/双seed各360目标3461-0-97已独立验收，见[产品验收](../../../changes/recover-proved-for-condition-expressions/results/own-product-ci-clean-root-v1/README.md)。[TestNestedLoops实际入口诊断](../../java-syntax-2026-10-12/cf11-testnestedloops-same-ir-root-v2/README.md)确认outer Assign入口拒绝、inner后续闭包拒绝仍待精确定位；走Engine真实facts，源码恢复且clean。此片不是整个CF11完成。

## 2026-10-18 Seq2 当前产品复核

上表是10月12日历史结果。固定原始 `TestSequentialLoops2` 已按[完整基线](../../java-syntax-2026-10-18/cf11-sequential-loops2-baseline-root-v1/README.md)和[完整生成源矩阵](../../java-syntax-2026-10-18/cf11-sequential-loops2-generated-matrix-root-v2/README.md)由root独立接受：当前 ee013 产品8请求均3方法Structured/无fallback/全BCI，Jarde16完整编译、64运行512case同原；JADX2完整编译/8运行64case同原，另2个javac8外层框架major55边界无runtime。default原短表与configured128控制分开，未手修源码。此输入现有路径已覆盖，无需新机制；不增加CF11/71/612整体完成数。

下一固定 `TestNestedLoops` 已有历史行为窄片，仍需按当前真实输出核typed foreach/泛型呈现。当前源码和永久测试已支持iterator声明/初始化分离，旧BCI13/42拒绝不代表当前首拒。先采完整原始外层/child输入及当前API/Builder证据，再决定OpenSpec；不重做已交付的NL3/NL5/Seq1或凭空探索新机制。
