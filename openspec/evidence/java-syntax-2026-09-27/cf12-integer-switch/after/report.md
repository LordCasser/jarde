# CF-12 实施后验收

固定输入仍为 `baseline/IntegerSwitchAudit.original.class`（SHA-256 `b1a6be1a8c0c3ecc517cae946e298711f1367b5735c683c69d4812c015f9857b`）；JADX 源码仍为原审计归档，并由 `replay-after.py` 检查其哈希。JADX `TestSwitchLabels` 的正例要求 `case CONST_ABC`、`return CONST_CDE`，禁用替换的反例要求 `case 2748`、`return 3294`，且私有嵌套字段不得被误用。本实施只采用当前完整类字段表中的唯一 `static final int ConstantValue`。

实施后的完整 Jarde 类源码在 `IntegerSwitchAudit.jarde.java`，其中 `labelConstant` 写出 `case LOW:`、`return HIGH;`；同一次物理方法报告以及单方法 `recover` 仍写 `case 2748:`、`return 3294;`。类报告的两个 `integer_constant_projections` 分别将源码中的 `LOW` 关联到原字段和 switch BCI 1，将 `HIGH` 关联到原字段和 literal BCI 20；原 `SwitchArm.keys` 与方法恢复来源没有更改。

运行 `PYTHONDONTWRITEBYTECODE=1 python3 replay-after.py --jarde <本分支 CLI>`：原源码、固定 JADX 完整类、Jarde 完整类各自经 `javac --release 8 -g:none` 编译，然后以 `java -Xverify:all` 执行同一 runner，三者 11 行逐行一致。命中/不命中、合并标签、fall-through、无 default 保值路径均在这 11 行内。`summary.json` 记录 CLI 与产物 SHA；各路径编译和运行日志在本目录。Jarde 重编 class 的 SHA 与基线数值源码重编 class 相同，说明这两个常量名只改变源拼写。

定向 Rust 测试覆盖重复值、参数遮蔽、非 final、long descriptor、无 `ConstantValue`、非法字段名、char selector、非 `I` 返回和非直接 return、截断字段表和最终源码预算拒绝。预算拒绝发生在物理 `labelConstant` 已恢复之后，类源码维持原数值，未发布部分 derived 范围。取消令牌在打开输入后、类请求开始前使选择不完整且不产生投影。相邻 switch 测试与库测试均通过；受限范围没有扩展到全局常量查找、一般数字替换或嵌套源码单元。
