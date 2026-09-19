# JVM Rust Engine 阶段路线

P0/P1 及 P1 验证维护已归档。本轮算法复核固定在 `eac3759`，并纳入随后 `955d7f3`/`823173b` 的 5.2 验收增量（2026-09-19）。`layer-jarde-crates` 已完成 7/7，尚未归档；P2 已完成至 5.2，共 25 项。3.4b 返回地址证明、3.5 规范化、Frame、初始化、SSA 与 `analyze_method` CLI 均已交付。本轮新增异常状态传播修正 4.2b 和派生存储计费修正 4.3b 后为 **25/29**；5.3/5.4 仍未完成。P3–P5 未实施。详见 [当前复核](changes/p2-jvm-ir/verification.md#review-2026-09-19-ir)；规划完整或历史 CI 通过不代表当前出口已满足。

相关入口：[OpenSpec 规划入口](README.md)、[技术栈与依赖选型](dependencies.md)、[架构验收与阶段映射](acceptance.md)。

## 阶段依赖

```text
P0 establish-p0-foundation → P1 p1-query-xref → P2 p2-jvm-ir → P3 p3-java8-recovery → P4 p4-modern-semantics
                                  │                │                 │                    │
                                  └────────────────┴─────────────────┴────────────────────┴──→ P5 p5-measured-optimization
```

`layer-jarde-crates` 已在 P2 3.4 后完成，随后 3.4b、3.5、4.x 与 5.1 均有交付。当前新增修正属于 P2 的异常语义与资源边界，不重开拆包或重做 canonical/SSA。

`p1-query-xref` 还直接消费 P0 的 snapshot/classfile/contract；`p4-modern-semantics` 需要 P1 的 views/query、P2 的 resolver/IR 和 P3 的 recovery。P5 选择一个或多个已有稳定结果契约作为优化目标，在被选阶段的真实基线稳定后即可进入，不要求先完成 P4；若优化跨阶段，再纳入所有受影响阶段的回归。

## 阶段边界和门槛

| 阶段 | OpenSpec change | 本阶段新增边界 | 主要验收 IDs | 进入条件 | 出口门槛 |
| --- | --- | --- | --- | --- | --- |
| P0 | `establish-p0-foundation` | bounded in-memory snapshot、CLASS/JAR/WAR 顶层 locator、Header/bytecode inspection、公共 coverage/diagnostics | P0 specs 与基础 reader/预算测试 | 架构基线和依赖评估完成后 | 快照不受源文件变化影响；顶层物理 entry、Header、BCI/attribute span 可复核；不宣称 X1/resolver/decompile |
| P1（已归档） | `p1-query-xref` | X0/X1 structural XRef、code/metadata/resource/bootstrap consumers；nested/MR/Boot 的物理/运行视图和分页 | A01–A08、A14、A17、A18 | P0 事实、identity、budget、JSON contract 稳定 | 不构建 CFG/SSA/Java AST；consumer 不漏报；PhysicalView 保留全部 entry；RuntimeView 显式选择；预算/取消永不伪造 Complete |
| P2（实施中） | `p2-jvm-ir` | Demand Resolver、Platform/LoadDomain providers、raw CFG、jsr/ret normalization、CanonicalCFG、Frame/SSA、effects、Conservative fallback | A09、A10、A11、A13、A14、A16、A17 | P1 query/view 及 P0 bytecode facts 可按需读取 | Resolver 状态可区分；IR pass 依赖/invalidation 可检查；单方法不物化无关 Body；verification 与静态分析分开；失败成员隔离 |
| P3 | `p3-java8-recovery` | Java 8/历史恢复模式、确定性命名、source map、representation/quality/compile_status/semantic_validation/verification | A09、A10、A12、A13、A16 + 第 20 节恢复矩阵 | P2 IR/Resolver/origin/effect contract 稳定 | 只有满足前置证据才 Structured；TWR/异常/副作用顺序可复核；X1 原始边保留；语料/重编译/受控行为结果只按 profile 记录 |
| P4 | `p4-modern-semantics` | module、nestmate、condy、modern concat、record/sealed、RuntimeMatrix、X2/X3、versioned query plugins | A04–A07、A12 + 53–71/preview registry tests | P1 views/query、P2 resolver/IR、P3 recovery status 可独立演进 | release-bound registry 和 preview 诊断；Unknown/OpenWorld 不被猜成唯一目标；Java 8 output conflict 明确；不把结构支持 Java 27 等同完整源码恢复 |
| P5 | `p5-measured-optimization` | 仅以实测决定的 query merge、并行、cache/index、退化优化和发布 gate | A14–A18 + 被选目标阶段适用回归 | 被选目标阶段的结果契约和真实基线稳定，不要求 P4 完成 | 每项优化有 direct baseline、完整 cache key、失效/回退、资源和正确性对照；无证据收益保持 disabled；不预设性能数字/新依赖 |

## 变更使用约束

- 每个阶段保持独立 OpenSpec change；`proposal.md` 的 capabilities 必须与 `specs/<capability>/spec.md` 一一对应。
- P0 的顶层 locator 与有界内存快照不在 P1 重做；P1 才增加 nested/MR/Boot 的递归候选、布局和 RuntimeView。
- 各阶段只有实际完成并验证的任务才能勾选；P0 状态以归档 tasks/verification 与主规格为准，P1 以归档记录为准，P2–P5 以 active change 的 `tasks.md` 与最新复核为准。本文描述依赖和门槛，不把规划完成解释为实现完成。
- 每个 change 的实施阶段归档前先执行对应 strict validation，再按架构验收 IDs 和真实语料验证；有行为 delta 时 archive 默认同步主 specs，不使用 `--skip-specs`。
- 优先复用 P0 已评估的 noak、rawzip、flate2 rust_backend、blake3、serde、thiserror、clap；后续库、持久 index 或并发方案先依 [选型准入](dependencies.md) 评估。JVM/JADX 不进入生产核心运行依赖，只可作为受控测试 oracle。

## 当前执行顺序（2026-09-19 IR 复核）

| 顺序 | 范围 | 交接门槛 |
| --- | --- | --- |
| 已完成记录 | layer 7/7；P2 0.x–5.2 已勾选的 25 项 | 保留各片提交与验证；新增问题单列 4.2b/4.3b，P2 当前 25/29 |
| 4.2b | 异常状态独立参与 Frame 固定点 | R9 的正常出口不变/throw-site locals 改变反例通过，SSA 不再收到陈旧 handler 状态 |
| 4.3b | Frame/SSA 派生存储增长前计费 | R10 多 throw-site × locals、发布复制、零/恰好/超限与取消均有证据 |
| 5.2（已验收） | `955d7f3`/`823173b` 的 5 例入口计数 | 无关 Body 为零、X0/X1 预算代理为零，结合源码/依赖守卫；后续真实 Region/AST 路径仍须检验 |
| 5.3 | 修正后的固定 replay、性质/oracle、方法分析 fuzz | 含异常回边、legacy clone、wide/switch 与资源乘积，不只测形状或不 panic |
| 5.4 | 文档、主规格、完整门禁与归档 | 精确候选与 CI；JDK 25 oracle、MSRV、两个 workspace supply-chain、规定时长 fuzz、strict 全通过 |
| P3 1.1 → 1.2 → 1.3 | 请求内只读 IR 交接、formatter、普通 Java 方法闭环 | P2 出口已过；消费真实 CFG/SSA/effect/origin，随真实实现创建 jarde-java |

5.1 已有 `analyze_method` CLI 与逐字段对照，不重复安排。报告是 Bytecode 分析摘要，私有 IR 尚未作为跨 crate 载荷交付；P3 1.1 必须补这个接缝。P4/P5 的既定范围保持不变，独立性能/查询债务不混入当前正确性修复。
