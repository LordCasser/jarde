## Context

`CT.cov` 的 `anewarray java/lang/Number` 分量与 `Integer.valueOf` / `Long.valueOf` 返回类型均已有字节码事实。当前 `Builder::array_initializer_element` 只接受 null、同型和 Object；`ArrayInitializers::prove` 已负责 fresh allocation、物理 index/store 顺序、效果边界、唯一消费者及闭合使用链。

2026-10-09 对照覆盖 CT、六 wrapper、CharSequence、Collection、Throwable、Number[][]、同 jar 自有类/接口，以及合法 BigDecimal 控制。初始 BigDecimal 源码错误原样保留，合法修正版与冻结 CT.class 在 additive evidence 中重放。JADX 初始 JVM 入口漏包名也保留，再按实际生成包名复核；统计以最终双流核对为准。Jarde 有的源码能编译但 main 输出丢失，不能以 exit 0 计成功。完整失败分析及原 manifest 必须先落入证据再实施。

## Goals / Non-Goals

目标是让所有现有事实可证明的 initializer 子类到分量关系得到恢复，包括平台类/接口、自有快照层级及等秩引用数组提升。保持 Java 数组构造语义，且完整类源在隔离编译和验证运行中与原程序一致。

不新增全局 assignability 或 LUB 服务，不按任意名字扫描 jar，不改变 invocation overload 选择，不重排元素求值，不对缺证明的合法输入谎称非法赋值或已覆盖。

## Decisions

### 1. 沿用既有 initializer 结构证明，只修正元素兼容判据

组件来自实际分配的 `Operation::NewArray` / reader descriptor；`aastore` opcode 自身只表示引用存储，不能推断 Number 等组件。只有已通过 fresh allocation 与 store 配对证明的初始化器才能消费放宽关系。

null、同型、Object 的旧路径及 primitive/boolean 处理保持不变。其后复用实际存在的 `array_reference_widens`、`platform_array_argument_widens`、`release_reference_argument_widens`、`java_lang_throwable_widens`、`platform_interface_argument_widens`。等秩引用数组可以把已有标量兼容事实提升到分量关系；最多 255 秩，primitive 不变性与 rank/array-to-scalar 边界仍由实际形状约束。只增加必要的私有判据，不复制平台表，不改 invocation 的旧数组覆盖范围。

`reference_overload_calls` 是调用选择证据，不能当作数组赋值证据。成功接受原元素表达式，不插入 per-element cast，不改变 new-array runtime 类型；避免把原 ArrayStoreException 变成 ClassCastException 或隐藏非法存储。

### 2. 扩展现有快照证据生产者到准确的 store 站点

在 `prove_snapshot_hierarchy_widenings` 的现有 instruction scan 中识别 `aastore`，从同 BCI SSA reads 按绝对 stack depth 取得 arrayref、index、value；不能假定从 Stack(0) 开始。数组 named descriptor 剥去一层取得所需分量；元素 named 类型是 source。descriptor 合法性沿用 reader，不引入第二套类型解析。

标量类关系复用 `selected_reference_header` 与有界 `snapshot_header_chain_widens`。等秩引用数组的最深非 primitive 类关系也可通过同一 walk 证明，记录的 source/target 仍是完整 Java 类型拼写，消费时按 store BCI、完整 source、完整 component 精确匹配。rank 不相等或不能由已有数组形状证明时不猜测。

继续使用当前 `ProvedSnapshotHierarchyWidening` 载体、同一请求 Runtime/loader selection、cache、poll、analysis charge、dependency depth 与最大 8 层 walk。不得借其他 invocation BCI 的关系或跨方法/环境记录授权当前 store。只在当前 class-source assembly 路径提供此事实，method-only 缺失时不伪造。

当前 walk 在读取目标 header 前检查名字命中：一个真实且被本次环境选中的 snapshot header 可直接声明外部 superclass/interface，因此目标自身不必物理存在于 snapshot；中间节点必须可按既有 resolver 完整读到。修正 report 中与该行为冲突的注释，行为不收窄也不扩成任意平台闭包。缺失/歧义 source 或中间 header、深度/预算耗尽、未知类型或身份不确定都不产生新证明。

### 3. 采用 Jarde 的效果证明，不照搬 JADX 的索引排序

JADX `ReplaceNewArray` 将 store 收集到 TreeMap 并按索引呈现，另有 args assigned TODO。本项维持 Jarde 的物理写入/求值顺序与效果闭包；乱序、重复索引、逃逸、旧数组更新和 handler 边界继续拒绝 fold。`TestArrayFill2.test2` 的 postfix/evaluation 缺口独立记录，不能作为异构引用数组已恢复的证据。

## Verification

1. 冻结旧 CT.class 与双 javac 8/23 Java 8 target；CT 的完整 cov、up、io、io2 和 main 同时验收。源码必须完整，不删除拒绝成员。
2. 标量家族包含六 wrapper、CharSequence、Collection、Throwable；数组包含平台提升和同 snapshot 自有类等秩提升；自有层级至少 direct superclass、两跳和 interface。全部 top-level class 都须由 Jarde 自行生成后一起编译，候选 runtime classpath 只有新编译目录。
3. 每条正例比较原程序、JADX、Jarde 的实际 stdout 和 stderr、退出码，运行使用 `-Xverify:all`；保存源/字节码/tool input、命令、raw双流、hash及失败记录。冻结 CT.class 重放与重新编译 CT 的腿分别计数。
4. 对抗性控制：降向/无关类、primitive/rank 不符、未知或缺失中间层级、超深、取消/预算、站点错配、非零 stack prefix、旧窄数组更新、乱序/重复 store 和副作用。BigDecimal 若无现有证明仍属合法但未覆盖，禁止计为非法负例。
5. 保持已有同型/null/Object 与不相关方法呈现；同一冻结基线核对差异。集中 Cargo 运行 focused、双 seed workspace、MSRV、fmt、CI-exact clippy、必要 ignored tests、strict OpenSpec、fingerprint/P5预算账本和 diff check。实际 CI 完成才勾选 CI 验收。

## Integration Boundary

2026-10-09 完整 baseline 另发现 direct `new` 元素在类型判据之前被构造器与数组证明拒绝。该组合独立规划于 `compose-constructed-reference-array-elements`；当前类型片以从创建起完整的 factory-element family 验证赋值事实，并把独立完整 direct-new family 及原18腿保留为未覆盖控制，不删除原类成员或宣称其通过。类型兼容契约不收窄，已证明 fresh initializer 的所有既有兼容事实仍须恢复；缺少构造组合证明的路径待相邻任务补齐。

## Risks / Trade-offs

SSA 原始 source 与 AST presented source 可能不同，精确匹配失败时拒绝，不通过猜测补全。存在外围完整类源失败时单独报告，不能以方法局部成功替代 EM-18 验收。闭表及有限 snapshot walk 仍非完整 Java 层级检查器；未覆盖范围在账本中明确保留。
