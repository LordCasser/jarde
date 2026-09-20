# OpenSpec 规划入口

P0–P5 与分层已按各阶段范围归档：P2 29/29、分层 7/7、P3 12/12、P4 10/10、P5 10/10；主规格现为 **18 份**。2026-09-20 对 `cd6f2f0` 的独立复核确认的两条 P1——嵌套表达式旧值重算（P3-R8）与字段生产者在 fallback 中丢失（P3-R9）——已由 `close-recovery-correctness-gaps`（4/4，`fd0aae8`）关闭并归档，历史归档记录保留。见 [收尾验证](changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。

- [架构基线](../JVM_Rust_Engine_Final_Architecture.md)：目标、I1–I12、模型与管线。
- [当前路线](roadmap.md)：已完成范围、正确性收尾和后续覆盖边界。
- [依赖选型](dependencies.md)：复用决策与准入要求。
- [验收映射](acceptance.md)：A01–A18 的适用范围与证据。

| 阶段 | 记录 | 当前状态 |
| --- | --- | --- |
| P0 | [establish-p0-foundation](changes/archive/2026-09-17-establish-p0-foundation/proposal.md) | 已归档；reader、快照、身份与预算 |
| P1 | [p1-query-xref](changes/archive/2026-09-17-p1-query-xref/proposal.md) | 已归档；X0/X1 与物理视图 |
| 分层 | [layer-jarde-crates](changes/archive/2026-09-19-layer-jarde-crates/proposal.md) | 7/7，`7a5f994` 归档 |
| P2 | [p2-jvm-ir](changes/archive/2026-09-19-p2-jvm-ir/proposal.md) | 29/29，`7a5f994` 归档 |
| P3 | [p3-java8-recovery](changes/archive/2026-09-20-p3-java8-recovery/proposal.md) | 12/12，`250fe1f` 归档 |
| P4 | [p4-modern-semantics](changes/archive/2026-09-20-p4-modern-semantics/proposal.md) | 10/10，`88416ab` 归档；结构/推断，不是完整现代源码恢复 |
| P5 | [p5-measured-optimization](changes/archive/2026-09-20-p5-measured-optimization/proposal.md) | 10/10，`cd6f2f0` 归档；cache 默认 off，性能阈值未定 |
| 收尾 | [close-recovery-correctness-gaps](changes/archive/2026-09-20-close-recovery-correctness-gaps/proposal.md) | 4/4，`fd0aae8` 归档；R8/R9 关闭，反例与正向对照进入永久语料 |
| Benchmark 批次 1/4 | [expose-recovery-content](changes/archive/2026-09-20-expose-recovery-content/proposal.md) | 4/4，`7d095ce` 归档；`RecoveryReport.content` 三值闭合 |
| Benchmark 批次 2/4 | [carry-declaring-class-evidence](changes/archive/2026-09-20-carry-declaring-class-evidence/proposal.md) | 4/4，`618de49` 归档；driver class name/flags 同源交接 |
| Benchmark 批次 3/4 | [bound-container-lookup](changes/archive/2026-09-20-bound-container-lookup/proposal.md) | 8/8，`b22ea04` 归档；定向 container 访问与有界复用 |
| Benchmark 批次 4/4 | [bind-prefixed-load-roots](changes/archive/2026-09-20-bind-prefixed-load-roots/proposal.md) | 6/6，`fbcf06b` 归档；container+prefix root 与 CLI 树枚举 |
| 性能专项 | [optimize-demand-workloads](changes/optimize-demand-workloads/proposal.md) | 22 项规划；以已归档的 `bound-container-lookup` 为首项实施 change，其余按实验准入 |
| 易用性 1/3 | [add-artifact-navigation](changes/archive/2026-09-20-add-artifact-navigation/proposal.md) | 8/8，`e0c83b6` 归档；候选/确认两级列举与身份交接 |
| 易用性 2/3 | [add-task-oriented-operations](changes/archive/2026-09-20-add-task-oriented-operations/proposal.md) | 11/11，`2428752` 归档；目标选择、阶段调度、预算与环境策略 |
| 易用性 3/3 | [add-task-oriented-cli](changes/add-task-oriented-cli/proposal.md) | 9 项规划；薄命令、text/JSON、显式退出状态 |

本仓库使用 OpenSpec 1.11.0 的 `spec-driven` schema。`specs/` 包含 P0–P5 的 18 份主规格；收尾 change 的两份 delta 已随归档并入 `java8-recovery` 与 `recovery-validation`（各保留原有 scenario，并各补两条反例/验收 scenario）。`isPlanningComplete`（旧字段 `isComplete`）只表示规划工件齐全，实施以 tasks、代码和验证为准。

```sh
openspec list
openspec validate --all --strict --no-interactive
openspec validate --archived --strict --no-interactive
```

A15/A18 的已实现路径通过；不存在的 index/parallel/merged 为不适用，不因此重做 P5。R8/R9 已关闭并归档；剩余的是单独记录的覆盖边界（MethodParameters、类级事实、handler 根策略、现代源码输出、容量与规模语料），需要时各自开 change。当前不发布 crate。
