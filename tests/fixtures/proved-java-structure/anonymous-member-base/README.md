# 非静态成员基类限定接收者拒绝边界

`AnonymousMemberBase.java` 是手写的 Java 8 源码，使用 `outer.new Base(sideEffect()) { ... }` 创建继承非静态成员类的匿名类。正常路径区分限定外层实例求值、真实基类参数求值、基类构造和匿名方法调用；null 路径验证限定实例的空值检查早于参数求值。`Base` 的 class 文件构造器带有编译器注入的 `Outer` 参数，该参数不是源码 `Base(...)` 的普通实参。

`run.sh` 在临时目录以 `javac --release 8` 重编译、校验并执行，退出时由 Python `shutil.rmtree` 删除临时目录。冻结的 `.class` 只保存在本 fixture 目录；SHA-256、完整 `javap -c -p -s -v` 输出和 JADX 1.5.6 对比记录见对应 OpenSpec evidence 目录。
