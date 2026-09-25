# p3-unary-negation

这是 `recover-unary-negation` 的独立 Java 8 fixture。源码只使用自写的 `UnaryNegation` 类，覆盖四种 JVM 取负指令、byte/char/short 的一元数值提升、嵌套取负、复合运算分组、局部变量、调用参数、一次调用与抛异常，以及由 `i2b` 形成的无法呈现操作数。

编译环境与命令：

```text
javac 23.0.1
javac --release 8 -g:none -d v8 UnaryNegation.java
```

提交的 class 是上述命令生成的 `v8/UnaryNegation.class`。测试只运行这份自写 class；jadx 对照和 jarde 输出摘录见 change 根目录的 `verification.md`。
