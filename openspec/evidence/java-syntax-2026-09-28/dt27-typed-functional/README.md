# DT-27 泛型目标的方法引用：固定基线

本样本取自固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestLambdaStatic`/`TestLambdaInstance` 方法引用方向，并把静态、绑定实例和零参 `Supplier` 放进一个独立完整类。`TypedRefs.java` 经 `javac --release 8 -g` 得到 major 52 的 `TypedRefs.class`，SHA-256 为 `be4ad8a5dffaa846d5ddc99624670f6c8085ab9bff1f34e58f663f40b6cc0334`。三个工厂的物理 descriptor 分别为 raw `Function`/`Supplier`；`Signature` 分别声明 `Function<String,Integer>` 和 `Supplier<String>`，代码均为直接 `invokedynamic` 后 `areturn`，绑定引用前另有 `aload_0`。

原 class 与固定 JADX 生成的完整类都用 Java 8 重编并经 `java -Xverify:all` 运行，输出一致：`5:5:xy`，随后三个工厂的反射泛型返回类型分别为 `Function<String,Integer>`、`Function<String,Integer>`、`Supplier<String>`。固定 JADX 写出 `Integer::parseInt`、`this::length`、`this::label`。

主线 `cdc98472` 的 fresh Jarde CLI：`parse` 输出 raw `Function` 和 `(Object p0) -> Integer.parseInt((String) p0)`；`bound` 因把绑定接收者退化为 lambda 会推迟创建期空值失败而在 BCI 6 拒绝；`supplier` 虽写 `this::label`，声明仍为 raw `Supplier`。三个 `Signature` 投影分别因缺少同轮 `GenericReturnCandidate` 或完整正文证明而拒绝。完整 Jarde 类在 `bound` 处没有返回语句，`javac --release 8` 失败。这证明只恢复 `::` 文本或只恢复泛型头都不足以构成闭环。

独立方法审计还指出：`src/class_source.rs::ordinary_parameterized_declaration` 已有 Signature 解析、擦除验证和类型拼写；`crates/jarde-java/src/report.rs::generic_return_candidate` 只识别 Local/Conditional/New，不给直接返回的 `invokedynamic` 出具正文来源证书。`crates/jarde-java/src/lambda.rs::plan` 把 erased SAM 的 `Object → String` 计为参数适配，故当前静态形态退化为 lambda，绑定形态安全拒绝。后续实现应复用这两条现有路径，把参数化目标类型证明与源码引用兼容证明连接起来；不能仅凭 classfile `Signature` 或 `Reference` handle 就发布源码。

本样本不证明普通 nullable 接收者、一次求值与捕获别名、重载、unbound `Object::toString` 或泛型参数伪造变体。它们分别作为近邻/后续边界验证，不据此宣称 DT-27 整项追平。
