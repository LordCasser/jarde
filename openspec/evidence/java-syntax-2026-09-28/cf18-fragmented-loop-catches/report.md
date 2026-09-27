# CF-18 固定形态实现证据

本证据只针对两份冻结的 `run()I`。运行
`CARGO_TARGET_DIR=/tmp/jarde-cf18-target cargo test -p jarde-java --test p3_java_recovery cf18_same_run_facts --locked -- --ignored --nocapture`
可重建 `same-run-facts.txt`。该文件从每个方法**同一次** `analyze_method_ir` 结果输出 Code 异常行、Canonical 块/边/可抛点，以及 SSA 的入口/出口、phi、指令、定义、使用和物理来源。
`cf18_fixed_methods_preserve_rows_dispatch_and_presented_origins` 中的正常流与恢复断言也读取同一结果。

## CFG、异常行与值流事实

| 方法 | 按 ordinal 排列的物理异常行 | handler 的普通路径 | 更新点 |
| --- | --- | --- | --- |
| `HandlerLoopProbe` | `0 [9,16)→19 NFE`; `1 [9,29)→38 ISE`; `2 [32,35)→38 ISE` | `19→51`、`38→51` | `51 iinc local1`、`51→4` |
| `ExceptionRegionsAudit` | `0 [28,35)→38 NFE`; `1 [13,25)→72 ISE`; `2 [28,52)→72 ISE`; `3 [55,69)→72 ISE` | `38→52→85` 或 `38→55→58→63/66→85`；`72→85` | `85 iinc local1`、`85→8` |

`NFE`、`ISE` 分别代表物理表中的 `NumberFormatException`、`IllegalStateException` 类型。
两类 Canonical 可抛点的 handler ordinal 列表分别是 `11:[0,1], 20:[1], 26:[1]` 和
`17:[1], 22:[1], 30:[0,2], 39:[2], 45:[2]`。完整类第 3 行、缩小类第 2 行没有实际可抛点；
证书仍保留其 Canonical 被保护块和异常边。每条外层行的结束 BCI 都是唯一块末尾的 `goto`，
且该块只有一条通往本方法更新点的普通后继。内层行结束处则通往其正常汇合点（`32` 或 `58`）。

正常流的 `loop_entered_at().blocks()` 是自然回边的逆向闭包，不等于词法 owner 集合；其中可以含有异常根。
因此证明区分循环头支配的普通核心与异常独占根。完整类的 `38,52,55,72` 是异常独占段，
`58,63,66` 是普通路径与异常路径共享的续接段，归属一个循环 AST owner。异常独占段进入普通核心的边
只落在内层正常汇合点 `58` 或更新点 `85`；缩小类的两个 handler 都进入 `51`。两个 handler 根均无
普通前驱。其有限正常闭包的每条路径都终止于同一更新点；出现环或旁路则拒绝。
通过 JVM verifier 的 `other-loop-point`、`ordinary-handler-entry`、`handler-index-write` 分别检验三种违反情形。

SSA 的 Local(1) 是循环索引。在两类各自的循环头、两个 handler、其正常闭包的所有块和更新点入口，
Local(1) 经平凡 phi 替换后均归一为同一个循环携带值 `ValueId(1)`；更新点的 `iinc` 读取它并写出下一值。
新证书同时检查更新点之前各块入口和出口的该值身份。Local(0) 是累计值，沿普通与异常路径**可以改变**：
完整类的 `58` 汇合 `34` 与 `55` 的写入，`66` 再汇合 `58` 的值与 `63` 的可选写入，外层 handler
在 `82` 再次写入。因此不能要求累计值始终只有一个 SSA 身份。既有 statement builder 的 reaching-write
检查（含 `iinc` 同槽读写形态）证明其 Java 局部作用域；下文的完整类重编与运行验证生成的源码。
两个 catch 入口的首条 `astore` 均消费 caught Stack(0) 值；外层 handler 的多个输入由 SSA 栈 phi 汇合。
具体定义/使用 ID 与物理 BCI 来源见 `same-run-facts.txt`。

`replay/*/jarde/report.json` 中 `run()` 的 Region 记录各认领每个 Canonical 块一次：缩小类循环为
`[4,9,19,32,38,51]`，完整类循环为 `[8,13,17,28,38,52,55,58,63,66,72,85]`。
集成测试将认领并集与全部 Canonical 块比较，并拒绝重复认领；还检查上述 handler、被保护点、共享续接段与
更新点的 source-map 锚点，核每个主来源都指向精确输入 class digest 内的 `run()`。进一步将同轮
`SSA_STEP` 的**全部物理指令 BCI**与该 `run()` 的 source-map 主来源及派生来源作集合相等断言：
缩小类覆盖 31/31，完整类覆盖 47/47，没有漏锚或虚构锚点。证书证明的内层正常出口 `goto`
（分别为 BCI 16、35）归入对应 `Try` 的派生来源；更新点回环 `goto`（BCI 54、88）归入
对应 `Loop` 的派生来源。二者均取自受证的物理末端指令和唯一普通后继，未对其它 `goto` 扩展来源规则。

## 完整类重放与 verifier 有效的拒绝样本

`replay.py --cli /tmp/jarde-cf18-target/debug/jarde-cli --output replay` 生成 Jarde 完整类源码与报告，
复制冻结的原/JADX 完整源码，以 `javac --release 8 -g:none` 编译全部六份，再用 `java -Xverify:all`
分别运行。命令、退出码、标准输出及错误输出保存在 `replay/results.json`。

| 类 | 原源码 | 固定 JADX | 当前 Jarde |
| --- | --- | --- | --- |
| `HandlerLoopProbe` | `4:110`，退出码 0 | `4:110`，退出码 0 | `4:110`，退出码 0 |
| `ExceptionRegionsAudit` | `124:115`，退出码 0 | `IllegalStateException: two`，退出码 1 | `124:115`，退出码 0 |

两份 Jarde 完整类源码均没有 `@bytecode` 标记。固定 JADX 完整类将外层 catch 放进正值分支，导致 `two`
异常外抛，因此仅作负对照；原完整类是语义 oracle。

`NegativeFixtures.java` 使用 JDK ASM 确定性地修改冻结原类，在 `negatives/` 下保存八份 class。
每份都能通过 `java -Xverify:all` 加载（其中部分因刻意改变的运行行为而非零退出）。这些变体分别修改
外层 catch 类型、异常行优先级、遗漏保护、行间新增可抛点、handler 进入其它环内点、handler 新增普通入口、
handler 修改循环索引，以及交叉保护范围。`cf18_verifier_valid_near_misses_refuse_the_whole_method`
断言它们都不会得到部分 Java catch；CLI 对每份也均报告整方法回退。

审查补充：跨异常区局部变量含 `iinc` 时，声明规划现在只接受已证明的 Java `int` 类型，并核同一 BCI
的 SSA 只有该局部槽一读一写、两者归同一 reuse variable 和可呈现 Region 路径；后续 reaching-write
遍历仍须证明每次读取都只来自可呈现的写入。JVM verifier 将 `boolean` 也放在 int 形态的局部槽中，
因此仅凭 SSA 槽形不能把 `boolean + 1` 写成 Java。`boolean_iinc_cannot_justify_a_cross_catch_java_declaration`
负例锁住这一源码类型边界；上面的 verifier 有效 `handler-index-write` 则锁住异常路径上索引值身份的边界。
