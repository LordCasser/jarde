# EM-23 one-arm loop continuation：架构草案

本草案承接 v2 的候选链，不把它当成已经动态定位的 site。已接受的 CLI probe 只记录外层 branch 0 的 join（45）和两个 successor（6、45）；stderr 没有递归臂返回值，也没有方法身份。所以下面的方案先以“arm walk 在 loop header 停下”作为待验证假设。

## 边界与可复用结构

`Frame::arm`（`region.rs:2088–2110`）把外层 join 作为 boundary，保留外层 scope、loop targets 和 try/finally 状态；`Frame::loop_body`（约 2025–2085）把自然循环块与父 scope 取交集，不允许子循环扩大父范围。当前 `region_at_inner` 到达 loop header 时，如果 `prefix` 非空，就返回已读的 straight 前缀和 header continuation（`2727–2741`）；空前缀才进入 `loop_region`。这能解释 prefix 先于循环语句已被声明，但外层一臂后续目前只用 `continue_early_return_arm` 接受 boolean 早返回 shape（`3447–3456`、`3965–4051`）。

循环本身已有完整的入口与覆盖证明可复用：`loop_region` 取 natural loop、先拒绝 irreducible，再试 latch/header-tested 形状（`9098–9177`）；header-tested loop 用 `loop_body` frame 和 `loop_body_sequence`，最后对 expected blocks 调 `covers`（`10086–10328`、`11614–11638`）。因此较小方向是让外层一臂在合适的 header continuation 处，通过 `region_at(header, same_arm_frame)` 请求现有 loop proof；不要直接绕开统一入口调用 `loop_region`，也不要给 `Frame` 新增通用 continuation 状态。只有取得单一、完整、structured 的 loop run，且该 run 的 `exit` 与返回的 `next` 都恰为外层 join，才能把此前 prefix 与 loop 作为同一个 arm 的有序 sequence；任何 loop fallback 都保留当前 refusal。

这只够说明可复用方向，不等于可以直接把 `next=header` 接受。拼接前仍须证明：prefix 的正常流确实到达该 header；循环没有另一个从此 arm 以外进入、却被本 if 错误收编的路径；循环的正常出口是外层 join；prefix、loop header/body、gateway/transfer blocks 在最终 Region tree 中各有唯一 owner。可参考 `loop_arm_join_source`（`4054–4160`）的自然循环全集、单入口、单出口和已 visited 检查，但它目前要求 branch 直接边到 loop header，不能原样证明有 prefix 的入口。`two_level_loop_join_sources`（`4482–4571`）则是更窄的 `Straight + Endless loop`、固定三块自然循环、两个 exit 的专用 join 证明，也不是普通 iterator/header-tested loop 的通用证书。

`continue_inner_join_arm`（`4283–4479`）已允许一个 inner-if 在未访问且仍在 scope 内的 loop header/continue target 结束，随后由它的**外层 loop-body sequence**继续认领该块。它不把 loop 吸进 inner-if，也不负责顶层 null guard arm 的剩余部分；本例的调用者是 one-arm branch，而不是 enclosing `loop_body_sequence`。`continue_effectful_loop_arm`（`4193–4277`）是相邻先例：识别 `Straight + loop header`，但只恢复专用 effectful dual-exit 形状，并依赖 `if_arm` 标记；one-arm branch 未设置这个标记，不能直接当作本例实现。

结论：现有 `Frame`、`Run`、`Region::Sequence` 和 loop prover 足以承载一个窄的 caller-side continuation；看不出需要新的全局 pass、frame 字段或 CFG 框架。可能需要的只是复用这些 API 的局部拼接/验证步骤，且必须先由诊断确认 header-return 假设。

## 不变量与拒绝边界

若后续确认并实施，成功路径至少要保持这些不变量：

- **唯一性和进展：** 只从未访问、位于当前 scope 的真实 loop header 恢复；不能把当前 `LoopTarget` 的 header/continue back-edge 重新当成新循环。每步须消费新的 Region blocks，循环 run 的 `next` 前进到 join；不能因再次返回 header 而自旋。
- **精确 ownership：** prefix 从所选 if successor 到 loop header 的边为正常边；loop proof 覆盖 natural-loop expected set；branch 的空臂与非空臂不重叠；loop 的完整 exit/owner 证明同外层 join 一致。不能靠 BCI 顺序或“join 看起来相同”替代 CFG 边证据。
- **Frame 不变宽：** 沿用 arm frame 的 boundary、scope、case/try/finally 约束；loop body 继续通过 `Frame::loop_body` 与父 scope 求交。若 header 是外层 loop 的 back/continue target，或不在父 scope 中，继续保持原有 stop/refusal。
- **预算与 Stop：** 所有新扫描沿用预算化入口，`region_at` 负责已有 recursion-depth/re-entry bound；边/owner扫描先计费并逐项 poll。遇到 `StopReason` 原样传播为 stopped outcome，不转成普通 fallback。若候选在部分递归后因 shape 不符而拒绝，应回滚该尝试新增的 visited 和其他持久 walk 状态，再走既有拒绝路径，不能留下半认领。
- **异常边：** prefix 和 loop body 仍经过原 `leaving_edge`、guard 和 exception-row accounting；不把异常/handler edge 纳入普通正常 join，也不新增 scope widening。没有现有 finally/try 证据时继续返回原有 refusal。

必要反例最多五类：

1. 另一个外部 predecessor 进入 prefix/header，或有绕过 if 的路径进入 loop，不能把共享循环块归到该 arm。
2. loop 有不同于 if join 的正常出口、无法唯一确定 exit，或自然循环不是当前 prover 支持的结构，必须保持 fallback。
3. header 是 enclosing loop 的 back/continue target、已访问块，或落在外层 scope/boundary 外，不能作为新 loop 再开一次。
4. prefix/body 含未由已有 guard 证明的 exception/finally/handler 边，不能因循环 splice 而隐藏原拒绝。
5. 在 loop body 深层遇到取消、nested-depth 或预算上限时，必须为 Stop 且无部分 produced text/source map；shape 失败的普通 `Ok` 路径则完整回滚尝试状态。

## 最小诊断与永久测试入口

下一次临时诊断只需覆盖实际调用链，不需新 fixture：在 `report.rs` 调 `region::recover` 前后输出 method owner/name/descriptor 的 begin/end marker；在 one-arm `region_at(walk, arm_frame)` 返回后输出 branch BCI、walk start、arm run 的 Region 类型与 block BCIs、`arm_next`、boundary/scope/own_loop/loop_targets；在 `region_at_inner` 的 loop-header 分支输出 method 范围内的 start/current/prefix BCIs、是否走 `prefix.is_empty()`、返回 continuation 或是否进入 `loop_region`。若进入 loop proof，再输出最终 `Loop`/fallback、exit 与 next。把所有块号转成 BCI，避免 node index 混淆。这样可以分别验证“一臂实际 arm_next=13”“inner branch36 未进入”或否定这条静态候选链。诊断只作局部观察，不加入长期接口。

永久回归最省实体可落在现有 `crates/jarde-java/tests/p3_effectful_exits.rs`：它已有 arbitrary-class `recover_class_method_with_budget` helper，以及 nested/2-level if-loop join 测试（例如 `nested_effectful_exits_keep_the_inner_tail_in_its_if_arm`、`two_level_effectful_exits_share_only_the_inner_join`），适合固定一条来自 TestVariablesDefinitions2 的完整 class bytes 并断言恢复与 block/source ownership。新例要断言 `countEmpty(List)I` structured、无 bytecode fallback、null/空/混合元素计数行为的 Java class oracle已由基线另行验证，并核对 branch/null、setup、loop header/body、内层条件和 return 的 anchors 每块一次。不要只断言文本中出现 `for`，也不要扩成 enhanced-for 全族支持主张。

**范围说明：** 本文件仅为源码架构草案。仍需临时诊断证明走到哪一个 early return；没有产品或 OpenSpec 变更，也没有运行 Git、Rust/JDK、JADX 或 CLI。
