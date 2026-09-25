# Lambda 多语句体内联正例

`LambdaBodyInline.java` 捕获 `build` 的稳定局部参数，并在 lambda 箭头块里依次记录开始、增加计数、记录结束，最后返回计算值。`build(capture())` 在创建期间只执行一次捕获表达式；断言确保这时 lambda 正文尚未执行。随后两次 SAM 调用各执行完整正文一次，输出包含精确事件顺序、计数与返回值。

`IntAction.java` 是独立顶层 SAM 接口。源码不依赖嵌套类型，也不访问其它类的私有成员。目录中的 `.class` 是 `javac --release 8 -g` 从这两份源码生成并冻结的唯一副本。

`run.sh` 会在临时目录重编译、以 `-Xverify:all` 运行，并在退出时清理临时产物。
