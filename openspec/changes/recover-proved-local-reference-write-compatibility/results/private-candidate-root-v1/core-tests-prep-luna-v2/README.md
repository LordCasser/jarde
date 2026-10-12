# 私有 core 对抗测试候选 v2

本版本以 root 格式化后的当前共享 `build.rs` 为基底，仅修正 core v1 测试身份定位。候选没有写入共享仓库，也未运行构建或测试。

- 基底 SHA-256：见 `build.rs.base`
- 候选 SHA-256：见 `build.rs`
- 差异：`build.tests.patch`
- 唯一行为变化：`local_reference_write_test_case` 接收预期 LVT 名称，从真实 `Code` debug table 找唯一记录，以该记录的 `slot/start` 调用 `reuse.variable_at`，再要求映射的 `LocalVariable` 确实出现在 `slot_uses` 中且有访问。失败信息包含 method、descriptor、debug 名称、slot、range 与 identity。

测试不再用 `NameTable.text(variable) == "target"` 推断 identity。没有唯一 LVT 记录、无 reuse 归属或无实际 uses 时直接失败，不猜 slot、identity 或访问。原有四个 core 测试、production 内容及公开 late-stop 草稿均未在本候选中重写。
