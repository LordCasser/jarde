# DT-25 Lambda helper 审计与修复回放

固定样例是三个顶级、无捕获 Java 8 lambda，目标类型依次为 `IntSupplier`、`IntUnaryOperator`、`IntBinaryOperator`，SAM 参数数为 0、1、2；方法体只含常量和直线整数运算。`Runner` 是独立 source-only 调用方。

固定 JADX revision：`2fb1b16386941660fda07e9017285aec40fcb37f`。参考 `TestLambdaStatic`（lambda body 语法与 helper 隐藏）及 `TestLambdaArgs`（1/2 个 SAM 参数）。`CustomLambdaCall` 解析实现 handle 后，对同类 synthetic 方法设置 `DONT_GENERATE` 并打开 inline；`InsnGen.makeInlinedLambdaMethod` 把 helper 指令写入 lambda body。

`replay.sh baseline|fixed` 显式接收 `JARDE_CLI`，使用固定 JADX HEAD；baseline 使用修前 CLI 并保留 javac 失败诊断，fixed 使用候选 CLI 严格要求成功。临时路径不写入冻结源码，`javap -p -c` 省略 verbose 时间戳，bookkeeping 的 elapsed time 归一化。生成内容先暂存再覆盖各自模式目录，不清理已有冻结文件；不以 CLI 二进制 hash 作正确性判断。每次回放冻结原 class 哈希、三方完整源码、Runner、结果和 SHA-256；fixed 连续两次 SHA 文件 hash 为 `d044cc077e06d1f360144eed240491ada441a8ef7758c5bd8a1a480506ce94ad`（source manifest）及 `91341839894f84ebd85acf47bc6cc8d62e0b4ef16a0cc1def2104a7b08a2daa0`（decompiled source manifest）。

修复前，原始源码和 JADX 完整源码均可由 `javac --release 8` 编译并经 `java -Xverify:all` 执行，输出 `7,15,42`。Jarde 原始 class-source 保留 `private static synthetic` helper（flags `0x100a`），lambda 调用 `LambdaFixture.lambda$...`，完整源码因 javac 合成符号冲突不能重编；对应原件与输出在 `baseline/`。

修复后，Jarde 在三个 lambda 体内投影常量、单参和双参整数表达式，省略源文件中的 helper 声明；同一 `ClassSourceReport.methods` 仍保留三个物理 helper 报告。Jarde 完整源码也通过 Java 8 重编和 `-Xverify:all`，三方逐行输出完全相同；`fixed/` 保存输出、bookkeeping 归一化结果与稳定哈希。

这是 DT-25 的一个已证差距及其窄首片修复，不声称整个 DT-25 单元完成。本次只覆盖 Java 8 javac、同类私有静态 synthetic helper、无捕获 `int` primitive SAM 的 0/1/2 参数，以及无效果直线表达式。helper 的准入不是 AST 节点数和指令数相等：类级 handoff 保留完整物理指令 BCI 集，投影证书比较它与 return 语句及递归表达式 AST anchor 的精确集合；同数量但额外物理指令的负例拒绝。分支体、异常/调用/不完整扫描、普通 invoke/handle、共享/嵌套 bootstrap、ConstantDynamic 反向闭包、取消与预算停止均有保留行为负例。捕获 lambda（DT-26）、方法引用（DT-27）、泛型签名、其它编译器与其它 SAM 类型均不触及。

Root 已另行验证捕获 `x -> x + base`、`() -> this.number() + delta` 仍会出现 helper 调用/完整源码编译冲突。它们是相邻 DT-26 独立差距，不属于该首片。
