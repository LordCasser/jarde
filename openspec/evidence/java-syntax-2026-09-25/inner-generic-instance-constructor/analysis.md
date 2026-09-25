# 泛型外部实例创建非静态内部类：两种调试表下的三方边界

[`fixture/OuterGeneric.java`](fixture/OuterGeneric.java) 使用 Java 8 的 `A<String>.B<Integer> b = a.new B<Integer>(3)`。[`replay.py`](replay.py) 以 `javac --release 8` 分别生成 `-g` 和 `-g:none` class，原程序经 `java -Xverify:all` 都打印 `3`。`OuterGeneric$A$B.<init>` 的物理 descriptor 为 `(LOuterGeneric$A;Ljava/lang/Object;)V`，泛型 Signature 为 `(TV;)V`：外部实例是字节码中的隐式首参，不占源级构造器参数位置。外部调用点还包含 `Objects.requireNonNull(a)`，并将 `a` 传给 `invokespecial`。

本地 JADX 1.5.6 把三份 class 合并为嵌套声明，却把调用写为 `new A.B<>(3)`（无调试表时为 `new A.B(3)`）；两种完整源码的 Java 8 重编都失败，提示非静态 `B` 缺少封闭 `A` 实例。局部 `Objects.requireNonNull(a)` 不能替代 `a.new B(...)` 的源级绑定。本地 JADX 测试 [`TestOuterGeneric.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/generics/TestOuterGeneric.java) 对相同形状标记 `@NotYetImplemented("Instance constructor for inner classes")`；`InsnGen.addOuterClassInstance` 仅在构造器标记 `SKIP_FIRST_ARG` 且首实参类型等于声明外部类时发射外部实例，这是一条值得核查的现有算法门，但本证据未归因到门中的唯一分支。

Jarde 当前也未恢复该语法。外层 `run` 在 `new`/`dup`/隐式外部实参/空值检查组合处给出未证明形状，故不宣称完整类可编译；独立读取 `A`、`B` 时 `class_generic_source_unproved` 保留嵌套类声明拒绝，`B` 的 `V` 字段、构造器与返回类型因可用类作用域不足而 `jvm_signature_scope_unproved`。这比生成错误的外部绑定更诚实，但与可恢复源级形态仍有差距。

| 变体 | 原外层 class SHA-256 | 原 `A` SHA-256 | 原 `B` SHA-256 | JADX 源 SHA-256 |
| --- | --- | --- | --- | --- |
| `-g` | `0b4abfdb36c92c84883c41769bb90b2991b4f30717df076ad63583f149ac8e9b` | `62a3c86094c26fe8141d9dbd5f6b6768d8b131054158db046d799cd3af13fb67` | `ed76095df92583aecf1a7aa13b56f387dc984aa76d146b40007712dfc42bc60a` | `387b4824b96c068c5bfb42c7c8d460c10aec2cecec746384649bd87b8b927286` |
| `-g:none` | `cf8cf3322fe9d18d603725a7f61d19c37be5468ec458cdfabdfffa15ce970de4` | `e2e64157f7047093ce609f7527047079ba3df0acb7de29a9f4fdfbb92c59b300` | `5b7537d2109ca44d920dd1d54424b822aee911cde59a619e02d0691b74317cfd` | `172a3d753fa1fb0c9122542d64fe091aef776fc2452686bd2fac83183b45c49d` |

架构上先证明 `InnerClasses` 所述非静态嵌套关系、构造器物理首参与实际 `this$0` 初始化相符、调用点的外部实例求值/空值检查/副作用顺序不变，才可把首参从源级参数列表移除并写 `a.new B<Integer>(3)`。类级 `T` 与 `B` 自有 `V` 的作用域、`Signature` 中缺席的隐式形参，以及三份 class 的嵌套声明装配必须同时闭合。现有 reader 擦除证明不能直接将 `(TV;)V` 当物理二参构造器签名；应在已证明的嵌套构造边界内对齐源级位置，而不是普遍跳过构造器首参。这个工作比单方法泛型 `throws` 首片大，应独立 OpenSpec，不混入当前声明改动。

可重放的 fixture SHA-256：`b463eef3f46112e9bc3f9d40be05d5f37d2f56f25cf25518a18d05dfcd8dfc1a`。脚本使用私有 Cargo target 和临时输出，结束时清理；以上归因限本地 JADX 1.5.6 与当前 Jarde 工作树。
