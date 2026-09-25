# JADX 增强 `for` 识别边界

本地 JADX 源码 `2fb1b1638694` 的 `jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java` 在 Region 建好以后、局部声明处理以前尝试把 `while` 投影成计数 `for` 或增强 `for`。数组分支先要求两输入 phi 的归纳变量、0 初值、+1 更新，再要求索引只用于测试/读取/更新，条件是 `< array.length`，元素读取和长度引用同一数组值；成功后隐藏原始更新、比较和读取。迭代器分支要求 iterator 值恰好被 `hasNext()`、`next()` 使用，检查调用签名、使用位置及元素类型，然后隐藏原赋值与调用并输出 `for (element : iterable)`。这些 SSA 用途和“先证明、后隐藏”的顺序可移植到 Jarde 已有的事实、Region 与 AST 层，无须新建通用 pass 管线。

[探针源码](NonIterableCursor.java)给 `CursorBox` 一个返回 `Iterator<String>` 的 `iterator()`，但该类型不实现 `Iterable`；`sum` 是手写的 `while`。用 `javac 23.0.1 --release 8 -g:none` 编译，外层 class SHA-256 `1ecc437091af764a14522c3fad85a3b060724e5a41f9508190e61d692e8efbfd`，嵌套 class SHA-256 `8e3dd45266412f0885f9052ea76b407958ab45eb3500642d864ce3742457b2f8`。将两份 class 装成 jar 后用安装的 JADX 1.5.6 `--no-res` 反编译，`sum` 保持 `Iterator<String> it = cursorBox.iterator(); while (it.hasNext()) { ... }`，没有输出增强 `for`。因此只读 `iterator()` 短签名不足以预测 JADX 实际会误投影；Jarde 的未来规则仍应独立证明容器类型可用于增强 `for`。

现有[数组/Iterable 三方执行审计](../../java-syntax-2026-09-22/enhanced-for/README.md)已证明 Jarde 的数组下标 `while` 行为正确；Iterable 的 `hasNext()` 失败已由 2c.6 闭合。增强 `for` 是输出投影而非恢复原作者唯一语法：手写迭代循环可能有同一字节码。P0 的循环转移与局部作用域闭合后，独立 OpenSpec 应要求容器只求值一次、元素读取/转换、退出路径、异常和副作用顺序等价，并对不满足约束的 class 保持现有 `while`。

## Jarde 拒绝边界重放（OpenSpec 1.2）

在当前工作树使用独立 Cargo target `/tmp/jarde-enhanced-for-neg-target` 构建 `jarde-cli`：`cargo build -p jarde-cli --target-dir /tmp/jarde-enhanced-for-neg-target`，CLI SHA-256 为 `97f1deeab4d4aaca79b7018783791e35c98f367fdef50a839766c4a9a0acb0f8`。输入以 `javac 23.0.1 --release 8 -g:none` 编译；`Probe.class` SHA-256 为 `e91644639a67d65d23396aab404effae28c9dec5cd11a46c6224c1a3531a7761`，`NonIterableCursor.class` 与 `$CursorBox.class` 分别为 `1ecc437091af764a14522c3fad85a3b060724e5a41f9508190e61d692e8efbfd`、`8e3dd45266412f0885f9052ea76b407958ab45eb3500642d864ce3742457b2f8`。前者是冻结的合法数组反例；后者由本目录的 `NonIterableCursor.java` 编译。为取得可观察输出，另编译了仅调用 `NonIterableCursor.sum(new CursorBox("a", "bbb"))` 的 source-only runner（不属于待反编译类）。

对原 class 执行 `java -Xverify:all`，`Probe` 退出码为 0，输出 `cached=9`、`mismatch=ArrayIndexOutOfBoundsException`；cursor runner 退出码为 0，输出 `nonIterable=4`。这些确认了本次 Jarde 输入 class 的 Java 8 编译和原始运行基线。

用上述 CLI 分别运行 `class-source --input <class> --class <name> --policy single-class --release 8 --format text --output <file>`，两次命令退出码均为 0。Jarde 的 [Probe 完整类文本](Probe.jarde.java)对 `sumAndReturnIndex` 和 `mismatchedArrays` 都输出 explanation-only 回退注释与 BCI，没有增强 `for`；它也未恢复 `main`。用 `javac --release 8 -g:none` 编译该文本退出码 1，两个方法各报“缺少返回语句”，所以没有可供 `java -Xverify:all` 对照的 Jarde class。[编译器 stderr](Probe.jarde-javac.stderr.txt)保留了完整诊断。

Jarde 的 [NonIterableCursor 单类文本](NonIterableCursor.jarde.java)保留 `while (local2.hasNext())`，没有增强 `for`。第一次对这份单类文本裸跑 `javac` 退出码 1，报告找不到 `NonIterableCursor$CursorBox`；[该次 stderr](NonIterableCursor.jarde-javac.stderr.txt)仅说明编译命令缺依赖。root 复核后把同一输入的原始嵌套类放在 classpath：`javac --release 8 -g:none -cp /tmp/jarde-enhanced-for-neg-probe/classes -d /tmp/jarde-enhanced-for-neg-probe/jarde-classes-cursor-withcp /tmp/jarde-enhanced-for-neg-probe/jarde/NonIterableCursor.java` 退出 0，输出外类 SHA-256 `b3260d1f5de00346c603937c285772fbe799ced16a6df5bfaa7dcaf0c225d162`。`java -Xverify:all -cp /tmp/jarde-enhanced-for-neg-probe/jarde-classes-cursor-withcp:/tmp/jarde-enhanced-for-neg-probe/classes NonIterableCursorRunner` 输出 `nonIterable=4`，与原 class 相同。单类策略本来不承诺生成包含嵌套类的整个工程，不能把缺 classpath 的编译错误记为产品缺陷。

因此三个反例的形状拒绝条件均满足（没有误投影增强 `for`）；非 `Iterable` 探针在正确的同输入 classpath 下也已完整重编执行。两条数组手写循环仍为 explanation-only，输出的完整类缺返回语句，没有 Jarde 运行结果可声称等价。这是既有循环/局部生存期的 P0 恢复缺口，不改变本任务的拒绝边界，也不在本次证据任务中修生产代码。

两个手写数组方法的 `javap` 均在测试块 BCI 4–7 每轮执行 `arraylength`（BCI 6），而正面增强 `for` 样本先缓存数组长度。当前 `region::test_is_pure` 的值操作集合尚不含 `Operation::ArrayLength`，对应 `present-proved-java-structure` 未完成的 2b.6；说明性源码报出的“局部跨引用区域”是后续词法闭包拒绝，不应倒推为必须新建 foreach 机制。先由该 P0 任务恢复可执行的普通循环，再讨论这些负例的其它结构问题。

复现命令（在仓库根目录；先构建到独立 target）：

```sh
cargo build -p jarde-cli --target-dir /tmp/jarde-enhanced-for-neg-target
CLI=/tmp/jarde-enhanced-for-neg-target/debug/jarde-cli
"$CLI" class-source --input /tmp/jarde-enhanced-for-neg-probe/classes/Probe.class --class Probe --policy single-class --release 8 --format text --output /tmp/jarde-enhanced-for-neg-probe/jarde/Probe.java
"$CLI" class-source --input /tmp/jarde-enhanced-for-neg-probe/classes/NonIterableCursor.class --class NonIterableCursor --policy single-class --release 8 --format text --output /tmp/jarde-enhanced-for-neg-probe/jarde/NonIterableCursor.java
javac --release 8 -g:none -d /tmp/jarde-enhanced-for-neg-probe/jarde-classes-probe /tmp/jarde-enhanced-for-neg-probe/jarde/Probe.java
javac --release 8 -g:none -d /tmp/jarde-enhanced-for-neg-probe/jarde-classes-cursor /tmp/jarde-enhanced-for-neg-probe/jarde/NonIterableCursor.java
javac --release 8 -g:none -cp /tmp/jarde-enhanced-for-neg-probe/classes -d /tmp/jarde-enhanced-for-neg-probe/jarde-classes-cursor-withcp /tmp/jarde-enhanced-for-neg-probe/jarde/NonIterableCursor.java
java -Xverify:all -cp /tmp/jarde-enhanced-for-neg-probe/jarde-classes-cursor-withcp:/tmp/jarde-enhanced-for-neg-probe/classes NonIterableCursorRunner
```
