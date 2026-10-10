# prove_switch_fallthroughs Walker 计费与停止观测方案

## 结论

把计费/停止测试放在 crates/jarde-java/src/region.rs 现有私有 tests 模块，沿用 integrated 文件中 cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir 的搭建方式：用 reader 分析冻结的 javac 8 B-nonadjacent class，取真实 CanonicalCfg、NormalFlowView、SSA 和解码事实，构造真实 Walker，通过 Walker::switch_fallthroughs 传入完整 canonical edge iterator。不要把 prove_switch_fallthroughs 的 helper 测试当成计费测试；它不覆盖 Walker 包装层所取的边集。

现有 reader-backed 集成草稿 tests/p3_conditional_switch_boundary_rejection.rs 已覆盖正常 recover caller 的行为和输出门控。这里补的最小单测只回答 proof 真实 Walker 路径的 AnalysisSteps、StopReason 和取消；避免再建测试框架或接入另一个 reader fixture。

## 真实 phase 与停点

在 after/region.rs 中，定位行号会随合入变化；用下面的代码锚点查找，而不要把行号写进测试：

1. **Canonical edge collection**：prove_switch_fallthroughs 中初始化 incoming / outgoing 后的 for (from, kind, to) in edges。每行先 poll，再 charge 一个 AnalysisSteps，然后同时进入入边和出边表。
2. **每个 case 的 DAG walk**：for (from_bci, start) in targets 下的 while let Some((node, finishing)) = work.pop()。每个 work item charge 一步；每个 successor 的展开也独立 charge 一步。真实 B fixture 的多跳 36 -> 40 -> 67 会经过此处。
3. **Incoming closure 验证**：// Case entries are boundaries 注释之后。先有一笔基于闭包节点数的批量 charge；之后逐一校验 case-entry 与 closure node 的 canonical incoming rows，每组按 rows 数批量 charge。owner_by_node 的建立有 poll，但没有单独 charge。

三段的 Stop 都用 Some(switch_bci)，所以只从 StopReason 的 at 不能断言是哪段停下。私有临时观察时，在这三个锚点设断点或临时阶段标记，单步看当时的 budget.usage().analysis_steps、incoming/outgoing/closures；不要保留新增生产观测接口或永久 phase enum。

## 最小真实调用构造

从既有 conditional_switch_proof_tests.rs::prove_real_conditional_switch 复用真实 fixture facts 的准备逻辑，或把这部分收敛进 region 私有单测的局部 helper：

- 选正例 B-nonadjacent/ConditionalSwitchBoundaries.class，固定 class digest，并验证 reader 给出的完整 edge multiset、blocks 和空 clone path。其真实 switch 在 BCI 9；解码事实和 case group 从该 MethodIr 获取。
- 用 reader 实际 canonical rows，不复制、删减或合成 edge row。NormalFlowView 与 targets 从同一 canonical graph 得出。
- 以 cf07 测试中的 real_walker! 构造完整 Walker 字段，使用 crate::init::Sites::empty()、JAVA_8 和干净的 visited。
- 直接调用 walker.switch_fallthroughs(...)，这就是 switch_region 使用的真实 Walker 包装方法，且把完整 canonical edge iterator交给 proof。让既有 reader-backed recover 集成草稿继续负责验证从外部 recovery 入口到 switch 输出的连接。

若要观察 switch_region 到 wrapper 的调用链，可临时在 switch_region 调用 self.switch_fallthroughs 处设断点；不建议让预算测试通过整次 recover 来反推 proof 的限额，因为 prepare、guard/loop prepass 和 root walk 都会先花 AnalysisSteps，导致阈值与 proof 本身混在一起。

## 永久测试建议

一个测试内用每次全新的 Budget 重跑同一真实 Walker::switch_fallthroughs：

1. 用宽松的 analysis_steps 跑到完成，断言 map 为 Some({36: 67})，并从这次实际调用的 budget.usage().analysis_steps 记录 observed_steps。要求该观测非零即可；不写 helper 的逐步公式，也不硬编码总步数。
2. 用 analysis_steps = observed_steps - 1 重跑，断言返回 Err(StopReason::Budget { dimension: AnalysisSteps, limit, at })，其中 limit 从本次 budget.limits() 读取并与配置值相等，at 与真实解码出来的 switch BCI 相等。由于 proof 里存在批量 charge，不要断言 consumed 恰好等于 limit，也不要断言这次必定落在某个 phase；只要求实际观测到的完整成本少一步时 Walker 会 Stop。
3. 用新的 CancellationToken 与宽松预算构造 budget，在调用前取消 token；断言通过 Walker 返回 Cancelled { at: Some(observed_switch_bci) } 且 analysis_steps == 0。可以同时将 analysis_steps 设成零，验证取消在同一真实 poll 上优先于预算拒绝。此例证明 caller 的取消透传，不声称覆盖“运行中异步取消”。

动态的 observed_steps 和 BCI 来自同一真实调用/解码结果，避免硬编码 count 或从 helper 的 charge 语句重新计算。边界重跑的预期只比较 dimension、实际配置的 limit 和实际 switch BCI；不复制 helper 的记账细节。

## 审查边界

- 不改 repo/Cargo/工具链；本方案只建议 region.rs 私有测试和独立 recovery 集成的职责。
- helper 的 full-row 和 edge/DAG/incoming 合成拒绝测试继续检验证书边界；Walker 测试专注预算/Stop/取消；外部集成测试专注最终可见 recovery 行为。
- StopReason::Budget 的 written 对这里应为 0，但无需把与该目标无关的 output/stop 字段扩展成测试矩阵。

