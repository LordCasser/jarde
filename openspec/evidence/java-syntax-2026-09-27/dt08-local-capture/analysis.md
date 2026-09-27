# DT-08：匿名接口捕获局部参数的隔离差距

固定 JADX `inner/TestAnonymousClass7.java` 直接断言 `final double d` 参数被匿名 `Runnable.run()` 引用，而不是以 synthetic 字段呈现在源码中；`TestAnonymousClass8.java` 的外层实例引用另与 DT-04 已验收子形态交叉。本切片只考察前者：顶级类的 `static Runnable create(final double d)` 直接返回唯一匿名体，Runner 分别捕获 `2.5` 与 `-0.0`，通过原始及重编源码运行来区分值传递。没有局部写回、异常、第二分配点或其它捕获。

`replay.py` 使用固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 与冻结 Jarde CLI（哈希见 `baseline/summary.json`）。原 class 和 JADX 完整源码均通过 `javac --release 8`、`java -Xverify:all`，逐行输出 `2.5`、`-0.0`。JADX 根类返回 `new Runnable() { ... println(d); }`。Jarde 根类报告 `anonymous_child_shape_unproved`，输出 `new p.Capture$1(arg0)`；其物理 child 源码保留 synthetic `final double val$d` 和构造器赋值。该赋值在 `super()` 前，故完整 Jarde 源码在 Java 8 下编译失败，不能称为语义运行一致。第二次独立重放的 summary 与冻结文件逐字一致，输入、生成源码、`javap`、编译诊断都在 `baseline/`。

这是两个相连但不同的边界。Java 8 class 的 child 构造器确实先写 synthetic 捕获字段，再调 `Object.<init>()`；JVM verifier 接受，Java 8 源码中的具名物理 child 却不能照此顺序书写。根类的匿名投影当前只识别无字段 child，或 `this$0:LOuter;` 的单字段外层实例捕获证书；`D` 类型参数捕获没有获证，因此退回无法重编的物理源码。不能把 `val$d` 的名字、`$1` 后缀或参数出现顺序本身当成捕获证明。

架构方向是在当前唯一分配点、准确 `EnclosingMethod`、整类 owner 使用普查之上，复用物理字段/构造器写入/SSA 读取链的已有捕获证明结构，将准确字段 descriptor `D`、构造参数 slot、根方法创建点实参和同次 AST 中每次字段读绑定到同一根参数值；然后原子地把 child 读替换为根方法可见参数，根类输出 `new Runnable() { ... }`。应检查创建时求值一次、参数槽复用/后写入、额外字段、method-handle/跨类引用、预算与取消。无需新的 Java 语法恢复框架；扩大现有捕获证书的输入形态即可。`TestAnonymousClass7` 中 `System.out.println` 的完整方法体可作为正例，`TestAnonymousClass8` 的外层实例仍按 DT-04 边界独立回归。


## DT-08 实现验收

实现复用 `prove_family_capture` 的同一捕获字段 SSA 扫描结构，但只接受匿名接口 child 的唯一 synthetic-final `D` 字段、唯一 `(D)V` 构造器，以及 javac Java 8 的严格构造器序列：BCI 2 写入字段，随后调用声明的直接 superclass 构造器；异常范围、额外字段写入、非直接字段访问和指向字段的 method-handle 均拒绝。还要求 child 至少有一个被证明的字段读取。根侧仅当创建实参是唯一使用该根 `double` 入口值的直接参数加载，并且 allocation / `EnclosingMethod` / 整类引用普查均闭合时才投影；按同次 AST 的 read BCI 和 field identity 把每个 child 字段读改写为根参数名。合成字段和构造参数只有整个投影成功时才从根匿名体隐藏，child 自身仍可按物理类查询。

定向 `class_source` 回归共 77 项通过，包含新增的正例和错槽、参数槽复用、错误字段 owner/descriptor、错误读取 owner、构造器额外写入、异常范围、method-handle、跨类构造器使用、预算及取消边界；完整同类切片也覆盖既有无捕获接口匿名体、`this$0` 捕获与静态初始化块行为。参数槽复用的变体在普通根方法形状恢复阶段即被拒绝，根源码没有匿名投影，child 仍独立报告。其它负例按预期给出对应匿名/捕获拒绝且保留物理 child。已有单类多分配和跨类 owner-use 用例继续验证整类封闭边界。

固定三方回放由 `replay-implementation.py` 重放并写入 `implementation-replay/`：JADX revision `2fb1b16386941660fda07e9017285aec40fcb37f`；Jarde CLI SHA-256 `147dbc6ec918b7eea1dcf0b61c77531f9bbbd2e2dfa69bc9f63970c90298b4f8`。固定输入哈希与冻结基线相同。原 class、固定 JADX 完整源码、Jarde 完整恢复源码都通过 `javac --release 8`，并通过 `java -Xverify:all`，逐行结果均为 `2.5`、`-0.0`；三方效果顺序一致。Jarde 根类返回匿名 `Runnable`，不含 `Capture$1` 源级引用；独立 `class-source` 查询仍成功呈现物理 `Capture$1`。冻结 Jarde 基线继续记录 Java 8 编译退出 1 和拒绝 `anonymous_child_shape_unproved`。验证还包括 `cargo check --workspace`、`cargo fmt --all --check` 与 OpenSpec strict。

范围仍限于一个直接返回的匿名 `Runnable`、一个直接捕获的根 `double` 参数和一条物理 child 字段链。多字段捕获、外层实例与局部参数共同捕获、局部变量复杂赋值以及闭包链没有在此变更中获证。
