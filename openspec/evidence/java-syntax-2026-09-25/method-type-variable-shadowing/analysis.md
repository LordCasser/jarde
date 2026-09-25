# 方法类型变量遮蔽类变量

[`replay.py`](replay.py) 以 `javac --release 8` 的 `-g`、`-g:none` 编译两个静态直接返回方法，并运行 [`StrongCaller`](fixture/StrongCaller.java)。两种 class 在 `java -Xverify:all` 下都打印 `plain`、`bounded`。`ShadowPlain<T>` 的方法另声明 `<T> T echo(T)`；`ShadowBounded<T extends Number>` 的方法另声明 `<T extends CharSequence> T echo(T)`。Java 源级最近的方法变量遮蔽类变量。后一例的 [`javap`](javap-g-none.txt) 表明物理方法 descriptor 是 `(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;`，方法 `Signature` 为 `<T::Ljava/lang/CharSequence;>(TT;)TT;`；类自己的 `T` 第一界却是 `Number`。用类变量解释方法 `TT;` 会得出错误擦除。

当前 Jarde 将方法变量与类变量同名当作 `jvm_signature_scope_unproved`，在两例里都退回物理 `Object` 或 `CharSequence` 方法头，正文 `return arg0;` 本身已恢复。把这两份 class-source 与原始强类型调用方一起用 Java 8 重编，`-g`/`-g:none` 都失败：`Object`/`CharSequence` 无法直接返回为 `String`。JADX 1.5.6 对无界 `ShadowPlain` 输出 `<T> T echo(T)`，强类型调用方可重编；但对界不同的 `ShadowBounded` 输出 `CharSequence echo(CharSequence)` 并留下 `Incorrect return type in method signature` 警告，强类型调用方同样失败。诊断和对应源文件分别在 [`logs/`](logs/) 与 `jadx-source/`、`jarde-source/`，版本和 SHA 在 [`tool-versions.txt`](tool-versions.txt)、[`sha256.txt`](sha256.txt)。JADX 在后一例的警告不能单独定位其内部哪个分支错误，只能确认本样本上的输出边界。

Jarde 阻断位置明确：[`signature.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-reader/src/signature.rs:377) 的 `prove_method_signature_erasure_with_class_scope` 拒绝与类作用域重名的方法变量；同文件 `erase_type` 又先查类作用域，若仅删除拒绝门，会把后一例方法 `T` 错擦成 `Number`。正确边界是在已有结构化 Signature 树内让方法局部变量先于类变量解析，仍分别拒绝各自作用域内部重复、未绑定变量、循环界及与物理 descriptor/Exceptions 不一致的签名。`class_source` 已有已发布类头作用域和静态直接参数返回的同轮候选，不需要新语法解析器、AST 或通用 pass。非静态方法、成员类继承的外层变量、泛型调用点绑定属于其他独立任务，不因本探针自动开放。

后续 OpenSpec 应用两个正例及至少三种负例验收：方法作用域内重复名、未绑定 `T`、界被改成与物理 descriptor 不符；另检验类变量 `T` 与不同名方法变量 `U` 不回归。Jarde 重编两个完整 class 后，原 `StrongCaller` 都要在 Java 8 下编译、执行一致，并比较反射的类/方法泛型签名。尤其后一例可比 JADX 1.5.6 保留更多正确泛型语义。
