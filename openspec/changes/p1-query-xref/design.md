## Context

以 P0 完成为进入条件。本 change 规划消费 bounded in-memory snapshot、top-level locator、Header/bytecode 事实和公共 coverage/diagnostic 契约，组织 X0/X1 产品，并把 nested/MR/Boot 明确留在本阶段；当前不把 Query 和 Decompiler 耦合或宣称已实现。

## Goals / Non-Goals

**Goals:**

- 以不可变 snapshot、physical view 和显式 RuntimeView 作为 query 输入。
- 用 noak/rawzip/flate2 适配层读取已有事实，覆盖 code、metadata、bootstrap、resource consumers。
- 在不构建 CFG/SSA/Java AST 的情况下提供 sound structural XRef、分页和可解释的 Partial。

**Non-Goals:**

- 不实现 X2 resolver、X3 复杂值传播、JVM IR 或 Java 输出。
- 不执行 bootstrap、服务、launcher 或用户 artifact；不自动联网解析依赖。
- 不把 P0 的顶层 locator 改写成全局索引，也不引入持久数据库。

## Decisions

1. **查询模型采用关系类型和 consumer schema。** P1 仅以 `mentions_symbol`、`literal_value` 和原始 CP 查询区分语义；`references_definition`、`may_dispatch_to` 保留 target relation 但返回 `UnsupportedAnalysis`，由 P2 resolver 负责。相比单一字符串搜索，这能避免 CP 候选被误报为实际调用。
2. **物理身份使用显式 root/entry location 和有向 origin chain。** standalone CLASS 以 snapshot root 表示；archive class 以真实 entry 表示。Nested origin 逐层记录父容器中产生 child container 的 entry ordinal/raw name 和 child container ID，不再以伪 entry 或只有节点而没有边的 `Vec<ContainerId>` 表示。普通/root MR 变体、版本变体与嵌套来源正交；相同 class bytes 仍不得合并不同物理来源。
3. **请求视图、视图身份与选择事实分离。** PhysicalView 声明 snapshot 和物理范围，RuntimeView 在其上绑定显式 RuntimeProfile 与 LoadDomain；只有 provider 实际派生的结果才能声称某定义被选择。LoadDomain 只记录 loader identity、委派策略、有序 roots、module mode、外部覆盖和 transformation uncertainty，P1 不据此实现 P2 definition/dispatch resolver。
4. **嵌套和布局使用 provider trait。** 外层 rawzip entry 产生 child source 和 origin chain；MR/Boot 规则在 view provider 中实现。相比把路径规则散落在 scanner 中，provider 能让 PhysicalView 与 RuntimeView 可测试、可扩展。
5. **bootstrap 使用 deferred graph。** 先记录 dynamic use-site，后续按实际 consumer 补 bootstrap 边，配合 visited/depth/edge 预算。相比一次性展开所有 bootstrap，结果不会重复爆炸，也不会把未使用节点当调用。
6. **结果采用 immutable page + coverage。** 每页绑定 snapshot/view fingerprint 和 continuation token，所有终止状态走 P0 diagnostics。相比返回 `Vec` 加 bool，调用方能区分 NoMatch、Unknown、Partial 和 Complete。
7. **依赖只复用 P0 选择。** noak、rawzip、flate2、blake3、serde、thiserror、clap 通过内部 adapter 使用；是否增加任何库必须由后续证据和审计单独决定，本阶段不预设新依赖。

## Risks / Trade-offs

- [Risk] 复杂 Boot 变体或第三方布局无法安全识别 → 保留 candidate、规则版本和 Unsupported，而不是猜测。
- [Risk] consumer 类别扩张导致扫描成本上升 → 每类计数、预算和分页；默认不读无关 Body。
- [Risk] 多层 nested/condy 图出现爆炸 → visited set、深度/边数上限和共享 graph node。
- [Risk] loader/order 语义不完整 → 返回候选集合与 open-world/ambiguous 状态，不伪造唯一解析。

## Migration Plan

实现前以 P0 的 snapshot、entry、Header、diagnostic API 为输入基线。P1 规划完成后用 A01–A08、A14、A17、A18 样本验证，再由下一阶段消费；若契约需要变更，必须通过 OpenSpec 显式修订，不作旧接口向后兼容承诺。
