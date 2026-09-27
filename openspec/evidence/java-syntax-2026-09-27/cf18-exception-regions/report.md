# CF-18：异常区与 if/loop/嵌套 try 交叉审计

日期：2026-09-27。固定 JADX checkout 为 `/Users/lordcasser/workspace/testzone/jadx`，revision `2fb1b16386941660fda07e9017285aec40fcb37f`。本次只更新审计证据与 inventory，不改生产代码、不写实现 OpenSpec。六个固定测试、Smali 输入、相关算法和集成测试 harness 的精确 SHA-256 位于 `baseline/jadx-source-sha256.json`。

## 六个固定测试的断言强度

六个文件各有一个启用的 `@Test`；源码审阅未发现 `@Disabled` 或 `@NotYetImplemented`。我检查了测试与固定版本 `IntegrationTest`/`SmaliTest` harness，但没有启动 Gradle/JUnit 整组测试。`IntegrationTest` 默认编译反编译类，并在存在公开实例 `check()` 时同时运行该方法；Smali 测试默认走 dx，改为 Java 输入插件时会由 assumption 跳过。

| 测试 | 断言与运行证据 | 边界 |
|---|---|---|
| `trycatch/TestIfInTryCatch.java` | 只调用 `.code()`；没有显式源码结构断言或 `check()`。Harness 提供编译 smoke check。 | 编译成功不证明 if/try handler 的归属和运行行为。 |
| `trycatch/TestNestedTryCatch.java` | 断言有 `try {`、两次 `Thread.sleep`、两种 catch 文本且无 `return`。 | 这些片段不证明两层 try/catch 的嵌套关系或精确边界；无 `check()`。 |
| `trycatch/TestLoopInTryCatch.java` | Smali 输入，`oneOf` 接受三组 `containsLines(2, ...)` 形状：两种 while 摆放和一种 do-while。 | 没有源级运行 check；`oneOf` 允许三种控制结构，断言是固定文本形状。 |
| `trycatch/TestTryCatchInIf.java` | 断言恰有一处 `try` 和 `NumberFormatException` catch。 | 文本本身不保证 try 在 if 内。`TestCls.check()` 由 harness 运行，覆盖 null、十进制、hex、无效十进制和无效 hex 值。 |
| `trycatch/TestTryCatchNoMoveExc.java` | Smali 输入，断言恰有一处 `if (autoCloseable != null)`、`try` 和 `close()`。 | 文件中的 Java 注释伪码不是编译输入；无 `check()`，文本不验证 handler 运行行为。 |
| `loops/TestTryCatchInLoop.java` | 断言恰有一处 Exception catch 和 `break`。 | 文本本身不证明它们位于同一循环。`TestCls.check()` 由 harness 运行，并断言异常循环将计数 `c` 增至 3。 |

因此，六项里只有 `TestTryCatchInIf` 和 `loops/TestTryCatchInLoop` 有源类运行时语义检查；`TestNestedTryCatch` 的嵌套关系、两个 Smali 测试和 `.code()` 测试都没有这样的直接语义断言。

## 固定实现入口

`ExcHandlersRegionMaker.process`（34–42 行）收集 handler 区域；`collectHandlerRegions`（44–80 行）从 handler block / top splitter 和路径交叉点推导出口，再构造 handler region。`processExcHandler`（119–162 行）对普通 handler 以 handler block 为 dominator、对 finally 以 splitter 为 dominator；交叉出口不适用时回到 dominator frontier，并过滤 loop 内不可达出口。

`ProcessTryCatchRegions` 在 27–39 行遍历含异常 handler 的 region tree，41–64 行优先定位外层 try 与 top splitter，并在 69–118 行沿 dominator 可达路径抽取 try 内容、跳过可从 handler 到达的后续共同块，再构建 `TryCatchRegion`。`PostProcessRegions` 的 20–35 行负责 loop precondition merge、switch break 和普通 region 的 edge instruction 插入。固定 SHA 见 `baseline/jadx-source-sha256.json`；测试运行/自动 check 的 harness 路径为 `IntegrationTest.java:91–105, 282–315, 429–438`，Smali assumption 为 `SmaliTest.java:30–36`。

## Java 8 三方完整类回放

`input/ExceptionRegionsAudit.java` 用计数器验证循环迭代、if continue、内层 `NumberFormatException` handler、外层 `IllegalStateException` handler 和 handler 之后的正常路径。输入值依次由 `-1` 到 `3`：`-1` 走 if continue；`0` 在内层 throw/catch 后继续循环；`1` 命中普通路径；`2` 触发外层 catch；`3` 验证 catch 后循环还能继续。原类输出 `124:115`，表示返回累计值 124 且受观察副作用共 115。详细源码、完整类来源、编译日志、Java 输出和 SHA-256 在 `baseline/`。

原 class、JADX 完整类、Jarde class-source 完整类均尝试以 `javac --release 8 -g:none` 编译，并对成功生成的类调用 `java -Xverify:all`：

| 产物 | 编译 / 运行 | 观察结果 |
|---|---|---|
| 原始 class | 编译和 verifier 通过 | `124:115`，exit 0。 |
| JADX | 编译和 verifier 通过，运行 exit 1 | `work(2)` 抛出 `IllegalStateException` 后逃逸。JADX 将对应 catch 输出在 `if (value < 0)` 分支里，而非覆盖 throw 所在的 `work(value)` 区域。 |
| Jarde | 完整类编译失败，未运行 | `run()I` 正文被解释性 comment 代替，javac 报缺少返回语句。方法报告首个拒绝码为 `jre_region_irreducible`。 |

### 原始 BCI 与 Jarde 拒绝点

`run()I` 的循环 header 条件在 BCI 8–10；if 分支在 BCI 13，负值副作用在 BCI 17–22，并于 BCI 25 跳至迭代器增量；`work(value)` 调用位于 BCI 28–30。NumberFormat handler 从 BCI 38 开始：副作用在 BCI 39–45、continue 判定在 BCI 48–52，正常 catch 尾部至 BCI 55。普通路径的 `value == 1` 在 BCI 58–63，累加 5 在 BCI 66，跳到迭代器增量 BCI 85；外层 IllegalState handler 位于 BCI 72–82，同样汇入 BCI 85。迭代器增量在 BCI 85，回到 BCI 8 的循环头；退出检查在 BCI 10，最终返回位于 BCI 91–92。

原 class 的异常表精确为：

```text
from  to  target  handler
28    35  38      NumberFormatException
13    25  72      IllegalStateException
28    52  72      IllegalStateException
55    69  72      IllegalStateException
```

Jarde 的 `run()I` 首个 fallback 记录在区域 BCI 0，诊断为 `jre_region_irreducible`，点名 8 个块 `[8, 13, 17, 28, 85, 58, 63, 66]`；这些块跨越循环头、if/调用区域、continue 汇流与迭代尾。region detail 将该 whole-method region 标为 `structured=false`。Jarde 生成源码在 `run()I` 留下该诊断，没有输出其任何语句；`javac` 在方法闭合处以 missing return 拒绝。随后出现的字段未输出警告是此方法没有成功发布正文后的伴随诊断，不是首个拒绝点。

JADX 输出的 `catch (IllegalStateException)` 被移到负值分支中，因此不能处理 BCI 30 调用抛出的异常；运行在输入 2 处直接失败。这是实际 handler 所有权差距，不是从文本形状推断。Jarde 没有发布错误源码，但将这个可由 Java 表达的结构作为 irreducible 拒绝，属于保守拒绝型恢复缺口。

## 判定

CF-18 发现可重复的窄差距：嵌套异常 handler 与 if/continue、循环回边相交时，固定 JADX 将外层 handler 归到错误分支并改变异常传播；Jarde 在 `run()I` 的首个区域把结构化源程序拒绝为 irreducible，导致完整类不能编译。固定用例中的两个运行时 `check()` 仍提供各自更窄的正向证据，但不覆盖这组 handler 交叉边界。此项记录不推断所有 try/loop 或嵌套 try lowering 都有问题。
