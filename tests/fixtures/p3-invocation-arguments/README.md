# p3-invocation-arguments

这是 `preserve-invocation-argument-types` 的独立 Java 8 fixture。源码覆盖调用点的
`Object`、`null`、数组、装箱、数值加宽、`byte`/`short` 常量、构造器、多参数、外部泛型
helper，以及方法引用的函数式目标。`GenericFactory.make` 保留源代码中的泛型声明；它
没有被预先擦除来迎合恢复文本。

永久语料只提交 `v8/InvocationArgumentsProbe.class`。`ArgumentTarget.java`、
`GenericFactory.java` 和 `InvocationArgumentsRunner.java` 是 JDK 对照使用的 source-only
文件，runner 在临时目录编译，不产生额外永久 class。方法引用使用命名的
`InvocationArgumentsProbe::returnFive`，因此不会依赖 lambda synthetic class 的名字。

编译环境与命令：

```text
javac 23.0.1
javac --release 8 -g:none -d /tmp/jarde-invocation-arguments-20260923/compile \
  ArgumentTarget.java GenericFactory.java InvocationArgumentsProbe.java InvocationArgumentsRunner.java
```

提交的 class 是上述命令生成的 `v8/InvocationArgumentsProbe.class`，SHA-256 为
`f4c364f1ecb388c64c4ae9140df820ad625a43e3b158426698865e4fe43bedde`。它包含 1 个 class
文件和 14 个带 Code 的方法（包括构造器与私有方法）。三方命令、原始 Java 输出和修前
debug CLI/jadx 差异见对应 change 的 `verification.md`。
