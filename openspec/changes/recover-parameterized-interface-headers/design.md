## Context

[桥准入实证](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/README.md)：`BR$Impl implements Comparable`（裸）+ 桥隐藏 → javac 报 `未覆盖Comparable中的抽象方法compareTo(Object)`；`implements Comparable<ParamI>` + 桥隐藏 → javac 重建桥、exit=0。判据范围的 2×2 对照见 [header-invariant/README.md](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/header-invariant/README.md)（root 实测四形，确认唯一失败形是"接口边 ∧ 参数 cast 形"）。

### 精确落点（root 已定位——本片是接线，无需扩展 reader）

- **类头投影处**：`src/class_source.rs:6349` 的 `parameterized_superclass`（由 `parsed.superclass.segments` 是否带 `arguments` 判定）与 6354 的 `direct_parent_candidate`（当前**硬编码** `java/lang/String` 单形参父类）。6360 的 `if parsed.type_parameters.is_empty() && !parameterized_superclass { return Ok(None) }` 是当前"无泛型即不投影"的总门。
- **接口事实已就绪**：`crates/jarde-reader/src/signature.rs` 的 `parse_class_signature`（134 行）**已完整解析** `ClassSignature.interfaces: Vec<ClassType>`（结构定义 22 行、填充 139–148 行），且 `class_references()`（27 行起）已把 interfaces 纳入引用收集。故本片**只需在 class_source.rs 消费 `parsed.interfaces`**，不新增解析器、不改 reader。
- **既有父类通道的可复用件**：`ClassSignatureErasureProof`（signature.rs:222，含 `type_parameters: Vec<TypeParameterErasure>`）与 6340 的 `attribute_facts` 擦除核对——接口投影应复用同一擦除证明结构（每个接口的擦除须与物理 `interfaces` 项一致）。

**第一个取证义务**：确认 `ClassSignatureErasureProof` 是否已含 interfaces 的擦除项（若只含 superclass，需按同一结构扩展擦除证明，仍不新建解析器）；并确认 6354 的 `direct_parent_candidate` 硬编码 `java/lang/String` 是否为既有片的刻意窄边界（本片不得顺手放宽父类边界，只加接口边界）。

## Goals / Non-Goals

**Goals:** 类头 `implements` 子句按 Signature 投影类型实参；不可解析时保留裸类型并拒绝该类桥投影。**Non-Goals:** 嵌套/多层参数化接口（`Map<K,V>.Entry` 形按既有裸回退）；类自身有类型参数的形（`class C<T> implements I<T>`——属 nested-headers 域）；接口方法声明的泛型签名（成员域，已由 9/9 片覆盖）；不改父类投影既有行为（含 6354 的窄边界）；不放宽 `recover-bridge-admission-gates` 的消隐前置（本片只**供给**其依赖的类头事实）。

## Decisions

1. **复用既有类头投影通道**：与 `recover-proved-direct-parameterized-superclass` 同一选定环境、同一"擦除 == 物理 header"核对、同一原子性（整个类头一次决定，不从调用或局部猜类型）。接口列表逐个投影：任一接口不满足判据则该接口保持裸类型（不整体回退已可证的其它接口，除非既有通道的原子性要求整体一致——**按既有实现的原子性口径执行，并在报告中说明选择依据**）。实现上是在 `class_source.rs:6349/6360` 的既有门处并列加入 `parsed.interfaces` 的判定，而非新建一条类头装配路径。
2. **消隐前置不变量（本片与 bridge-admission-gates 的共享契约）**：擦除桥的可重建性来自类头类型实参。故桥投影准入 MUST 检查"该桥擦除契约所属父类型在类头文本已带类型实参"；未带 → 拒绝桥投影、保持桥可见。**本片提供类头投影，该前置由 `recover-bridge-admission-gates` 实现**（owner 分离：类头文本的 owner 是类头投影，消隐决策的 owner 是桥准入）。
3. **验收锚定**：`BR$Impl`（`implements Comparable<BR$Impl>`）类头投影 + 桥投影 → 整类 `javac --release 8` 通过、`-Xverify:all` 运行与原 class 一致（`0`，含经接口引用的 `compareTo` 调用）；多接口形（`implements A<X>, B`）部分可证时按决策 1 的原子性口径呈现；负例（接口不可解析、arity 不符、擦除不一致）保留裸类型且桥可见。

## Risks / Trade-offs

- **与 bridge-admission-gates 的实施顺序**：本片是其前置依赖（无类头投影则 Impl 无法安全隐藏桥）。两片**串行**实施（bridge 先落地其两门与 A 三项，本片随后提供类头投影并启用其消隐前置），共享契约是"类头是否带类型实参"这一事实的读取方式——由本片 owner 定义、bridge 片消费。
- **裸类型保留的信息损失**：接口不可解析时类头仍是裸类型（现行为），但此后桥**必须**可见（决策 2），故不会出现"契约丢失 + 桥隐藏"的双重损失。
- **corpus 面变化**：类头文本变化会波及所有参数化接口实现类的输出——双腿扫描，差异应仅类头与桥家族；既有父类投影与成员参数化测试零回退。
