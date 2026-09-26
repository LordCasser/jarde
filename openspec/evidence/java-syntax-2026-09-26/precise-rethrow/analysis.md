# 精确重抛与 finally 候选边界

本页是证据索引。冻结输入和 runner 位于 `tests/fixtures/p3-precise-rethrow/`；证据目录只保留对照源码、必要命令输出和 `javap` 结果，不重复存放 class 文件。修前/修后 Jarde 来源报告的 stderr 保留完整，以便核对来源标记和拒绝原因。

这组 Java 8 样例记录一个候选误分类：`finally_copy` 在读取异常表类型前，仅凭 `astore; body; aload; athrow` 把具名 catch 当作 finally 复制。具名 handler `PreciseRethrowProbe.precise(I)I` 入口正是 `astore_1`，体内调用 `log(e)`，最后 `aload_1; athrow`。

## 固定输入

fixture 用 OpenJDK `javac 23.0.1` 的 `javac --release 8 -g -Xlint:-options` 编译。原 class 通过 `java -Xverify:all`。`PreciseRethrowProbe.class` SHA-256 为 `1290464fe261baec374a5de3a347dada062256ef0042853e91c330da4f9fc454`，有具名异常表行 `[0,33) -> 34, Class java/lang/Exception`。`javap -v -p` 输出在 `before/original/PreciseRethrowProbe.javap.txt`：handler 入栈保存位于 BCI 34，重抛 `athrow` 位于 BCI 40，`Exceptions` 属性为 `ParseException, IOException`。边界类 `PreciseRethrowBoundaryProbe.class` SHA-256 为 `2daac6f9eaa5c115d4cefb2246cc031e9b8985b88cd95d7d5f3fd75cb268603b`，其中 any 行 finally 为 `[0,17) -> 24, any`，多捕获有两个具名行指向同一个 handler。

原 class 正常路径、两类异常、finally 正常/抛出路径、多捕获两类异常以及替换异常的执行结果分别冻结在 `before/original-run.txt` 与 `before/original-controls-run.txt`。异常表与方法声明见 `before/original/*.javap.txt`。JADX 1.5.6 的源码位于 `before/jadx/{positive,controls}/sources/`；经 Java 8 重编译和 `-Xverify:all` 执行，两份轨迹均与原 class 完全一致。重编译 exit code 与运行输出保存在同级 `before/` 文件中。JADX 把 `precise` 与 `multiCatch` 的窄 `throws` 放宽为 `throws Exception`，偏离 class 文件声明。

## 修复前

冻结的修前 Jarde CLI SHA-256 是 `c020f727b931fe8c4d97af5e97ce231643d1ed042ba31f53ed3fbcf8028b5b9d`。正例输出在 `before/jarde/PreciseRethrowProbe.java`：两个 `athrow` 分支都以 `finally_copy` 原因引用，handler 也成为 normal-flow 外的未覆盖块；Java 8 javac 因 `precise` 缺少返回语句失败。any 行 finally 和多捕获 controls 也分别保留修前的引用边界。

## 修复后

当前 Jarde CLI SHA-256 是 `db053e0c153c58a25ea62bd04428987ac2ada2a218fd3085bfd8ba9af77641c8`。`after/jarde/PreciseRethrowProbe.java` 把具名行写为 `catch (java.lang.Exception e)`，其中顺序为 `log(e); throw e;`，并按 `Exceptions` 属性保留两个窄 `throws`。整类使用 `javac --release 8` 编译成功，重编译类以 `java -Xverify:all` 执行得到与原 class 完全一致的三行结果：正常返回 23；分别抛出 `ParseException("parse")` 和 `IOException("io")`，且每种异常只记录一次。

`after/jarde/PreciseRethrowBoundaryProbe.java` 显示边界行为：any 行 finally 不成为用户 catch；其受限形状仍保留 finally-copy 引用，并让异常路径保持部分呈现。仓库已有 finally 证明正例继续由 `guard::finally_copy_tests::implicit_cleanup_has_a_physical_copy_certificate` 覆盖。两行共享 handler 的多捕获进入普通 union catch 路径，不由本项单行精确重抛形状证明；`log(e)` 因现有参数引用类型呈现缺少安全转换而继续带引用标记，`throw e` 则照常恢复。非参数替换值仍输出 `throw new IllegalStateException("replacement");`。该 controls 源明确标为非完整恢复，不能作为可编译声明；any-finally 的输出仍不能整类重编译，这不是本次修复的目标。

两版源、必要命令结果、JADX 生成源码、`javap` 结果以及 javac/运行状态均保留在本目录 `before/` 与 `after/` 下。零字节 stdout/stderr 和由 fixture 可重建的 class 副本已删除。测试 `tests/p3_precise_rethrow.rs` 检查具名单行输出及入口存储、`athrow` 的物理来源，并确认 any 行和 controls 的保守边界。
