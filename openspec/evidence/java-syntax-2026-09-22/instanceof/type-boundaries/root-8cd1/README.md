# root 独立重放：instanceof 静态类型边界

`replay.py` 从上级目录复制三份自写 Java 8 源码，在独立临时目录编译原 class，并以冻结的 `/tmp/jarde-cli-boolean-root` 和 jadx 1.5.6 原样反编译整类；不编辑任何反编译正文。CLI SHA-256 为 `8cd1f767ba8cde9928bacedc29263d44be8e5e61b46d3b6f00456fac1c7c77cd`，原 `InstanceOfTypes.class` 是 1124 B、8 个 Code 方法、SHA-256 `8525cf1aabaef30c94e5387f4d29883bdcb7002da87fd742cdc40eaf59682f47`。

原 class 在 `-Xverify:all` 下运行成功，产生 11 行。jarde CLI exit 0，但六个 `instanceof` 消费方法被引用，完整输出 `javac --release 8` exit 1（六处缺返回）。JADX 整类 `javac` exit 1：四处丢失源码中 `(Object)` 加宽而出现不兼容的左操作数类型，另有一处把方法引用直接放在 `instanceof` 左侧。两种失败均未执行恢复源码，因此没有把未编译的文本计作运行 oracle。

`javap` 确认源码的四种加宽不生成 `checkcast`，真实指令只有 `instanceof`；这是实现须在表达式构建时恢复合法 Java 静态类型上下文的原因。命名方法引用还须由已有 lambda/factory 目标类型证明承接。上述现象与 `recover-instanceof-expressions` 中的既有设计一致；这次重放未发现需要额外机制的证据。
