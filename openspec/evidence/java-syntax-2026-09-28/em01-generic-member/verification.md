# EM-01 Generic.A 实施验证

固定输入由同目录的 `Generic.class` 和 `Generic$A.class` 保存。前者 SHA-256 为 `c55a1b43d880f69df08b557988acd8c8f3e2158588911e9d3de0e91196c3a890`，后者为 `172a63a58d3f7b62b040a2e038e05f387447943ddecf64a10cf36155b168e1f5`。`Generic-A.javap.txt` 冻结 child self `InnerClasses` row、class/field/method Signature、物理接口表、桥 flags 与逐 BCI Code。root row 与 child self row 相符；child 的物理 `compareTo(Object)` flags 是 `0x1041`，Code 为 `aload_0; aload_1; checkcast Generic$A; invokevirtual compareTo(Generic$A); ireturn`，无异常处理器。桥另有一个精确的 synthetic `MethodParameters` 属性，Code 内无附加属性。

`replay.py` 从固定原源码生成完整 `multi` jar，核两个 class 哈希，并固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。原始、JADX、Jarde 的 `Shape.java` 和 `Generic.java` 都按 `javac --release 8 -g:none` 重编；原 Runner 与外部 `BridgeRunner.java` 都在 `java -Xverify:all` 下运行。三份结果一致：Shape 方法数 `2:1`，泛型接口 `java.lang.Comparable<em01.Generic$A<T>>`，两个声明方法中恰有一个 bridge，typed 与桥的 null 调用均返回 `0`，错误 Object 实参抛 `ClassCastException`。具体输出、CLI 二进制哈希在 `replay-summary.json`。独立 Jarde 物理 child 仍写 `Generic$A` 和物理 bridge；根源码只写 `A<T>` 与 typed 方法。

`negative_replay.py` 逐个构造七个 child 近邻：错 self row、Signature/物理接口不符、字段和方法的未知 `T`、桥参数不再取实参、桥调用错目标、桥 cast 收窄到真实子类。每个类均通过 `java -Xverify:all` 的加载，Jarde 均拒绝根投影。三个桥变化还观测到错误参数被接受、`NoSuchMethodError`、合法 `A` 实参被 `ClassCastException` 拒绝，说明不能省略这类物理桥。额外根方法使用 child 的近邻拒绝；与 child 无关的根方法仍允许无构造使用点的声明投影。输出在 `negative-summary.json`。

定向 Rust 测试覆盖正例的物理 child 保留、派生锚点、全部近邻、分析/输出预算和预取消。`class_source`、`member_family_identity`、`generic_outer_member_family`、字段与方法泛型投影、普通泛型投影以及 `jarde-java` 的 `p3_patterns` 回归通过。此切片只接受固定的单个静态抽象 `A<T>`、`T value`、typed `compareTo(A<T>)` 和精确 javac bridge 形态；其它接口、多个泛型 child、复杂 bridge 保持拒绝。
