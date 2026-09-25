## Context

根独立重放 `array-initializers/root-7747/`：原 class 与 JADX 完整源码都能用 Java 8 重编译，`-Xverify:all` 执行 14 行逐字节一致；jarde 整类因四个非空初始化方法缺少返回语句而编译失败。`literalInts` 的真实形状是 `iconst_4; newarray int; dup; iconst_0; iconst_1; iastore; …; areturn`。`effectfulInts` 的每个值来自一次 `intElement(mode,index)`，异常模式分别停在第一个、第二个、第三个元素。空数组 `new T[0]` 是已恢复对照。

当前 `ExprKind::NewArray` 只有元素类型与 `lengths`，创建值仅在最终消费者处呈现。`chained_pair` 正确地只把 `dup; local-store; local-store` 当作链式赋值，并明确排除初始化器的 `dup; index; value; arraystore`；这条规则不可放宽。普通 `ArrayStore` 是有副作用语句，不能单靠相邻指令文本拼接成表达式。

## Goals / Non-Goals

**Goals:** 只恢复经现有 SSA/块指令证明的一维数组初始化链，保持 JVM 分配、元素求值、写入、异常与结果引用语义。

**Non-Goals:** 不恢复任意写入后的数组、跨块/循环初始化、数组逃逸或别名、稀疏/重复/倒序索引、多维嵌套初始化、未知引用组件转换；不改普通数组赋值及 `chained_pair` 合同。

## Decisions

1. 在现有 Builder 的数组创建值上做局部形状证明。创建长度必须由常量事实精确证明为 `N`；同一基本块中紧接一条有限 `dup; constant-index; value; arraystore` 链，索引恰为 `0..N-1`，最后只剩原数组引用给一个已支持消费者。SSA 证明每条 store 操作数都是**同一分配身份**的 copy；整个链中不得出现额外使用、逃逸、控制流入口、异常处理边界或元素计算之外的独立副作用。每条指令/SSA 访问按既有预算计费、轮询取消，链长受 Code/IR 与数组大小共同约束。
2. 在既有 `NewArray` 表达式附加可选元素列表；有列表时仅准入总 rank 1，发射 `new T[]{…}`，无列表时维持现有 `new T[length]`，部分维度分配仍由其独立 rank 事实处理。元素依序经现有 `render_value` 和位置/类型准入渲染；不能证明 Java 初始化器赋值与对应 `*astore` 的组件检查等价时拒绝，尤其不能将可能的 `ArrayStoreException` 偷换成 `ClassCastException`。
3. 分配是初始化表达式的起点，元素表达式只在各自已验证的位置求值一次；共享的 deferred 绑定负责跨独立副作用的生产者顺序。形状认领分配、每个 `dup`、索引和 store 后，普通数组语句/`chained_pair` 不再重复呈现它们。来源包含每个实际 BCI；失败则引用完整链，不产生一半初始化器或丢失 store。
4. 最终值可进入已支持的 return/local/field/call 消费位置，但仅在其现有类型与一次求值规则准入后提交。额外数组读取、先暴露引用再填充、动态长度、非顺序索引一律维持拒绝或原本的普通数组语句，不能用初始化器掩盖运行差异。

## Risks / Trade-offs

- `new T[]{…}` 的源码在分配后才逐个求值元素；与真实链顺序一致，但把可抛错的长度计算换成常量会改变行为，所以只接受静态常量长度。
- `aastore` 对运行时组件的检查不等于 Java 编译期赋值规则；用现有元素类型证明约束准入，不猜外部继承关系。
- 分配与 store 的来源是多个指令，同一个 AST 节点必须完整承载，默认/all 证据与输出预算仍走现有通道。

依据：[JLS 8 §10.6 数组初始化器](https://docs.oracle.com/javase/specs/jls/se8/html/jls-10.html#jls-10.6)规定新数组长度由元素数决定、先分配再按源码顺序求值，且每个元素须与组件类型赋值兼容；[§15.10.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.10.2)区分带尺寸的数组创建与带初始化器的创建。
