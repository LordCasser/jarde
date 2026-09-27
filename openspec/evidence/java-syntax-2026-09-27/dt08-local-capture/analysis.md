# DT-08：匿名接口捕获局部参数的隔离差距

固定 JADX `inner/TestAnonymousClass7.java` 直接断言 `final double d` 参数被匿名 `Runnable.run()` 引用，而不是以 synthetic 字段呈现在源码中；`TestAnonymousClass8.java` 的外层实例引用另与 DT-04 已验收子形态交叉。本切片只考察前者：顶级类的 `static Runnable create(final double d)` 直接返回唯一匿名体，Runner 分别捕获 `2.5` 与 `-0.0`，通过原始及重编源码运行来区分值传递。没有局部写回、异常、第二分配点或其它捕获。

`replay.py` 使用固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 与冻结 Jarde CLI（哈希见 `baseline/summary.json`）。原 class 和 JADX 完整源码均通过 `javac --release 8`、`java -Xverify:all`，逐行输出 `2.5`、`-0.0`。JADX 根类返回 `new Runnable() { ... println(d); }`。Jarde 根类报告 `anonymous_child_shape_unproved`，输出 `new p.Capture$1(arg0)`；其物理 child 源码保留 synthetic `final double val$d` 和构造器赋值。该赋值在 `super()` 前，故完整 Jarde 源码在 Java 8 下编译失败，不能称为语义运行一致。第二次独立重放的 summary 与冻结文件逐字一致，输入、生成源码、`javap`、编译诊断都在 `baseline/`。

这是两个相连但不同的边界。Java 8 class 的 child 构造器确实先写 synthetic 捕获字段，再调 `Object.<init>()`；JVM verifier 接受，Java 8 源码中的具名物理 child 却不能照此顺序书写。根类的匿名投影当前只识别无字段 child，或 `this$0:LOuter;` 的单字段外层实例捕获证书；`D` 类型参数捕获没有获证，因此退回无法重编的物理源码。不能把 `val$d` 的名字、`$1` 后缀或参数出现顺序本身当成捕获证明。

架构方向是在当前唯一分配点、准确 `EnclosingMethod`、整类 owner 使用普查之上，复用物理字段/构造器写入/SSA 读取链的已有捕获证明结构，将准确字段 descriptor `D`、构造参数 slot、根方法创建点实参和同次 AST 中每次字段读绑定到同一根参数值；然后原子地把 child 读替换为根方法可见参数，根类输出 `new Runnable() { ... }`。应检查创建时求值一次、参数槽复用/后写入、额外字段、method-handle/跨类引用、预算与取消。无需新的 Java 语法恢复框架；扩大现有捕获证书的输入形态即可。`TestAnonymousClass7` 中 `System.out.println` 的完整方法体可作为正例，`TestAnonymousClass8` 的外层实例仍按 DT-04 边界独立回归。
