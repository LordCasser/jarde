# 非泛型外层与泛型成员的 qualified diamond 构造

这个 fixture 覆盖一个可单独验证的边界：`Outer` 是非泛型 public 顶层类，声明 public 非静态 `Inner<V>`；caller 方法接收准确静态类型为 `Outer` 的 `outer`，并执行 `outer.new Inner<>(...)`。[`Use.java`](fixture/Use.java) 返回 `Outer.Inner<Integer>`，作为泛型返回类型可见的强对照；[`UseObject.java`](fixture/UseObject.java) 返回 `Object`，隔离出构造表达式本身，不要求 caller 的方法签名呈现 `Inner<V>`。

[`replay.py`](replay.py) 以 Java 8 (`javac --release 8`) 分别用 `-g`、`-g:none` 编译全部原始源码，并通过 `java -Xverify:all` 执行。两个变体的 `Use` 都输出 `7`；`UseObject` 都输出两行 `minimal.Outer$Inner:1`、`null:1`。第一行确认普通调用创建了成员类实例并且构造参数表达式只执行一次；第二行表示 null receiver 导致 NPE 路径，参数副作用计数仍为 1，故 `argument(9)` 没有在失败构造前执行。对应原始运行记录为 `original-g-run.txt`、`original-object-g-run.txt` 及 `g-none` 同名文件。

[`javap-inner-g.txt`](javap-inner-g.txt) 和 [`javap-inner-g-none.txt`](javap-inner-g-none.txt) 显示 Inner 构造器物理 descriptor 为 `(Lminimal/Outer;Ljava/lang/Object;)V`，泛型构造器 Signature 为 `(TV;)V`，类 Signature 为 `<V:Ljava/lang/Object;>Ljava/lang/Object;`。合成外层实例占物理参数首位，却不出现在源级泛型构造器 Signature 中。`-g:none` 仍保留 Signature 与 InnerClasses 信息；局部变量调试表不是识别该窄构造形状的必要条件。

本地 JADX 1.5.6 的输出在这个窄边界上按 caller 形状分化：对强对照 `Use`，它保留 `outer.new Inner<>(...)` 和 `Outer.Inner<Integer>` 返回类型；Outer.java + Use.java 单独用 Java 8 重编成功，运行输出 `7`。对 Object 返回的 `UseObject`，它生成：

```java
Objects.requireNonNull(outer);
return new Outer.Inner(outer, Integer.valueOf(argument(value)));
```

这段源码丢失了限定接收者，并把物理 synthetic outer 实参作为普通构造器实参发出。两个 debug 变体中，Outer.java + UseObject.java 的 Java 8 重编都失败，诊断为“需要包含Outer.Inner的封闭实例”；Outer.java、Use.java、UseObject.java 的完整输出集也因此重编失败。失败诊断保存在 `jadx-UseObject-g-javac.txt`、`jadx-UseObject-g-none-javac.txt` 和 `jadx-full-*-javac.txt`。由于生成源码未能编译，不存在可运行的 JADX UseObject 输出；脚本对成功编译的正对照 Use 执行了 `-Xverify:all` 验证。

另外，[`original-outer-g-none.jar`](original-outer-g-none.jar) 固定保存原始 Outer.class 与 Outer$Inner.class。脚本把原始 `UseObject.java` 单独编译到隔离目录，classpath 只指向该 jar；caller-only 编译成功，运行输出同样为 `minimal.Outer$Inner:1` 和 `null:1`。因此，qualified diamond 在 Java 8 中有效，且只重编 Object-returning caller 时可针对原始成员类二进制编译；JADX 对该 caller 形状的源码恢复会失败。这个结果不证明任意完整 jar 声明都能重编。

这组证据界定了实现目标：对非泛型外层、准确 `Outer` 接收者、Inner 自有类型变量的情形，source constructor 参数应从 Signature 的 `(TV;)V` 恢复，并与物理 descriptor 去掉已证明 outer prefix 后的尾部对齐；源码应保留 `outer.new Inner<>(...)`，不能把合成首参当源级实参。它不覆盖泛型外层 `A<T>.B<V>`、外层变量出现在 Inner 类型/成员签名中的情况、wildcard/bound 推断、多级嵌套 receiver 或重载选择。JADX 的负例也不能替代 Jarde 对 SSA 接收者身份、求值/空值检查/副作用顺序及目标唯一性的物理证明。

环境版本见 [`tool-versions.txt`](tool-versions.txt)。输入源码、原 class、JADX 输出源码与冻结 jar 的 SHA-256 见 [`sha256.txt`](sha256.txt)。重放脚本使用系统临时目录并自动清理，不运行 Cargo/Jarde。
