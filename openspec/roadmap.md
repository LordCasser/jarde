# JVM Rust Engine 阶段路线

P0/P1 及 P1 验证维护已归档。P2 的 0.x、1.x、2.x、3.1–3.4 有交付记录；本轮新增未完成的 3.4b 后为 **18/27**。reader/query/jvm 与根门面已落地，`layer-jarde-crates` 仍需集成与门禁收尾。P3–P5 未实施。以 `35a779d` 为算法反例固定基线，并复核随后 `724bf1b`/`3646a97` 的门面/门禁增量，既有交付与新发现分开记录：[当前复核](changes/p2-jvm-ir/verification.md#review-2026-09-18-layers)。规划工件齐全、历史 CI 或局部测试通过不代表整个阶段完成。

相关入口：[OpenSpec 规划入口](README.md)、[技术栈与依赖选型](dependencies.md)、[架构验收与阶段映射](acceptance.md)。

## 阶段依赖

```text
P0 establish-p0-foundation → P1 p1-query-xref → P2 p2-jvm-ir → P3 p3-java8-recovery → P4 p4-modern-semantics
                                  │                │                 │                    │
                                  └────────────────┴─────────────────┴────────────────────┴──→ P5 p5-measured-optimization
```

`layer-jarde-crates` 插在 P2 3.4 与后续 IR 之间；主体迁移已经完成，当前只收尾，不重复执行拆包。新发现的 3.4b 属于 P2 语义修正，拆包验收后独立执行，随后才能进入 3.5。

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

## 当前执行顺序（2026-09-18 拆包后复核）

| 顺序 | 范围 | 交接门槛 |
| --- | --- | --- |
| 已完成记录 | P2 0.x、1.x、2.x、3.1–3.4；layer 1.1/1.2/2.1/2.2 | 保留原 commit/测试记录；历史 Approve 不代表 3.4b 新反例通过 |
| layer 3.1–3.3 | 五包主体已存在；补独立 query consumer、完整 A17/依赖闭包与 CLI/examples/fuzz/CI 收口证据 | 同一候选的门禁和反例有证据；核对既有 operation，不增加方法 CLI 或修语义 |
| P2 3.4b | returnAddress 来源、普通引用/丢弃/非法 aload、内层覆盖与异常状态 | 四个非法字节串不能完成上下文；合法直线/共享/嵌套/历史 finally、预算/取消回归成立 |
| P2 3.5 | driver 传递真实 CallContexts；有界克隆与 CanonicalCFG | origin/异常顺序/克隆界及 fallback 可复核；不把截断前缀标成完整 IR |
| P2 4.1 → 4.2 → 4.3 | Frame/Top/category-2 → 初始化别名/逐 throw-site → SSA/phi | 独立小图对照、后置定义/回边、预算与最后有效阶段有证据；不宣称 verifier 已执行 |
| P2 5.x | 库接通剩余阶段、方法 CLI、构造计数、golden/fuzz、文档及完整门禁 | `representation=Bytecode`；精确提交与 CI，整体出口通过后归档 |
| P3 1.1 → 1.2 → 1.3 | 只读 JVM 输入契约、普通控制流到 Java、最小命名/source map | 随真实实现创建 jarde-java；先得到可用方法闭环，再扩展语法糖 |

P4/P5 保留现有边界；P5 仍按所选稳定契约和测量进入，不作为正确性修正的前置。重复物化、query 游标/坐标等债务继续独立处理，不为本轮新增 common/core 或跨项目中端包。
