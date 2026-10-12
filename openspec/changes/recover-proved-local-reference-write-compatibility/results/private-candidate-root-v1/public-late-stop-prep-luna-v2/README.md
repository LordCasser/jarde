# 私有 public late-stop 测试候选 v2

基于当前公共集成测试快照，仅新增一个按四个 positive fixture 变体运行的公开 API 测试。没有写共享仓库，也没有运行工具链。

- 基底 SHA-256：见 `p3_local_reference_write_compatibility.rs.base`
- 候选 SHA-256：见 `p3_local_reference_write_compatibility.rs`
- unified diff：`public-late-stop.patch`
- 新测试：`measured_public_late_stop_does_not_publish_partial_target_artifacts_in_all_profiles`

测试遍历 `debug/javac8`、`debug/javac23`、`none/javac8`、`none/javac23` 的 `CF11OuterAssignmentTarget.noCastAssign(Ljava/util/List;)Ljava/lang/Object;`。每个变体先从完整 public body 的 `usage.analysis_steps` 实测费用，再以 `usage - 1` 作为唯一的受限 `analysis_steps` 输入预算。它核对 class 与目标 method 的 `AnalysisSteps` partial、准确物理 method identity、stop location，以及目标 body 未提交的 produced/text/source-map/news/content，并打印实际 limit、partial usage 和 stop。

该 public API 断言没有声称 stop 恰好落在最终 local Return consumer；测试注释指出精确 final-consumer 门由私有 core 测试覆盖。测试未运行；四个 profile/JDK 变体的 stop 所属与报告状态仍需 root 实跑验收。
