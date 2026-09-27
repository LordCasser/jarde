# EM-01 `Generic.A<T>`：成员关系、签名作用域与 bridge 的联合边界

固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f` 的 Java 正例要求根源码包含 `public static abstract class A<T> implements Comparable<A<T>>`。现有 [EM-01 回放](../../java-syntax-2026-09-27/em01-declarations/replay.py)中，原类和固定 JADX 的完整 `Generic` 源码经 Java 8 重编、`-Xverify:all` 运行均输出 `1:java.lang.Comparable<em01.Generic$A<T>>`。EM-01 已验收的 `Shape.I`/`Shape.A` 联合声明不含字段、泛型或 bridge，不能代表此正例。`TestClassImplementsSignature` 另有 Raung 畸形签名负例，`TestIncorrectFieldSignature` 是 Smali 负例；二者不计入可编译正例。

额外的外部 consumer [`BridgeRunner.java`](BridgeRunner.java) 用 Java 8 源码直接引用 `Generic.A<String>`，并通过反射检查泛型接口、方法数和桥标志，通过擦除调用检查错误参数的 `ClassCastException`。原始 `Generic.java` 与固定 JADX 的完整 `Generic.java` 分别重编后，在 `java -Xverify:all` 下均产生 [`BridgeRunner.expected.txt`](BridgeRunner.expected.txt) 中的六行；这个结果作为后续 Jarde 全源码验收基线。

`Generic.class` 对唯一 child `Generic$A` 有直接 `InnerClasses` row；child 的 self row 与之吻合。child 物理类为 public abstract，类 Signature 是 `<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/lang/Comparable<Lem01/Generic$A<TT;>;>;`，字段 `value:Object` 的 Signature 为 `TT;`，源码方法 `compareTo(Generic$A):int` 的参数 Signature 引用 `T`，另有编译器生成的 `compareTo(Object):int`（`ACC_PUBLIC|ACC_BRIDGE|ACC_SYNTHETIC`）。Jarde 对 child 的物理读取和方法正文完整，但根源码只输出根构造器；当前 [`multi` 重放](../../java-syntax-2026-09-27/em01-declarations/replay.py)明确只剩 `Generic.A` 缺失。

这里不是单一的嵌套 writer 漏字：

1. `prepare_class_source_member_family` 对静态 child 必须先得到 `ProvedStaticMemberTarget`；`prove_static_family_target` 还要求 child 无字段、无 Signature，因而当前 capture 为 `static no-capture target was not proved from the class-level relation`。这个证明原本服务根方法里的构造使用点，不能充作无使用点声明的唯一关系证书。
2. 独立物理 child 没有根类的词法位置；`project_generic_signature` 因 `Generic$A` 含 `$` 和带参父接口而拒绝类头，故 child 的 `generic_scope` 未发布。字段 `TT;` 与方法 Signature 随后都因 `T` 不在可用作用域而拒绝。这些物理拒绝是诚实的；不能在脱离根类的物理报告中把 `$` 直接改成嵌套源码。
3. `member_family_source_text` 目前要求静态构造 target、无 child 字段、无 child generic Signature；既有泛型家族子形态则只接受另一种根类 `T` 与唯一 `make()` 构造站点。`Generic.A` 无该站点，不能直接复用任一 writer 门。
4. `bridge@1` 对 `compareTo(Object)` 的参数 `checkcast Generic$A` 正确拒绝“纯直接转发”：该 cast 可能抛错，不能按冗余返回 cast 删除。若根源码输出泛型 `Comparable<A<T>>` 与 typed `compareTo(A<T>)`，Java 编译器会生成桥方法；要省略物理桥，需独立证明它与该编译器桥的参数 cast、单次转发、返回和异常行为一致，不能靠 `ACC_BRIDGE` 标志盲删。

架构上仍可沿用现有**物理读取 → 家族关系 → 源单元投影**：先用双向 `InnerClasses`、唯一 child 选定定义与完整 root 使用闭包证明“声明关系”，与构造站点证明分离；在根源单元的词法上下文解析 child 类 Signature，核类型参数擦除、物理父类/接口表及 `A<T>` 的 self 绑定，然后把同一已证作用域用于字段/方法 Signature；最后以物理桥的 Code/SSA 和泛型声明共同证明由源编译器再生的 bridge，整组一次提交。独立 child 报告继续保留其物理名和拒绝说明。需要一个私有的上下文签名/桥再生证书，但不需要新的 JVM IR、公共 API 或按二进制名猜测嵌套关系。

固定 JADX 的参考分层是 `RootNode.initInnerClasses()` 建立 parent/child、`SignatureProcessor.parseClassSignature()` 解析并按接口表核对类型、`ClassGen.addInnerClsAndMethods()` 在根源码递归写 child。Jarde 可以借用顺序和校验思路，但应比 JADX 更严格：签名声明了物理接口表没有的接口时拒绝整个参数化头，不用补写一个不存在的接口。实施前需冻结 `Generic.A` 的 class/字段/方法 Signature、桥方法逐 BCI、完整类三方基线和畸形签名/错桥/错 self row/预算近邻；按一个完整可编译源单元验收，不把中间某个门放宽算作完成。
