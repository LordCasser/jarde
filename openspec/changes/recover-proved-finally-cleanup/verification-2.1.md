# 2.1 局部证明验收（2026-09-24）

`guard::prove_finally_copy` 只为直线型正常返回和 catch-all 重抛生成私有证书，目前不认领 `Plan`，因此 Java 正文仍以原有 `jre_guard_finally_copy` 引用结束；2.2–3.2 尚未实现。证书比较两份 cleanup 的实际 `Operation`、成员与常量，并把每个栈输入归一到各自副本内的生产者序号。正常副本必须从异常表半开范围 `end_pc` 开始，正常和 handler 的每条 cleanup 指令都在该范围外；两份副本及返回/重抛链的局部 SSA 与 CFG 出口必须闭合。额外入口会分裂 canonical block，正常 cleanup+返回只接受单个直线块，handler 必须在单个无正常后继的块内。

Root 审阅了 `guard.rs` 的边界后，要求补上 normal cleanup 中段外部入口检查；实现将正常副本限制为单 canonical block，现有 `NormalFlowView` 入边检查也拒绝非保护块直接进入正常副本或 handler。定向八项单测经 root 使用独立命令重跑全部通过：原始 `ImplicitCleanup` 与快照正例有证书；范围扩围（`end_pc` 从 20 到 23）、参数/调用目标差异、范围缺口、竞争 handler、重复清理、额外消费者、预算和取消分别拒绝或显式停止。原始正例 class SHA-256 `924437916dc278eefe3b83cdcf3bad14bfb3cf8b9e44c027786a89f0712247b6`，扩围负例 `c5407d3f29b135818f003e24b218b5e4b7348d261665f0c71605471a7492935e`。

验收命令：`CARGO_TARGET_DIR=/private/tmp/jarde-string-switch-target cargo test --locked --offline -p jarde-java --lib finally_copy_tests` 为 8/8；同一 target 的 `cargo test --locked --offline --test p3_guard` 为 13/13，包含原有 finally 引用边界。`cargo test --locked --offline -p jarde-java --lib` 为 135/135。`openspec validate recover-proved-finally-cleanup --strict` 通过。此结果仅验收局部证明，不把尚未写出的 `finally {}` 或原/JADX/Jarde 完整类对照计为完成。
