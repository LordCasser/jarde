# EM-23 loop continuation：v3 补充

本页只收窄 v3 的三个架构判断；它不确认实际失败点，也不提出产品实现。动态诊断仍待完成。

## 两步续走先例的范围

`continue_multi_return_loop_arm`（`region.rs:4162`）的确展示了一个 walk 可在 header 返回后再次调用 `region_at`：最多续走两次，并在结果为空或没有前进时停止。但它只在调用点 `3541–3546` 的 `frame.multi_return_finally_rows.is_some()` 分支中被调用，服务于特定 multi-return/finally 两段证明。因此它只能证明局部续走模式已存在，不能据此给普通 one-arm 或任意 finally 行签发证书，也不能省略新形状自身的 scope、edge、exit 与 ownership 证明。

## Region ownership 与 source-map provenance

Region 树的职责是分配物理 CFG block。恢复结束时，未覆盖块会单独记录；`overlapping_owner`（`region.rs:1529`）发现同一 block 被多个 region 认领时会拒绝结构化树。这里要求每个可达物理 block 恰有一个 owner。

source map 回答的是另一问题：生成文本由哪些字节码事实产生或呈现。`jarde-java/src/source_map.rs` 的 `OriginSet` 含一个 primary 与有序 derived origins；不同文本 segment 可以合法引用同一 BCI，同一节点也可以呈现 primary 之外的来源。故不能要求“每个 BCI 只出现一个 anchor”。审计应逐段比较完整来源关系：segment 范围、每个 origin 的 method identity/BCI/CP/provenance，以及 primary 和全部 derived 项；允许重复 BCI，但重复关系必须与原方法及既有映射精确一致。Region block 的唯一 owner 检查不能替代这项来源核验。

## 失败路径不需要通用事务

普通 one-arm walk 若拿到非 join continuation，而现有 `continue_early_return_arm` 不接受，会设置 `unclosed_tail_at`，返回无后续 continuation 的 gap（`region.rs:3447–3456`）。顶层 walk 随后在 `1375–1380` 检查该标记并直接 `quoted_whole`。因此这条拒绝路径不会把试走形成的局部 Region 发布成结果；保留 whole-method refusal 已足够，不必新增通用 transaction/rollback 实体。

若新逻辑在同一拒绝路径上直接标记 `unclosed_tail_at` 并停止尝试，`visited` 无须为后续候选恢复，因为不会再有候选 walk，且最终树会整体引用。只有设计成“试走失败后继续尝试另一候选”时，才需要恢复会影响后续决策的 Walker 状态；至少要核对 `visited` 与 `unclosed_tail_at`，并逐项审查新引入的持久状态。预算消耗和 Stop 不应回滚：Stop 应原样传播，不能变成普通 shape refusal。建议本片避免 alternatives，沿已有 whole-method refusal 收敛。

**范围：** 上述是源码结构判断，不是动态触发链结论。需等带 method identity 的诊断确认实际 arm continuation、loop-header 分支及早退位置后，再决定是否值得实施。
