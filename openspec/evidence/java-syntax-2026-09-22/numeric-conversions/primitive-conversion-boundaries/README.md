# Primitive conversion storage/effect boundary evidence

该独立探针补充永久 15-opcode fixture 未覆盖的内存边界：同型旧局部与转换后值共存、显式表达式分组、转换结果写入字段/数组、合法 boolean descriptor 的拒绝边界，以及转换值被丢弃或重复使用时的调用次数。`DiscardedConversion.drop(I)V` 是 javac 生成后插入 `iload_0; i2b; pop` 的 classfile；它与 `(Z)Z` boolean 样例都经 JVM 验证。

运行 `python3 run_audit.py [path/to/jarde-cli]` 可复现。脚本以 `javac --release 8 -g:none` 编译源码，对 `(Z)Z` 方法只插入 JVM verifier 允许的 `i2b` 指令并调整 Code 长度，再用 `java -Xverify:all` 执行原始输入。`javap`、冻结 class bytes、原始输出、JADX/Jarde源码、编译日志、退出状态、hash 与摘要一并保存在本目录。boolean 样例的 descriptor 保持合法 Java/JVM 类型 `(Z)Z`；其代码是 `iload_0; i2b; ireturn`，用于检查已知 boolean 来源是否被错误解释成数值。它不是由非法 Java cast 生成的样例。

只有重新编译成功且再次经 `-Xverify:all` 执行的输出才会参与逐字运行比较。无法重编译的 Jarde/JADX 输出只记录源码、引用数与编译错误，不称为等价。`summary.json` 的 `original_jarde_equal` / `original_jadx_equal` 在没有成功执行时为 `null`。

结果摘要显示：javac 原始探针成功执行；JADX 的主要探针可重编译执行但运行输出不同，弃值 class 可重编译且观察相同，boolean descriptor 输出不能通过 javac；当前 Jarde 的主要探针无法重编译，boolean descriptor 也无法重编译，弃值 class 则可重编译执行且输出相同。摘要将未成功运行的整体比较置为 `null`，没有将引用文本当成等价输出。`sha256sums.txt` 固定本目录证据文件 hash。

该探针不声称覆盖无显式转换的 byte/char/short 局部声明或 `ireturn` 隐式窄化；这两项属于单独的恢复债务。除 verifier 边界变体外，转换样例来自 Java 8 classfile。
