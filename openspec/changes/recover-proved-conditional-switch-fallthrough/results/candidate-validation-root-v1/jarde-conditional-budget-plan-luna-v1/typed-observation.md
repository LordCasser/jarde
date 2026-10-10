临时 typed proof 观测稿

只需把 typed-observation.patch 和 typed-observer-test.patch 临时应用到 live source，用 root 的一次性观察运行采样，然后完整还原：

- decide_types 中每个物理 write 的 charge 之前，ROOT_TYPED_WRITE 打印 use_.bci 与当前 AnalysisSteps usage。
- null_leading_reference_type 里唯一的 uses.len() 批量 charge 之前，ROOT_TYPED_NULL 打印 first.at 与当前 usage。
- cf12_proved_local_source_types.rs 临时 observer 用两个分开的、宽松 fresh recovery Budget 调用已有 recover_class_method_with_budget，分别读取 CHAR_CLASS / TestSwitch.test(Ljava/lang/String;)String 与 NULL_CLASS / TestSwitchNoDefault.test(I)V，并打印 outcome/text。

内部观察保留 use_.bci 和 first.at；这份草稿不改原永久预算 Stop 测试的 at，也没有保留测试 helper API、Usage 字段或 phase 字段。只写在 private tmp；未修改仓库或运行工具链。
