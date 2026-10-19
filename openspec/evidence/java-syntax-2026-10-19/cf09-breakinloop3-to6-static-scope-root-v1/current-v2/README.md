# TestBreakInLoop3–6：源码固定计划（v2）

冻结日期：2026-10-19。本目录保存固定 JADX 测试源码及其关联 Smali fixture 的逐字副本，`sha256.txt` 为副本摘要。盘点只做静态控制流与有限输入设计，没有运行 JADX、Jarde、编译器、测试或目标字节码；源码推导出的调用次数不视作实测。`v1-v2.diff` 记录对 v1 计划文档的修订；源文件副本沿用 v1 并逐字一致。

## 固定原件

- `TestBreakInLoop3.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop3.java`
- `TestBreakInLoop4.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop4.java`
- `TestBreakInLoop5.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop5.java`
- `TestBreakInLoop6.java` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestBreakInLoop6.java`
- `TestBreakInLoop6.smali` ← `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/smali/loops/TestBreakInLoop6.smali`

## 范围对比与有限 case

| 固定用例 | 控制流形状与静态可达性 | 最小有限 case 计划 | 观察项 / 验收边界 |
|---|---|---|---|
| 3 | `while (true)` 中 `try` 创建空路径输出流；首个异常置 `tryMkdir=false` 后 `continue`；后续异常根据 `File.exists()` 分支，落到 `break` 后循环外。`check()` 对空路径调用 `writeMore("")`，意图触发异常路径；首异常 continue 与二次异常退出均可能触发。 | 单次 `check()`，沿用空路径，不另造目录或文件。可观察断言由原件 `hasToString("12135678")` 提供。 | 返回为 void；预期不抛异常，异常在内部 catch 处理。stdout 无；`builder` 是内部状态，原件 `check()` 断言其内容。测试入口 `test43` 标有 `@NotYetImplemented` 且 `@Test` 被注释，所以不能把它当作当前有效正向样例。优先先确认 JADX 对该 fixture 的实际能力及测试状态，再决定后续价值；源码假说不等于运行证据。 |
| 4 | 普通有界 `for`，无 `try/catch`、无异常 handler。外层条件 `c != '.'` 包住内层 `a || b` 更新；该外层分支无论内层是否命中都会 `break`；`.` 会自然结束循环。方法不抛异常。结构测试要求反编译结果保留一个 `for` 且不出现 `while`，含 debug 与 no-debug 两种反编译输入。 | 原件四值全部保留：`.`, `a`, `b`, `c`。它们分别覆盖自然退出、内层两个 OR 操作数命中，以及内层未命中但外层条件仍触发 break。 | 返回依次为 `1.0`, `1024.0`, `1024.0`, `1.0`；stdout 无；无输入状态变更。原件 `check()` 是辅助方法，两个 `@Test` 只检查反编译结构，不调用它。它没有 Loop2 的 exceptional RetryLoopFacts 形状；只能作为普通 Java loop/break 候选，先跑现有 JADX 正向样例并待 baseline 后再判断 Jarde。 |
| 5 | `try/catch` 包住扫描与解析；有限 `for` 接受数字和点，遇到首个其他字符时可更新 multiplier 并 `break`；之后解析累计前缀并向上取整。空串会在解析处进入 catch。`m/M/g/G` 分支与无单位分支可达。 | 原件五值全部保留：`"1.2"`、`"12am"`、`"13m"`、`"1G4"`、`""`。分别覆盖自然扫描结束、小写无效字母后 break 且 multiplier 不变、m 单位、G 单位且后缀不再扫描、解析异常。 | 返回 `2`, `12`, `13*1024`, `1024*1024`, `-1`；stdout 无；输入字符串不可变，无外部状态变更。原件两个 `@Test` 只检查可反编译代码，不调用 `check()`。它是 Java loop 后解析并由 catch 汇合返回的独立结构候选；先作为 Java 正向候选做 baseline，不和 4 或 Loop2 合并成同一异常机制。 |
| 6 | Smali 输入包含外层数组循环；条件通过时进入内层数组循环；内层对象若与当前对象同一引用则 `goto` 外层尾部，若 `f` 返回等于当前 long 则继续内层，否则抛 `IllegalStateException`。注释明确目标是避免在已有 break edge 上误插 continue。`disableCompilation()` 表明此测试检查反编译输出。 | 由 `test` 使用固定 Smali fixture 做静态验收即可。动态行为若需要，须另行定义 `test.Test` 接口 stub 与输入，并覆盖外层条件、内层空数组、同一引用跳转、`f` 等值继续、`f` 异值抛异常；本 fixture 未提供接口定义或输入数据，因此不能声称路径已动态触发。 | 静态检查要求恰有一个 `break` 文本命中。计划中的动态观察为正常返回 long 或失败抛 `IllegalStateException`；stdout 无；方法源码只读数组并调用接口，输入数组/元素由外部 stub 管理。当前 Jarde class/JAR 输入边界与 Smali/Dex fixture 不同；除非明确落实输入转换或字节码构造，不能将其算作 Jarde Java 输入验收。Smali lowering 应单列输入边界后再决定是否实施。 |

## 优先顺序与边界

先推进 4、5 作为已有 Java 正向候选：沿固定原件执行基线，再据实际结果决定是否存在 Jarde 缺口和最小闭环。4 是无异常 handler 的普通有限 loop、嵌套条件、赋值与 break；5 是 loop 后解析和 catch 返回的另一形状。

3 虽含异常后重试，但其测试标为 `@NotYetImplemented` 且没有启用 `@Test`。先核实 JADX 当前对该 fixture 的实质能力与有效测试状态；不要仅凭源码把它列作新增机制候选。源码中的两段异常路径可达性是假设，需由运行观察确认。

6 是独立的 Smali/Dex 输入模态问题。当前 Jarde 输入边界需要明确的转换或字节码构造验收方案，不能以等价 Java 源码覆盖这条边界，也不纳入 4/5 Java baseline 批次。

本文件只冻结源码形状、有限 case 和执行优先级，不据源码宣布 Jarde 缺口，不建立 OpenSpec，不创建 collector，不更改产品、fixture 或共享账本。若后续审计发现架构债务，应另行拆分记录，不混入当前范围。
