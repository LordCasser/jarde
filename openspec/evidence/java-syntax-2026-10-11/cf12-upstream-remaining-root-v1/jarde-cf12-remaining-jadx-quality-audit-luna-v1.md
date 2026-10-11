# CF12 剩余两个 switch 用例的 JADX 质量审查

## 结论

这批 5 个 upstream JUnit 用例中，3 个通过，2 个失败，但失败性质不同：`TestSwitch4` 是已执行出的语义错误；`TestSwitchWithFallThroughCase2` 是 JADX 把可读的重复代码标成 warning，测试在代码质量检查处失败，现有证据没有证明其生成代码运行错误。Jarde 的留存输出对 `TestSwitch4` 保留了 `off++` 的后缀求值写法；对另一个方法则明确拒绝恢复。两份 Jarde 输出均是静态产物，本次没有编译或运行它们。

优先级应先处理 `TestSwitch4` 这类“表达式中有副作用时，读值与更新顺序被改写”的语义问题。重复代码 warning 属于另一个输出质量债务；现阶段 Jarde 的保守拒绝是安全结果，不需要为了消掉 warning 增加通用代码去重机制。

## 实际观察到的证据

`/private/tmp/jarde-cf12-remaining-direct-root-v1/upstream-junit.stdout.raw` 记录了 5 个测试、3 成功、2 失败：`TestSwitchSimple`、`TestSwitch2`、`TestSwitch3` 成功；`TestSwitch4` 的断言是 expected `1234`, actual `2234`；`TestSwitchWithFallThroughCase2` 因 `Code duplicated, block: B:16:0x007d` 的 method-problems 检查失败。后者的 stderr 栈停在 `TestUtils.checkCode` / `getClassNode`，没有进入生成类的运行时 check。前者的栈进入 `IntegrationTest.runDecompiledAutoCheck`，并由 `TestCls.check` 的断言观察到错误值。

`TestSwitch4` 的原始测试源码在 `/private/tmp/jarde-cf12-remaining-direct-root-v1/sources/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitch4.java`。`parse` 的 case 4 是 `num += (ch[off++] - '0') * 1000`，后续 case 3 再执行一次 `off++`。留存 JADX 源 `/private/tmp/jarde-cf12-remaining-direct-root-v1/capture/TestSwitch4.test/jadx-source/jadx/tests/integration/switches/TestSwitch4$TestCls.java` 却先执行 `off++`，再用 `ch[off]` 读千位；结果是同一个后移后的字符被用于千位和百位。以检查调用 `parse("a=1234", 2, 4)` 为例，原源码读取索引 2、3、4 的数字位并得到 1234；改写后的 case 4 从索引 3 读取 `2`，故实际结果 2234，与 JUnit 一致。

已保存的 class/Javap 证据也支持这一定位：class 在 `/private/tmp/jarde-cf12-remaining-direct-root-v1/capture/TestSwitch4.test/input/TestSwitch4$TestCls.class`，既有记录给出的 SHA-256 为 `9e8a852890b8b2cce5db5cc1c5086c2dd52ad66e27f499001e3f910439149526`。保存的 `javap.stdout.raw` 显示 case 4 的 `iload_1` 先提供 `caload` 的索引，随后 BCI 43 才执行 `iinc 1, 1`；这与 Java 后缀更新先取旧值的顺序相符。这里引用的是已保存的反汇编输出，没有重新运行 JDK 工具。

Jarde 对应静态输出 `/private/tmp/jarde-cf12-remaining-render-root-v1/TestSwitch4.test/TestSwitch4$TestCls/jarde-default.java` 保留了 `ch[off++]`，再在下一个 case 使用 `ch[off++]`，结构上与原 Java 一致。这证明当前输出没有复现 JADX 展示的错误；没有对该输出编译或执行，不能据此声称 Jarde 已通过运行时等价验证。

`TestSwitchWithFallThroughCase2` 原始源码在 `/private/tmp/jarde-cf12-remaining-direct-root-v1/sources/jadx-core/src/test/java/jadx/tests/integration/switches/TestSwitchWithFallThroughCase2.java`，captured JADX 源在 `/private/tmp/jarde-cf12-remaining-direct-root-v1/capture/TestSwitchWithFallThroughCase2.test/jadx-source/jadx/tests/integration/switches/TestSwitchWithFallThroughCase2$TestCls.java`。case 1 先追加 `>`；当 `(a == 5 && b)` 为真时追加 `1` 或 `!c` 并退出 switch；否则会自然进入 case 2，按 `b` 决定追加 `2`。直接进入 case 2 也执行同一个 `if (b) append("2")`。JADX 把 case 1 的自然落入路径改写成 `else if (b)`，同时保留 case 2 的独立副本，并在注释中报告重复块。按分支条件逐条比较，两处 `append("2")` 在输入路径上的作用相同；此结论是对源码控制流的静态推演，不是对该 decompiled class 的运行观察。

此用例的 captured class 在 `/private/tmp/jarde-cf12-remaining-direct-root-v1/capture/TestSwitchWithFallThroughCase2.test/input/TestSwitchWithFallThroughCase2$TestCls.class`，留存 SHA-256 为 `1b1e78da99c371f3292bdf672e599fb07737b2b9f4eca5233bad5fd724f66a3b`。Jarde `default` 与 `all` 输出都没有恢复 `test(IZZ)Ljava/lang/String;`，而是保留完整 bytecode BCI 列表并给出 `the arms of the branch in block 0 do not meet at one join`。这是显式 abstention，不能当作正确重建或运行时失败；`check()` 中的原始断言只被照录为调用表达式。

## JADX 源码定位：证据与推断

本地 JADX 源码 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/PrepareForCodeGen.java` 的 `modifyArith` 会把 `a = a + 2` 收缩成一元形式的 compound assignment；`InsnGen.java` 的 `makeArithOneArg` 会把值为 1 的加减输出为 `++`/`--`，否则输出 `+=`/`-=`。`ArithNode` 用 `ARITH_ONEARG` 表达这种形式。它们是输出表达式形态的相关层，但当前读取到的代码不足以证明本例的错误一定由 `modifyArith` 或 `InsnGen` 首次引入：错误也可能在更早的 SSA/寄存器重建或索引表达式形成时发生。能确定的是最终源码的语义错误，并且错误核心是“副作用更新发生在索引取值之前”。

`SwitchRegionMaker` 中的 case/fallthrough 组织逻辑负责切换标签顺序和 break/fallthrough 标记；`TestSwitch4` 的反例发生在 case 内的局部索引表达式，因此不能仅通过扩大 switch region 识别来修好。后续若处理 JADX 对照，应把副作用表达式顺序作为独立回归主题，追踪从 `iinc`/旧值使用到数组索引表达式打印的转换，不先假定单一责任函数。

## 对照 Jarde 现有证明范围

当前 Jarde `/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs` 的 switch 探针要求完整 canonical edge rows 与 normal-flow successors 一致，按每个 case entry 扫描无环闭包，并检查 case 所有权、incoming/outgoing 关系。其注释限定的 fallthrough 是某一 arm 的正常控制流能到达恰好一个后续 case entry；真实 case 标签按顺序相邻。对同时有 case-entry 出口和 switch join 出口的 arm，代码还要求 join 出口由可呈现的 `Transfer` 或条件比较控制；不能呈现时拒绝。两个 arm 声明拥有相同 CFG block 时也会拒绝。

这说明本例的“条件成立则离开 switch，否则落到后续 case”并非从现有探针描述中显然排除：它可能由条件比较分别导向 join 与下一 case，属于可以证明的候选形状。但本次只读的 `test(IZZ)` 整体产物显示 Walker 最终在 BCI 0 以 `ArmsDoNotMeet` 拒绝；没有临时 instrument 或单独调用证明器来观测其内部 fallthrough 结果。因此不能把该拒绝归因为 switch probe 不支持此形状，也不能承诺只需放宽某条 fallthrough 规则就能恢复。需先在后续独立任务中用受控内部观测定位拒绝阶段，再判断是否是外层 if 与 switch join 的组合边界。

因此，现有机制对这一方法的覆盖状态应写成“安全拒绝，整体方法未恢复；switch fallthrough 子形状是否已通过内部证明尚未观测”。不需要为了当前 gate 添加新的 region 变体、共享块去重或放宽所有权规则。若产品目标要求恢复此方法，应先证明每条路径的 `>`、可选 `1`/`!c`/`2`、switch 后 `+` 与末尾 `b && c` 后缀恰好一次，再单独设计最窄 join/arm 表达。

## 后续债务与优先级

1. **高优先级：副作用顺序正确性。** 以 `TestSwitch4` 为回归锚，审查读取旧 local 值、`iinc` 写入、数组索引消费、case fallthrough 的 SSA/表达式路径。要求动态校验或明确拒绝，不允许仅因输出语法可编译就接受。Jarde 当前静态输出看起来保留顺序，但应在专门的执行验证任务中确认。
2. **中低优先级：JADX 重复代码 warning。** 先确认项目是否把“无重复块 warning”作为目标。如果目标只是语义保真，当前 JADX 捕获尚不能证明该 warning 是行为错误；不要新增普遍重复代码合并机制。若目标是恢复该用例，先定位 Jarde 的 `ArmsDoNotMeet` 产生阶段，保留现有完整边和 ownership 拒绝条件，再针对这个窄形状验证。
3. **不并入当前 gate。** 两项分别属于 expression side-effect ordering 与跨 switch join 的结构覆盖/输出简洁性，不应混入当前变更，也不应以 `TestSwitchWithFallThroughCase2` 的 warning 代替语义失败证据。

本报告只基于已保存的 upstream 原始输出、源码/class 捕获、现有 Jarde 文本产物与本地源代码阅读；没有运行 JDK、javap、Cargo、JADX CLI、测试或 Git 命令。
