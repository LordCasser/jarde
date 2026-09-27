# CF-20：整数位掩码谓词审计

日期：2026-09-27。此轮只提交审计证据和 inventory 状态，不改生产代码、不新增 OpenSpec change。JADX 固定 checkout `/Users/lordcasser/workspace/testzone/jadx`，revision `2fb1b16386941660fda07e9017285aec40fcb37f`；对应测试和实现文件的 SHA-256 见 `baseline/summary.json`。

## 固定测试与实现边界

`TestConditions17.test()` 的实际断言只有 `.containsOne(" & ")`。测试主体条件是 `(a & SOMETHING) != 0`，其中 `SOMETHING = 2`；`print` 是空方法，没有 `check()` 或运行期期望值。因此这是一个窄的输出文本正例：要求结果源码保留一次 `&`，但它自身不验证 mask 命中/未命中、负数边界、分支结果或求值次数。测试未带 `@NotYetImplemented`。

固定实现分工为：`IfNode` 保存比较的 `IfOp` 与两个比较操作数；`ConditionGen.addCompare` 用比较操作数与比较符生成谓词文本；整数位操作作为 `ArithNode` 由 `InsnGen.makeArith` 输出其 `ArithOp` 符号和两个操作数。就这个路径而言，比较关系与表达式内的整数 `&` 各自从 IR 取符号输出，没有将 bitwise-and 转写为布尔短路运算的步骤。该映射限于本项路径，不外推所有位运算 lowering。

## 独立 Java 8 三方回放

`input/BitmaskAudit.java` 在两个谓词中分别测试 `(observe(value) & 2) != 0` 与 `(observe(value) & 2) == 0`。`observe` 每次递增计数器，样例覆盖 0、mask 未命中 1、命中 2/3，以及 `Integer.MIN_VALUE` 与其加 2 的有符号边界。固定脚本 `replay.py` 校验 JADX revision 和四个相关源码哈希，编译原输入，运行 JADX/Jarde 完整类源码，并将三份源码分别用 `javac --release 8 -g:none` 编译后以 `java -Xverify:all` 执行。

三方得到相同六行输出：

```text
0=22:1,33:1
1=22:1,33:1
2=11:1,44:1
3=11:1,44:1
-2147483648=22:1,33:1
-2147483646=11:1,44:1
```

每行中的两个 `1` 分别表示两个谓词各自调用 `observe` 一次；零/非零 mask 路径结果与原输入一致。`baseline/javap.log` 给出字节码边界：`select(I)I` 在 BCI 1 调用 `observe`、BCI 5 执行 `iand`、BCI 6 `ifeq 12`；`selectZeroMask(I)I` 同样在 BCI 1 调用、BCI 5 `iand`，再于 BCI 6 `ifne 12`。因此调用在分支前完成，计数输出验证每次评估一次；正负值结果同时符合整数掩码的 bit semantics。

JADX 和 Jarde 的源码分别留存在 `baseline/BitmaskAudit.jadx.java` 与 `baseline/BitmaskAudit.jarde.java`，重编 class、命令日志、运行结果和 SHA-256 清单也在 `baseline/`。Jarde class-source 文本本身声明它不保证可编译；本样本的完整类实际通过 Java 8 编译和 verifier，运行结果也与原/JADX 相同。

## 判定

CF-20 的首个独立可执行样例未发现整数 mask 谓词的语义差距：bitwise `&`、`!= 0`/`== 0` 两种比较、掩码边界和单次副作用均保留。固定 JADX 测试的断言强度仍只到输出文本；语义结论来自本次三方重编运行样例，不来自 `.containsOne`。这是一个已测首片而非全 lowering 覆盖；boolean `&`/`|` 归 CF-01，本报告不评价那些形态。
