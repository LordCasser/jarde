## Context

固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，EM-01 [完整基线](../../evidence/java-syntax-2026-09-27/em01-declarations/report.md)已证明 `Shape` 两声明通过，`Generic.A` 未恢复。`javac --release 8 -g:none` 的 `Generic.class` SHA-256 是 `c55a1b43d880f69df08b557988acd8c8f3e2158588911e9d3de0e91196c3a890`，`Generic$A.class` 是 `172a63a58d3f7b62b040a2e038e05f387447943ddecf64a10cf36155b168e1f5`。child 类 Signature 为 `<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/lang/Comparable<Lem01/Generic$A<TT;>;>;`；字段 `value:Object` 有 `TT;`，typed `compareTo(Generic$A):int` 有 `(Lem01/Generic$A<TT;>;)I`，桥 `compareTo(Object):int` 为 `aload_0; aload_1; checkcast Generic$A; invokevirtual compareTo(Generic$A); ireturn`，flags `0x1041`。

Jarde 当前 `prove_static_family_target` 要求 child 无字段/Signature；`project_generic_signature` 在脱离根类的物理 child 上正确拒绝 `$` 泛型头；字段/方法随之没有 `T` 作用域；`bridge@1` 正确拒绝参数 cast 是“纯直接转发”。本任务不能把这些保护全局放宽。固定 JADX 在 `RootNode.initInnerClasses` 建词法关系、在 `SignatureProcessor` 处理泛型、在 `ClassGen` 写内类；Jarde 借用处理顺序，但必须按真实接口表和擦除证明，不复制 JADX 对畸形 Signature 的宽松修补。

## Decisions

1. **关系证书独立于构造使用。** 在既有 `ClassSourceMemberFamily` 内允许准确的单个 `public static abstract` child 以声明关系进入准备态。双向 `InnerClasses`、唯一选定定义、root/child 完整读取和根方法对该物理 child 的引用闭包仍须通过；没有构造调用不再是假拒绝。有构造调用的旧路径保持原证书，不共享一个未经证明的 `StaticNoCapture` target。
2. **泛型只在源单元上下文投影。** 物理 child 自己的类头/拒绝文本不改。根家族投影重读已选定 child 的 class/field/method Signature，用现有 parser 和 erasure checker 验证 `<T:Object>`、`Object` 父类、唯一物理 `Comparable` 接口、`A<T>` self 类型、字段 `T` 和 typed 方法参数 `A<T>`。词法 `A` 来自受证 relation，不从 `$` 字符串替换猜得；派生文本的每一段须锚定 class Signature、成员 Signature 或物理声明。错接口数/顺序、未知类型变量、错误擦除和异常元数据均拒绝整组。
3. **桥再生作为独立证明。** 对唯一 `compareTo(Object):int` 核 `ACC_PUBLIC|ACC_BRIDGE|ACC_SYNTHETIC`、无异常表/额外属性、精确参数 `checkcast`、唯一 `invokevirtual` 到受证 typed `compareTo(Generic$A):int`、同值返回和无其它效果。此 cast 可能抛错，不能让 `bridge@1` 删除；在已证 `Comparable<A<T>>` 的根源码中省略物理桥，让 Java 8 编译器按该泛型实现再生，并以反射、合法/错误实参路径验收。桥不匹配时保留物理证据且拒绝根嵌套投影。
4. **整组一次写入。** 在同一个 class-source checkpoint 中写 `A<T>` 类头、字段、构造器、typed 方法并省略 bridge，保持根物理文本可精确复现、预算/取消原子性与 child 物理查询。不要先发布 raw `Comparable` 或不带 `T` 的半份声明；只有完整 `Generic` 源码可编译才算本变更成功。
5. **回放和负例。** 扩充固定 EM-01 `multi` 回放，冻结 class 哈希、原/JADX/Jarde 完整 Java 8 源码及原 Runner，再用外部 consumer 验证 `getGenericInterfaces()`、`getTypeParameters()`、`isBridge()`、合法 typed 调用及桥参数 cast 的 `ClassCastException`。制作 verifier 有效的错 child self row、Signature 与物理接口表不一致、字段/方法 type variable 不在 scope、桥目标/参数 cast/附加效果变化、根额外使用以及预算/取消近邻。旧 Shape pair、单成员、一般泛型/桥回归均须通过。

## Risks / Trade-offs

- 把物理 child 的 `$` 头直接改名会伪造源级作用域；故 context 只属于 root family 投影，物理报告不回写。
- 省略真实参数 cast 若无桥再生证书，会把 `ClassCastException` 行为丢掉；必须将泛型接口、typed 方法和桥 Code 作为一个联合前提。
- 固定 Java 8 形态可编译不证明其他 `Comparable` 实现、多个桥或不规则 Signature；未知形态留在明确拒绝状态，不借本切片扩大全局准入。
