# DT-17：单个通配符 `List` 参数的空 `void` 方法

基线是本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f`。JADX `TestGenerics` 对 `List<?>`、`List<? extends A>`、`List<? super A>` 有显式文本断言，`TestGenerics3` 对 `? extends byte[]`、`? super int[]`、`? super Integer`、`? super String` 有断言。这里把用户定义的成员类界 `A` 换成顶级可解析的 `Number`，避免 DT-02/03 成员类路径；固定 `?`、`? extends Number`、`? super String` 和两个原测试的数组界。`raw(List)` 是无 `Signature` 的负控制。测试中的 `mthExtendsString` 实际参数是 `? super String`，以源/断言内容为准。

[replay.py](replay.py) 以 `javac --release 8 -g:none` 构造原 class，并分别取固定 JADX 和 Jarde 的完整 `Wildcards` 源码。三份类型源码均与同一个外部 [Runner.java](Runner.java) 重编、用 `java -Xverify:all` 运行；runner 逐方法读取真实反射参数类型，避免“能编译但丢失 Signature”被误判为成功。运行 `python3 replay.py baseline /absolute/path/to/jarde-cli`；修后 `fixed` 模式独立写 `fixed/`。原 class SHA、字节码、双方源码和结果在 `outputs/`。

原始与 JADX 的五个泛型参数分别是 `List<?>`、`List<? extends Number>`、`List<? super String>`、`List<? extends byte[]>`、`List<? super int[]>`；无 Signature 控制仍是原始 `List`。完整 Java 8 编译与验证运行均成功。Jarde 的完整源码也编译和运行成功，但五个方法的参数在反射中全部退化为 `java.util.List`；`raw` 仍正确保留原始类型。这是可观测的 Signature 丢失，而非 javac 失败。

reader 的 `Signature` parser 与 `class_source::spell_ordinary_signature_type_with_member_path` 已经认识三种通配符和数组界。具体断点是 `jarde-java::generic_return_candidate` 的 `EmptyVoid` 候选只允许没有参数；同轮存在一个未使用参数时，`ordinary_parameterized_declaration` 得不到正文候选，五个方法都记录 `ordinary_generic_source_unproved`。可复用现有候选、参数槽名和原子方法投影，为“单个 `List` 通配符参数 + 精确无效果 `return;`”增加证明门；无需新增 wildcard parser 或通用类型推断。其它含通配符的方法体、字段、返回值、成员类界留在 DT-17/18 后续切片。
