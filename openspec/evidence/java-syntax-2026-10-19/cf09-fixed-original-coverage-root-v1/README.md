# CF-09 固定原件覆盖历史（v4）

本目录的v4/current-status.json是历史收据，不是最新完成状态。2026-10-19当前固定13原件为9接受、4待完整基线；Loop4/5产品412245d51完整验收见[最新控制流账本](../../jadx-feature-inventory-2026-09-27/control-flow.md)及[交接入口](../../../../handoff.md)。下面保留v4时点原文，不覆盖冻结JSON。

本包保留 v1 固定清单、加入 v2/v3 状态演进，并把 v1 的 `README.md` 与 `review.json` 原样复制到 `prior-inventory/`。v1 README 里的状态描述是历史记录；固定 13 项源文件清单及原始路径、大小、SHA-256 见该目录的 `review.json`。本版 `current-status.json` 也内嵌完整清单，附 v4 新收据的现场 pin。

ContinueInLoop 的 current-product 050 原生门与自身 CI 已 root 接受：product SHA `050cedbd255ffaddf9e51acf8d2ad52fe573519b`，CLI SHA-256 `6c83ec82cd4c6caee86c08e7fefddc2fea675952f936c0e4aa3f295ed9be7fc6`，CI run `39193571868`。四个 job、52 个官方 step ZIP；native 390 targets / 102 batches / 3802 passed、0 failed、98 ignored、57 exact gates；CI 每 seed 为 3802/0/98、102 exact gates，3900 个具名结果一致。这个 current-product 接受只覆盖 ContinueInLoop，不能推成其他固定原件的旧矩阵也在 050 上重跑。

13 项中，BreakWithLabel、ContinueInLoop、BreakInLoop 有已接受的完整原件对照；DoWhileBreak 有已接受的完整原件基线，但保留 16 个 Jarde 整类编译失败和 0 个 Jarde VM 运行。其同轮 IR 探针已归档并接受，定位到 block 13 的 `ArmsDoNotMeet`。当前 change `recover-proved-post-test-loop-break-prefixes` 的 tasks 文件显示 1.1/1.2 已完成，2.1/2.2/3.1/3.2 尚待完成；因此 DoWhileBreak 仍是当前工作优先项，TestBreakInComplexIf 排在其后。另有九项缺少已接受的完整原件对照，名称列在状态 JSON 中。

`v4_persistent_evidence_pins` 包含 Continue 的产品/CI审查，以及 DoWhileBreak 原件归档、同轮 IR 持久归档、inventory 和 root-review 的大小、SHA-256、mode。没有执行任何构建或运行；本包仅整理现有已接受收据。
