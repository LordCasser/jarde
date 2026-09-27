# DT-25 Lambda helper 审计

固定样例是三个顶级、无捕获 Java 8 lambda，目标类型依次为 `IntSupplier`、`IntUnaryOperator`、`IntBinaryOperator`，SAM 参数数为 0、1、2；方法体只含常量和直线整数运算。`Runner` 是独立 source-only 调用方。

固定 JADX revision：`2fb1b16386941660fda07e9017285aec40fcb37f`。参考 `TestLambdaStatic`（lambda body 语法与 helper 隐藏）及 `TestLambdaArgs`（1/2 个 SAM 参数）。`CustomLambdaCall` 解析实现 handle 后，对同类 synthetic 方法设置 `DONT_GENERATE` 并打开 inline；`InsnGen.makeInlinedLambdaMethod` 把 helper 的指令写进 lambda body。这为本片提供参考形态。

`replay.sh` 使用 `javac --release 8` 重编三方完整源文件，并用 `java -Xverify:all` 运行。原始源码和 JADX 输出均通过，结果都是 `7`、`15`、`42`。Jarde 完整类源码保留三个 `private static synthetic` helper（flags `0x100a`），并从三个 lambda 中调用 `LambdaFixture.lambda$...`。重编失败：javac 为 lambda 生成同签名 helper 时报告 `compiler-synthesized symbol conflicts`。这证明了当前窄差距：物理 synthetic helper 被当作普通源成员投影，导致完整源码不能重编。

这只是 DT-25 的一个已证差距，不表示整个单元完成。本证据限定于 Java 8 javac、同类无捕获静态 synthetic helper 和原始类型 SAM 的 0/1/2 参数。捕获 lambda、方法引用、泛型 SAM 适配、其它编译器、helper 多处引用、带 effect/控制流的方法体及全类引用清点不在本次实验范围。
