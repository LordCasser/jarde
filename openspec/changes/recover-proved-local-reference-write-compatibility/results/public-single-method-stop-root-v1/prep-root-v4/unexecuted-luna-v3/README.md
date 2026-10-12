# 私有 public 单方法 late-stop 候选 v3

本候选以共享集成测试当前快照为基底，用正确的单方法总预算替换 v2 的 class-source late-stop 测试。没有写共享仓库，也没有运行工具链。v2 目录及 root 已执行的 v2 raw 保持原样。

- 基底 SHA-256：`d5a2c7e8014d04a036220eb18e34b06dc6703f8bbd12ac99308f38ce130ce59d`
- 候选 SHA-256：见 `p3_local_reference_write_compatibility.rs`
- unified diff：`public-method-stop.patch`
- 测试：`measured_public_single_method_late_stop_does_not_publish_partial_target_artifacts`

对 debug/none × javac8/23 的四个 `CF11OuterAssignmentTarget` 变体，先用 class-source complete report 获取准确的 `noCastAssign(Ljava/util/List;)Ljava/lang/Object;` `PhysicalMethodId`。随后以同一 snapshot 及原 `ClassSourceRequest.environment` 构建 `MethodAnalysisRequest`，对该 identity 通过 `recover_method_with_evidence` 执行完整单方法请求。完整基线费用读取外部 `Budget::usage().analysis_steps`，不使用 `RecoveryReport` 内的 usage 作输入上限；随后以实测总量减一作为第二次单方法请求上限，并再次读取外部 usage。候选在基线日志中并列打印完整 body usage 与外部总量，但只以外部 `Budget::usage()` 作为限额来源，不假设二者相同。

受限调用要求 analysis 和 recovery 均报告 `AnalysisSteps` partial，method identity 不变，stop 有位置，外部实际 usage 不超过输入限制，目标 body 未产出文本、map、news 或 content。每个变体打印完整外部总量、limit、受限外部总量、body partial usage 和真实 StopReason。

该 public API 测试不声称 stop 停在最终 local Return consumer；精确 consumer 门由 core 测试覆盖。未运行测试；四个变体的单方法 usage 与 stop 归属仍待 root guarded 验收。
