# Constructor-conditional workspace acceptor candidate

这是独立的私有验收器候选，供 root 审核。它只接受已完成且 `status=passed` 的 execution receipt；准备过程没有运行 acceptor、runner、Cargo/Rust/JDK/JADX 或 Git。

`accept_workspace.py` 复用已固定 SHA 的 committed CI base 和 conditional helper，独立解析 metadata target、raw test blocks、命名 outcomes、summaries、Cargo argv 与运行 target 归属。它会核验 4727 个动态 source pins、361 个 target 恰好执行一次、95 个批次、98 条命令以及 196 个 raw streams。12 个 required tests 按当前 runner 的完整名称逐一核对并要求状态为 `ok`。

所有 workspace totals 均从本轮 raw stdout 重新解析，没有预填历史总数。验收还覆盖各命令的 argv、exit、guard、资源、环境、raw SHA，fresh target/free-space preflight、completed executable removal receipts、最终 guarded `cargo clean` 和 target 消失。

默认验收清单会新建在 `/private/tmp/jarde-constructor-conditional-workspace-root-v1/local-workspace-acceptance-root-v1.json`；若文件已存在则拒绝覆盖。候选文件和输入哈希见 `pins.json`，字段与边界审阅见 `review.md`。
