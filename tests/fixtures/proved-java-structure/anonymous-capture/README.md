# 匿名类捕获映射 fixture

`AnonymousCaptureCases.java` 是自写的 Java 8 完整源文件。目录中冻结了 `javac --release 8 -g` 产生的全部 `.class`，便于工具使用同一输入。`run.sh` 在临时目录重编译原始源码、以 `-Xverify:all` 执行并在退出时清理临时目录。

输出中的 `events` 与 `counts` 固定检查捕获局部求值、`choose()`、`Base(long)` 和匿名体调用各发生一次且顺序正确。第二例用两个不同状态值区分被捕获的同型参数 `other` 与词法外层 `Outer.this`。
