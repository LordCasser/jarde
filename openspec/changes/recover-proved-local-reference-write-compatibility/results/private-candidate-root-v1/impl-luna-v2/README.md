# 私有候选 v2：局部 reference write compatibility

只含 `build.rs`。共享基底 SHA-256：`3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7`。

- `build.rs` candidate SHA-256：`7174f770e76045117690409e622eae8eadc657492a1702050ef1272dd999a5f6`
- `build.patch` unified diff SHA-256：`405026eb445f48b99e15894612f539f0fff6aeb9be3e34423386f4ad66852aed`
- v1 保留在 `/private/tmp/jarde-local-reference-write-impl-luna-v1`，不再改动。

生产决定仍仅在普通首写窄 scalar/reference fallback 中触发：精确 Object 写需来自 local `Store` 的唯一 Stack 输入，且 `store_operand` 的直接 producer helper 精确返回 `java.lang.Object`。首存是非 null reference；参数、resource/catch header、数组首型排除。一个仅含物理 Object 写候选的 slot index 延迟构建一次，包含 reuse identity；cleanup 中被 SlotUse 忽略的 Object 写也可触发拒绝。None identity 触发歧义拒绝，映射到其它 identity 的 Object 写隔离。完整性证明逐项扫描同一 physical slot 的 SSA local reads/writes，且以 (BCI, read value, written value, stored operand) tuple 多重集逐项核对 SlotUse；所有 instruction/access traversal 均先 charge/poll。

证明要求全写 path 完整、written/stored 是 initialized reference 或 null、至少两个写和一个真实读；已知静态数组只依据 direct producer 或命名 descriptor 拒绝。每个 Load 必须只输出单 Stack，Definition block/BCI 与索引一致，所有 uses 恰好一个且实际 consumer 唯一读取该 Stack、是 Object-return 方法里的 `Return` 并无写出。含异常处理表的本方法切片拒绝这项 widening；精确 Object 写后的失败通过现有 `Decided::Unknown(NoType::ConflictingWrites)`，文案不声称不存在任何容纳该值的 Java 声明。proof helper 直接返回 `Option<Decided>`：None 未触发，Some(Type(Object)) 通过，Some(Unknown(...)) 触发后拒绝。

无需增加新类型实体、递归数组/LUB/identity 服务或永久内部测试；真实 API fixtures 由 root 准备。未运行 Rust/JDK/JADX、编译、格式化或测试。此候选未验收，不能视作产品实现完成。
