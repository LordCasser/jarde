# DT-16：实例泛型方法直接返回 `null`

固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`TestUsageInGenerics` 的有效源码断言是 `public <T extends A> T test() {`，其方法正文 `return null;`；本例将界替换为 JDK `Number`、类型移到顶级类，以隔离 DT-16 的方法形参/返回语法，不把成员类路径 DT-02/03 混进失败点。`TestGenericsInArgs` 另涉及 `List<? super T>`、`Set<T>` 与较复杂正文，留在 DT-16/17/18 的后续组合验收。

[replay.py](replay.py) 从 [NullResult.java](NullResult.java) 和外部 API consumer [Runner.java](Runner.java) 以 `javac --release 8 -g:none` 编译。它从固定 JADX checkout 的 CLI 和 Jarde `class-source` 取得完整 `NullResult`，分别与同一个 `Runner` 重编，并运行 `java -Xverify:all`。命令是 `python3 replay.py baseline /absolute/path/to/jarde-cli`；修后 `fixed` 模式写入独立 `fixed/` 并要求三方均成功。冻结源码、原 class hash、`javap -c -p -s` 的精确 `aconst_null; areturn` 字节码、实际两份反编译源码、诊断和结果均在 `outputs/`。

原始 class 和 JADX 完整源码都通过 Java 8 重编及验证运行，输出 `true:T:java.lang.Number:T`，同时检查带显式 `<Integer>` 的调用结果、反射类型变量名称/界和泛型返回类型。Jarde 的方法体正确显示 `return null;`，但 `ClassSourceMethod::project_method_signature` 将泛型头拒绝为 `generic_source_shape_unproved`，当前源码声明是 `public java.lang.Number value()`；原 consumer 因 `Number cannot be converted to Integer` 无法重编，故未运行。不能把 Jarde 的方法体可读性计为本单元成功。

reader 已有方法 `Signature` 解析与 descriptor 擦除证明；`jarde-java::generic_return_candidate` 只产出空 `return;`、直接参数返回、条件参数返回和两个成员创建形状，单语句 `return null;` 没有同轮候选。`class_source::generic_method_declaration` 也只接受静态参数来源的 `T` 返回。缺口可在既有候选接缝增加严格的无参数、无效果、唯一 `aconst_null; areturn` 实例方法形状，并在现有方法签名投影里证明其可以作为任意已解析引用类型变量的值；不需要第二个 Signature parser 或通用类型推断机制。具体拒绝边界在 [独立 OpenSpec](../../../changes/recover-proved-generic-null-return/) 中约束。JADX 的泛型方法输出用于定位并比较，不替代 Jarde 的同轮正文证明。
