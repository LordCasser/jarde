# CF-16：固定 JADX finally 正向测试三方审计

基线是 Jarde `ae822479`。审计对象是固定 JADX 集成测试 `TestTryCatchFinally.TestCls`，不是整个 `TestTryCatchFinally` 测试类。固定测试源码位于 [`TestTryCatchFinally.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally.java)，SHA-256 为 `22fda761dd9762341665e11491d169109efe56938dbbc2fb38104d0d32571b18`。JADX checkout HEAD 是 `2fb1b16386941660fda07e9017285aec40fcb37f`。最终重放使用 checkout 自带的 `/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`，其 `--version` 输出为 `dev`。最初探查曾调用 `/opt/homebrew/bin/jadx`（1.5.6）并将 class 暂存为 `TestCls.class`；该轮不作为固定三方输出证据。最终保存的 JADX 输出及 [`replay-baseline.sh`](replay-baseline.sh) 均基于 checkout 安装版和原二进制名 `TestTryCatchFinally$TestCls.class`。

## 固定测试要求

目标内层类包含 `public boolean f`、私有 `test(Object)`、私有静态 `exc(Object)` 和公开 `check()`。`test` 先执行 `this.f = false`，调用 `exc(obj)`，捕获并打印 `Exception`，最后在 finally 执行 `this.f = true` 并返回字段。`exc(null)` 抛出 `Exception("test")`，其他输入返回是否为 `String`。

测试 `test()` 要求默认恢复结果各含一次 `this.f = false;`、`exc(obj);`、`catch (Exception e)`、`e.printStackTrace();`、`finally`、`this.f = true;` 和 `return this.f;`，且不含 `boolean z`。测试 `testWithoutFinally()` 设置 `args.setExtractFinally(false)`，要求保留 `Exception` catch 与外层 `Throwable` catch，并要求 `this.f = true;` 出现三次。固定断言可在上面的原始测试源码中核对。

## 输入字节码与结果

证据 fixture [`TestTryCatchFinally.java`](fixture/TestTryCatchFinally.java) 保留目标内层类的方法体；[`JadxAssertions.java`](fixture/JadxAssertions.java) 是 AssertJ 风格的最小断言 stub，供独立 fixture 编译和运行。`check()` 仍按固定测试的两条表达式执行 `test("a")` 和 `test(null)`。[`RunnerReflect.java`](fixture/RunnerReflect.java) 另外逐条调用这两条路径并打印结果，随后调用 `check()`。这是带完整异常表的 class 文件，major version 52，SHA-256：

`24a23e9f96826d8a7ffe104ba31bfc0abb005141572920604f82e13b4b082807`

完整物理指令和异常表见 [`original.javap.txt`](outputs/original.javap.txt)。三行异常表依次为：

| ordinal | 保护范围 | handler | 类型 |
| --- | --- | --- | --- |
| 0 | `[5,10)` | 18 | `java/lang/Exception` |
| 1 | `[5,10)` | 31 | any / catch-all |
| 2 | `[18,23)` | 31 | any / catch-all |

三份清理都是同一实例字段赋值 `aload_0; iconst_1; putfield f:Z`，分别位于 BCI `[10,13)`、`[23,26)`、`[32,35)`。异常 handler 在 BCI 31 保存原异常，完成第三份赋值后于 BCI 37–38 重抛。正常和 catch 完成路径经过 goto 汇入 BCI 39 的共同 continuation，再读取 `f` 并于 BCI 43 返回。初始的 `f=false` 位于 BCI 0–2，具名 catch 正文是 BCI 18–22 的 `e.printStackTrace()`。

## 三方重编和执行

| 输入 | Java 8 重编 | `java -Xverify:all` | 正常输入 | `null` 异常输入 | `check()` |
| --- | --- | --- | --- | --- | --- |
| 原 fixture class | 通过 | 通过 | `true`, `f=true` | `true`, `f=true` | 通过 |
| 固定 JADX 默认完整类 | 通过 | 通过 | `true`, `f=true` | `true`, `f=true` | 通过 |
| 固定 JADX `--no-finally` 完整类 | 通过 | 通过 | `true`, `f=true` | `true`, `f=true` | 通过 |
| Jarde 完整类 | **失败：缺少返回语句** | 未运行 | 未运行 | 未运行 | 未运行 |

JADX 默认完整类保存在 [`jadx-default.java`](outputs/jadx-default.java)，对应固定测试的 finally 抽取断言；固定 checkout 安装版输出源码 SHA-256 为 `5970df78b543a3ac4f568d6b97aa28fea3e920100e2174abdfdfcb576954d944`。`--no-finally` 完整类在 [`jadx-no-finally.java`](outputs/jadx-no-finally.java)，结构上有嵌套的 `Exception` 和 `Throwable` catch，`this.f = true;` 恰好出现三次，源码 SHA-256 为 `44205967a3fff12c57cb653f1ac11b8c87a3c4e559b160050aa9902a42cb6141`。两种 JADX 输出的 runner 结果分别见 `jadx-default-run.*.txt` 和 `jadx-no-finally-run.*.txt`；原 class 的结果见 `original-run.*.txt`。执行异常路径时，`printStackTrace()` 的预期堆栈进入各自 stderr 文件。

Jarde 完整类源码在 [`jarde-complete.java`](outputs/jarde-complete.java)，SHA-256 为 `d13f0e676436c884b7e08d3afdfe14495953579645add683fbd444e421a95c1f`。`test(Object)` 被整体保留为 explanation-only；Java 8 重编在该方法末尾报“缺少返回语句”，日志见 [`jarde-javac.txt`](outputs/jarde-javac.txt)。这是不完整源码的编译拒绝，不能将它作为 Jarde 的行为结果。生成类源给出的首个拒绝位置是 BCI 31：该异常路径重复正常路径也执行的 finally 副本，但候选缺少完整的 straight-body、copy、range 和 ownership 证明。

## Verifier-valid 负例

[`NegativePatch.java`](fixture/NegativePatch.java) 使用本机 JDK 内部 ASM，只将第二份清理（catch 完成路径）的 `iconst_1` 改为 `iconst_0`，异常表、栈形和其他指令保持不变。负例 class SHA-256 为 `0d2eb1586657e42667491485bcb000528c96344b2eb57d75c573b31e555face1`。完整反汇编见 [`negative.javap.txt`](outputs/negative.javap.txt)。负例通过 `java -Xverify:all` 加载和执行：正常输入仍返回 `true, f=true`，异常输入返回 `false, f=false`；随后固定断言 `check()` 按预期以 `AssertionError: expected true` 失败，输出见 `negative-run.*.txt`。Jarde 对该类仍以 BCI 31 拒绝整个目标方法，见 [`jarde-negative-class-source.txt`](outputs/jarde-negative-class-source.txt)。因此，仅凭三处赋值的外形相同而合并会改变 catch 完成路径语义。

## 现有架构的边界

- [`guard.rs::finally_copy`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/guard.rs:1648) 识别 any handler 的 finally 候选；[`FinallyCopyProof`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/guard.rs:1716) 是单一异常行、单个正常清理、单个保存返回值和 handler 清理的证书。它不能单独闭合这里的具名 catch 加两条 catch-all 行。
- [`guard.rs::shared_cleanup_span`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/guard.rs:2048) 只抽取无参静态 void 调用或 static int 字段增量。当前清理从 BCI 10 的 `aload_0` 开始，是 `this.f=true` 的实例字段写，因而不满足该 span 形状。[`prove_shared_finally`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/guard.rs:2204) 还要求两份正常完成副本之后分别接 `load; return`；本例的两条正常完成边都通过 goto 汇到 BCI 39 后再统一返回。
- [`region.rs::shared_finally_body`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:3782) 与 [`bounded_shared_finally_body`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:3853) 已具备两个有界正文及共享 finally 的所有权框架，但以两份保存返回值确定 try/catch 正文边界，并各自要求保存点。当前目标没有这两处 saved return。
- [`build.rs` 的 SharedFinally 分支](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:12935) 已可生成带 catch 和 finally 的 `StmtKind::Try`，finally 写入复用现有语句 Builder；它仍消费上述带 saved-return 的 `SharedFinally` 证书。故 AST 与单句字段写入呈现可复用，行几何、共同 continuation、三副本效果相等及物理来源闭合需要一个更窄的证明入口。

这里的直接差距是固定结构的证书没有覆盖实例布尔字段赋值和“try/catch 均正常汇入同一 continuation”的终止形式。安全的正例边界应同时证明固定三行、三个字段身份和值相同、try/catch 两个有界正文、共同 continuation、catch-all 原异常重抛及三份 cleanup 的物理所有权，然后在一个原子 Builder 提交中只输出一次 finally 赋值。任何副本、行范围、handler 次序或 owner 有变化都应整体拒绝。该方向是证据推论，不代表已实现。

## 证据边界和重放

独立 fixture 将固定 `TestCls` 方法体按测试源码编译，并以 stub 提供 `assertThat(...).isTrue()`；没有运行 JADX 的 JUnit 集成测试类，也没有声称测试框架和测试依赖均已重建。重编器是本机 `javac 23.0.1 --release 8`，产物 classfile 为版本 52；这是 Java 8 classfile 验证，不是用 JDK 8 编译器验证。固定 JADX 输出来自 checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的安装版 CLI 对该 fixture class 的运行；该安装版标记版本为 `dev`。Jarde 的执行使用基线 `ae822479`。

[`replay-baseline.sh`](replay-baseline.sh) 检查固定 JADX checkout 和安装版标记，重建原 class、两种 JADX 完整输出、Jarde 完整类、执行 runner，并构造/验证负例。它是 `ae822479` 的修前基线门：当前 Jarde 完整类编译失败是预期结果；如果恢复实现后复用脚本，应将这项基线负向检查改为编译/执行成功的正向门。它将 Cargo target 放入 `/tmp` 临时目录，退出时执行 `cargo clean`；没有把 class、JAR 或 Cargo target 提交到证据目录。成功重放会打印临时输出路径，便于检查详细 stdout/stderr 和完整 CLI 报告。

root 于 2026-09-28 独立运行该脚本，退出码 0；原 class 与两份固定 JADX 输出的 SHA-256 分别仍为 `24a23e9f96826d8a7ffe104ba31bfc0abb005141572920604f82e13b4b082807`、`5970df78b543a3ac4f568d6b97aa28fea3e920100e2174abdfdfcb576954d944`、`44205967a3fff12c57cb653f1ac11b8c87a3c4e559b160050aa9902a42cb6141`。脚本确认 Jarde 修前编译失败和负例的 verifier/断言结果，并由 trap 清理 `696.6 MiB` 专用 Cargo target。
