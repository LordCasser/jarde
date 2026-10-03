## Context

[桥准入实证](../../evidence/java-syntax-2026-10-04/bridge-method-patrol/README.md)：`BR$Impl implements Comparable`（裸）+ 桥隐藏 → javac 报 `未覆盖Comparable中的抽象方法compareTo(Object)`；`implements Comparable<ParamI>` + 桥隐藏 → javac 重建桥、exit=0。既有通道：`recover-proved-direct-parameterized-superclass`（6/6）对 `extends Parent<T>` 的类头 Signature 投影——本片把同一机制的边界扩到 `interfaces` 列表。**第一个取证义务**：读该片的实现落点（`src/facade.rs` 类头投影段），确认它当前是否已读取类 `Signature` 的 `interfaces` 部分（若已读但未投影，本片只是接线；若未读，需扩展读取但不新建解析器），以及 `recover-ordinary-parameterized-signatures`（9/9）的成员级 Signature 解析能否直接复用其逐位置擦除证明。

## Goals / Non-Goals

**Goals:** 类头 `implements` 子句按 Signature 投影类型实参；不可解析时保留裸类型并拒绝该类桥投影。**Non-Goals:** 嵌套/多层参数化接口（`Map<K,V>.Entry` 形按既有裸回退）；类自身有类型参数的形（`class C<T> implements I<T>`——属 nested-headers 域）；接口方法声明的泛型签名（成员域，已由 9/9 片覆盖）；不改父类投影既有行为。

## Decisions

1. **复用既有类头投影通道**：与 `recover-proved-direct-parameterized-superclass` 同一选定环境、同一"擦除 == 物理 header"核对、同一原子性（整个类头一次决定，不从调用或局部猜类型）。接口列表逐个投影：任一接口不满足判据则该接口保持裸类型（不整体回退已可证的其它接口，除非既有通道的原子性要求整体一致——**按既有实现的原子性口径执行，并在报告中说明选择依据**）。
2. **消隐前置不变量（本片与 bridge-admission-gates 的共享契约）**：擦除桥的可重建性来自类头类型实参。故桥投影准入 MUST 检查"该桥擦除契约所属父类型在类头文本已带类型实参"；未带 → 拒绝桥投影、保持桥可见。**本片提供类头投影，该前置由 `recover-bridge-admission-gates` 实现**（owner 分离：类头文本的 owner 是类头投影，消隐决策的 owner 是桥准入）。
3. **验收锚定**：`BR$Impl`（`implements Comparable<BR$Impl>`）类头投影 + 桥投影 → 整类 `javac --release 8` 通过、`-Xverify:all` 运行与原 class 一致（`0`，含经接口引用的 `compareTo` 调用）；多接口形（`implements A<X>, B`）部分可证时按决策 1 的原子性口径呈现；负例（接口不可解析、arity 不符、擦除不一致）保留裸类型且桥可见。

## Risks / Trade-offs

- **与 bridge-admission-gates 的实施顺序**：本片是其前置依赖（无类头投影则 Impl 无法安全隐藏桥）。两片**串行**实施（bridge 先落地其两门与 A 三项，本片随后提供类头投影并启用其消隐前置），共享契约是"类头是否带类型实参"这一事实的读取方式——由本片 owner 定义、bridge 片消费。
- **裸类型保留的信息损失**：接口不可解析时类头仍是裸类型（现行为），但此后桥**必须**可见（决策 2），故不会出现"契约丢失 + 桥隐藏"的双重损失。
- **corpus 面变化**：类头文本变化会波及所有参数化接口实现类的输出——双腿扫描，差异应仅类头与桥家族；既有父类投影与成员参数化测试零回退。
