# OpenSpec 规划入口

P0/P1 及 P1 验证维护已归档。本轮算法复核固定在 `eac3759`，并纳入随后 `955d7f3`/`823173b` 的 5.2 验收增量（2026-09-19）。`layer-jarde-crates` 已完成 7/7，尚未归档；P2 已完成至 5.2，共 25 项。3.4b 返回地址证明、3.5 规范化、Frame、初始化、SSA 与 `analyze_method` CLI 均已交付。本轮新增异常状态传播修正 4.2b 和派生存储计费修正 4.3b 后为 **25/29**；5.3/5.4 仍未完成。P3–P5 未实施。详见 [当前复核](changes/p2-jvm-ir/verification.md#review-2026-09-19-ir)；规划完整或历史 CI 通过不代表当前出口已满足。

- [架构基线](../JVM_Rust_Engine_Final_Architecture.md)：产品目标、I1–I12、模型与管线。
- [阶段路线](roadmap.md)：阶段依赖、范围与进入/出口门槛。
- [依赖选型](dependencies.md)：可用库、版本/许可证、复用决策与准入测试。
- [验收映射](acceptance.md)：A01–A18 的负责阶段及需要保留的验证证据。

| 阶段 | 变更文档 | 交付目标 |
| --- | --- | --- |
| P0（已归档） | [2026-09-17-establish-p0-foundation](changes/archive/2026-09-17-establish-p0-foundation/proposal.md) | 有界快照、物理 locator、Header、共享指令解码和结果契约 |
| P1（已归档） | [2026-09-17-p1-query-xref](changes/archive/2026-09-17-p1-query-xref/proposal.md) | 独立 X0/X1、metadata/resource/bootstrap、nested/MR/Boot 和分页 |
| 分层（7/7，待归档） | [layer-jarde-crates](changes/layer-jarde-crates/proposal.md) | reader/query/jvm 编译边界、门面与集成门禁已验收 |
| P2 | [p2-jvm-ir](changes/p2-jvm-ir/proposal.md) | Demand Resolver、raw/canonical CFG、legacy normalization、Frame/SSA 与降级 |
| P3 | [p3-java8-recovery](changes/p3-java8-recovery/proposal.md) | Java 8 高频恢复、命名、Java 输出和 source maps |
| P4 | [p4-modern-semantics](changes/p4-modern-semantics/proposal.md) | 现代语义、RuntimeMatrix、X2/X3 深度与框架插件 |
| P5 | [p5-measured-optimization](changes/p5-measured-optimization/proposal.md) | 按实测决定的缓存、并行、索引与性能回归 |

本仓库使用 OpenSpec 1.11.0 的 `spec-driven` schema。`changes/*/specs` 是拟议规格；`specs/` 当前包含 P0 归档的 `analysis-contracts`、`artifact-snapshots`、`classfile-inspection` 与 P1 归档同步的 `artifact-views`、`query-api`、`structural-xref` 六份主规格。CLI 的 `isPlanningComplete` 只表示规划工件齐全，不能解释为实现完成；实施状态以 tasks、代码和实际验收记录为准。

```sh
openspec list
openspec status --change p2-jvm-ir
openspec validate --all --strict --no-interactive
```

P0 归档记录及最终验证位于 `changes/archive/2026-09-17-establish-p0-foundation/`，P1 位于 `changes/archive/2026-09-17-p1-query-xref/`（两者都按各自 tasks 逐项实现并有 verification 记录，最终候选 CI 通过后才归档并同步主规格）。当前继续 `p2-jvm-ir`，执行顺序与闸口见 [阶段路线](roadmap.md) 和 [任务清单](changes/p2-jvm-ir/tasks.md)。已有 `.agents/skills` 为 OpenSpec 生成的工作流文档，不属于引擎代码；当前不发布 crate。
