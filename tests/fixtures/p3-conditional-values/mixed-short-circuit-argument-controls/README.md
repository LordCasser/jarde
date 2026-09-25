# 混合短路调用实参拒绝控制

`javac --release 8 -g:none MixedArgumentControls.java` 生成冻结 class，SHA-256 为 `23f441cdf7dd016f8a8b20da7046a6786262ef1fc93d8b1afeb53d782b3a2d3b`。`many(Z)V` 在同一三测试图后向 `(ZI)V` 传额外参数；`instance(Z)V` 在图前读取接收者，并以 `invokevirtual (Z)V` 消费结果。两者都是有效 Java 8 产物，均须保持整图引用及完整来源。
