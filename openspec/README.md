# OpenSpec 规划入口

P0–P5 的 proposal、design、specs 和 tasks 已建立，当前已按 P0 tasks 开始实施；阶段完成度以任务勾选、实际代码和行为验收为准，在对应门槛满足前不发布引擎能力或性能承诺。

- [架构基线](../JVM_Rust_Engine_Final_Architecture.md)：产品目标、I1–I12、模型与管线。
- [阶段路线](roadmap.md)：阶段依赖、范围与进入/出口门槛。
- [依赖选型](dependencies.md)：可用库、版本/许可证、复用决策与准入测试。
- [验收映射](acceptance.md)：A01–A18 的负责阶段及需要保留的验证证据。

| 阶段 | 变更文档 | 交付目标 |
| --- | --- | --- |
| P0 | [establish-p0-foundation](changes/establish-p0-foundation/proposal.md) | 有界快照、物理 locator、Header、共享指令解码和结果契约 |
| P1 | [p1-query-xref](changes/p1-query-xref/proposal.md) | 独立 X0/X1、metadata/resource/bootstrap、nested/MR/Boot 和分页 |
| P2 | [p2-jvm-ir](changes/p2-jvm-ir/proposal.md) | Demand Resolver、raw/canonical CFG、legacy normalization、Frame/SSA 与降级 |
| P3 | [p3-java8-recovery](changes/p3-java8-recovery/proposal.md) | Java 8 高频恢复、命名、Java 输出和 source maps |
| P4 | [p4-modern-semantics](changes/p4-modern-semantics/proposal.md) | 现代语义、RuntimeMatrix、X2/X3 深度与框架插件 |
| P5 | [p5-measured-optimization](changes/p5-measured-optimization/proposal.md) | 按实测决定的缓存、并行、索引与性能回归 |

本仓库使用 OpenSpec 1.11.0 的 `spec-driven` schema。`changes/*/specs` 是拟议规格；`specs/` 当前为空，只有对应实现验收后才通过 archive 同步主规格。CLI 的 `isPlanningComplete` 表示规划工件齐全，不能解释为实现完成；实施状态以 tasks 和实际验收记录为准。

```sh
openspec list
openspec status --change establish-p0-foundation
openspec validate --all --strict --no-interactive
```

P0 正按 `establish-p0-foundation/tasks.md` 实施和验证；只有 3.x 出口任务全部满足后才 archive 并同步主规格。后续阶段进入实现时继续按对应 change 的 tasks 推进。已有 `.agents/skills` 为 OpenSpec 生成的工作流文档，不属于引擎代码；当前不发布。
