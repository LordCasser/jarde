# 窄数组写入的求值顺序端到端对照

这个夹具用真实 Java 8 `bastore`、`castore`、`sastore` 检验数组、下标和值的求值顺序。源类先以 `int[]` 编译；`patch_order_stores.py` 只调整三个写入方法的 descriptor、对应数组 helper 的 Methodref descriptor，以及每个方法唯一的 `iastore`。Effects helper 同时保留原 `int[]` 和 B/C/S 数组重载，因此补丁后的调用能解析到已有且可由 verifier 检查的方法。helper 与 runner 仍是普通 Java 源码。

使用 `javac 23.0.1 --release 8 -g:none` 编译的未补丁主类为 530 字节，SHA-256 `2c19ecb68cf39f83677b916053696635d35623cc262de75701f61b9830027966`；补丁后为 599 字节，SHA-256 `eaf27ba0badd9cf6649a829a6fccf17ad06081f6523395f0dff7179fc9d51470`。三个方法的 Code 均保持 29 字节，真实 store 位于 BCI 22，最终 descriptor 分别为 `([BIIZZZ)V`、`([CIIZZZ)V`、`([SIIZZZ)V`；数组 helper 的对应 descriptor 分别为 `([BZ)[B`、`([CZ)[C`、`([SZ)[S`。

runner 在写入前后各执行独立的 `P`/`Q` 标记。写入表达式按顺序调用数组、下标和值三个生产者，并记录每次调用、异常类型、生产者异常的固定消息，以及未完成写入时的原数组值。每种元素类型各有八项：成功、三个生产者分别抛错、null/越界时值生产者先抛错、null/越界写入本身抛错。写入 `-129` 后，byte 为 `127`、char 为 `65407`、short 为 `-129`。原 patched class 与 Jarde 恢复完整类均通过 `java -Xverify:all`；后者原样通过 `javac --release 8`。两边 24 行完全一致。null/越界的 JDK 错误消息不参与判定，因为措辞可随版本改变；异常类型、trace、调用次数与数组值都参与判定。

Rust 测试还要求三个目标方法均为无 `@bytecode` 的 Java/Structured 恢复，数组/下标/值生产者各只出现一次，BCI 7、13、19 与真实 store BCI 22 均在来源中，且完整类没有拒绝标记。重放命令：

```sh
CARGO_TARGET_DIR=/tmp/jarde-narrow-array-order-agent cargo test --test p3_narrow_array_store_order -- --ignored --nocapture
```

测试退出时删除临时 Java 类与源码。上述私有 Cargo target 已在代理运行后清理；root 又用另一私有 target 复跑该测试，并以独立临时目录执行一次 CLI/JVM 对照，24 行输出 SHA-256 `f6b57664f4b875a33e657275e21cbfd1cd581bd9d3122cd15cc47943347c6de5`。本证据只覆盖已证明的 B/C/S 写入和求值顺序；boolean[]、未知元素及其它数组生产者仍按各自边界处理。
