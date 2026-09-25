# 顶层类型匿名捕获正例

`AnonymousTopLevel.java` 自写声明了顶层接口 `Renderer`、顶层抽象基类 `Base` 与公开入口类。目录里的四个 `.class` 是该源码经 `javac --release 8 -g` 编译后冻结的唯一副本。

`run.sh` 在临时目录重编译源码，以 `-Xverify:all` 运行，并在退出时清理临时目录。事件和计数输出验证局部捕获、`choose()`、`Base(long)` 与匿名体调用各执行一次且顺序明确。
