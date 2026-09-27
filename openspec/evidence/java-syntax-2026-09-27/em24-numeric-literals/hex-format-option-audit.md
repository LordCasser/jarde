# EM-24：固定 JADX 十六进制整数格式测试

固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `arith/TestNumbersFormat.java` 有三个活动测试：AUTO、DECIMAL 和 `testHexFormat()`。最后一项显式设置 `IntegerFormat.HEXADECIMAL`，只用 `containsOne` 检查 byte/short/int/long 四个数组的字面量文本；它没有运行断言。JADX CLI 的 `--integer-format auto|decimal|hexadecimal` 是源码展示选项，`TypeGen`/`StringUtils` 按该选项格式化常量并为 byte/short/int 的十六进制负值补 cast。

在主线 `25ba6f0269312dce8083afacf2f8110a8675173a` 上，以保留原 `TestCls.test` 四次数组赋值的最小 Java 8 类作隔离对照。原源码 SHA-256 `84377829ea7cef723d41ddaf09ed7824d996317a69fd725bd65f770c0e4acd86`，原 class SHA-256 `237078dec134a4758ef45cf47cc699dada8a42d83d81a53013e8796d9a3b28a3`。原 class、固定 JADX 十六进制输出和 Jarde 默认输出的完整类都通过 `javac --release 8` 与 `java -Xverify:all`，观察到同一最后一次 `long[]` 字段赋值结果：`[J:[[0, -1, -10, -1, -9223372036854775808, 9223372036854775807]]`。JADX 写十六进制及显式 cast；Jarde 写十进制。Jarde CLI 输入 `--integer-format hexadecimal` 以 exit 2 拒绝未知选项。

Jarde 当前整数表达式在 `crates/jarde-java/src/emit.rs` 的 `ExprKind::Integer/Long` 分支直接用 `to_string()` 或十进制 `L`，CLI 的 `--format` 仅选 text/json 报告格式。这里没有发现语法、常量值或 Java 8 重编差距；未追平的是用户可选的**字面量显示基数**。若以后决定追平这个配置，最窄改动是在展示层加入整数 radix 选项并沿现有 emitter 传递，不应增设字节码恢复机制。这个选项差距不作为新的语法恢复 OpenSpec；EM-24 其它形态仍按清单扩验。
