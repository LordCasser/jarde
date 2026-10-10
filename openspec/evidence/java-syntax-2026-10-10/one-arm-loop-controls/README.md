# One-armed loop controls

本目录准备一组单个完整 Java 类对照，目标是把四种 `if` 与 `while` 结构放在相同的源码、编译和运行边界内比较：`prefixWhile`、`noPrefix`、`loopAndTail`、`takenArm`。每种方法均通过 `e=true/false` 和 `n=0/1/4` 调用；Runner 的输出由每个 JDK 上重新编译的原始类作为运行 oracle，不在 collector 中写死 stdout。

输入在 `inputs-prepared-luna-v1/`：`PlainOneArmLoops.java` SHA-256 `8f5ccc668017113caf93002d3e07853206412ca581a23dfb98950ad7393805b4`，`Runner.java` SHA-256 `240b28968c2a0f466660e2b191b6c08cc801e7024362c0dccbe2f47427bc2f9b`。

`prepare-baseline-luna-v1.py` 准备 2 个原始腿、4 个 JADX 腿和 4 个 Jarde default/all 腿。每腿都在空 classpath/sourcepath 下以 Java 8 source/target 完整编译；只有编译成功才用 `-Xverify:all` 运行。JADX 输入 JAR 仅含 javac23 的 `PlainOneArmLoops.class`，不含 Runner。Jarde 使用冻结的 multiply CLI v2 与 metadata v2，只记录其外部路径和 SHA，不把 CLI 二进制复制进新证据。

collector 从固定 JDK manifest 取工具路径并复核哈希，也固定 JADX 1.5.6 launcher。原始类的 `javap -p -c -s -v` 用于核对 0 字段、5 个方法的 flags/descriptors 和各方法全部 BCI。Jarde 两种 profile 会逐方法记录 `outcome`、quality、representation、content、fallback、执行状态、source-map span/origin 及 BCI 覆盖；`noPrefix` 与其他三个方法在记录中分开呈现。完整生成类的编译和运行也独立记录。候选文本编译失败属于 Jarde 的产品观察，不会被记成实现通过，也不会覆盖原始/JADX 对照腿的状态。每条命令的 argv、exit、raw stdout/stderr、生成源、class set、map 与同 JDK 原始 runtime 比较结果写入闭合 inventory。

本基线只准备了 source、Runner 和 collector，尚未运行 JDK、JADX、CLI 或任何编译/运行命令；不预判任何工具的恢复结果，也不代表实现通过。
