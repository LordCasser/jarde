# JVM Rust Engine 阶段路线

本路线描述 OpenSpec changes 的依赖、交付边界和验收门槛。P0、P1 与 P1 验证维护均已完成并归档。P2 已交付 1.x、2.x、3.1–3.3，3.4 有未提交候选；2026-09-18 review 确认基础行为缺口，当前先执行 p2-jvm-ir 的 0.1–0.3 修正，再完成 3.4，不能直接进入 3.5。任务为 11/23（原 11 项完成记录保留，新修正未完成），证据见 [本轮复核](changes/p2-jvm-ir/verification.md#2026-09-18-当前工作区复核)。P3/P4/P5 未实施。规划工件齐全、局部测试或前一提交 CI 通过均不代表阶段整体完成。

相关入口：[OpenSpec 规划入口](README.md)、[技术栈与依赖选型](dependencies.md)、[架构验收与阶段映射](acceptance.md)。

## 阶段依赖

```text
P0 establish-p0-foundation → P1 p1-query-xref → P2 p2-jvm-ir → P3 p3-java8-recovery → P4 p4-modern-semantics
                                  │                │                 │                    │
                                  └────────────────┴─────────────────┴────────────────────┴──→ P5 p5-measured-optimization
```

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

## P2 当前执行顺序（2026-09-18 修订）

| 顺序 | 范围 | 交接门槛 |
| --- | --- | --- |
| 0.1 | resolver 的 initiating/defining loader、caller/driver 绑定、声明与 dispatch 身份 | parent Owner/双 Base 反例与 2.x 回归通过 |
| 0.2 | reader 的 wide effective opcode、数组/调用操作数，CFG/effects 同步 | 普通/宽化、51+ ret、数组维数、P0/P1 对照通过 |
| 0.3 | 调用上下文存储预算、Effects requires、未来 pass 预算声明 | 零/恰好/超限、取消、stale 反例通过，不新增后续算法 |
| 3.4 | 返回地址值流及 handler 中的上下文/locals | 错 ret local 不再 Established、异常写入不漏，真实 finally 与独立复核通过 |
| 3.5 → 4.x | 有界规范化 → Frame → 初始化/异常状态 → SSA | Top、全别名初始化、逐 throw-site 输入和最后有效阶段均有证据 |
| 5.x | 库/CLI、读取/构造计数、golden/fuzz、文档/全门禁 | 精确提交 CI 与各片复核均完成，才归档并解锁 P3 |

不重做已准入的依赖选型，不把 P5 索引、并行或缓存混入正确性修正。本轮只修改 OpenSpec 与路线；保留已写代码，后续按修订任务实施。
