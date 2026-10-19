# OpenSpec 入口

本仓库使用 OpenSpec 1.11.0 的 `spec-driven` schema，工件以中文编写，结构标题及 SHALL/MUST 保留英文。现有 [18 份主规格](specs/)；P0–P5 和分层已按当时范围归档，不能据此宣称全部 Java 语法恢复完成。

| 内容 | 入口 |
| --- | --- |
| 当前交付与下一步 | [handoff](../handoff.md) |
| Java 语法覆盖与执行顺序 | [JADX 71 单元账本](evidence/jadx-feature-inventory-2026-09-27/summary.md) |
| 当前实现的行为契约 | [主规格](specs/) |
| 各项实施与验收 | [changes](changes/) 中的 tasks、verification 或 results |
| 历史归档 | [archive](changes/archive/) |
| 架构、不变量与管线 | [架构基线](../JVM_Rust_Engine_Final_Architecture.md) |
| 依赖与验收要求 | [dependencies](dependencies.md)、[acceptance](acceptance.md) |
| 历史阶段设计与复核 | [roadmap](roadmap.md)、[completion-review](completion-review.md) |

按完整原件、JADX 和 Jarde 的同输入对照确认缺口，核实实际 IR 与所有权后再写最小 proposal/design/specs/tasks。实现完成须经过定向回归、完整对照和产品验收；静态计划、任务勾选或 `isPlanningComplete` 均不能替代执行证据。跨机器的私有绝对路径需从持久归档恢复并重新核对输入。

```sh
openspec list
openspec status --change <change-name>
openspec validate --all --strict --no-interactive
```

已完成 change 的证据可能仍被工具和冻结收据引用。归档时须按各 change 的引用关系处理，文档整理不批量搬迁 evidence、工具或规格。
