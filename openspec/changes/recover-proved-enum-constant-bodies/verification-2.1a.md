# 2.1a Root 验收

`may_capture_group_code` 仅去掉抽象类的便宜采集排除；Java 8、enum 身份、`java/lang/Enum` 父类与恰好两个常量字段的门仍在。`class_source_with_evidence` 仍通过同一次 prepared member IR 调用 `capture_method_code`，没有另建扫描或从已发射文本回推事实。`prove_group` 的抽象类成功门保持原样，所以此步不产生 `Proved` 或常量体投影。

Root 检查了改动和调用路径，独立运行 `cargo test --locked --target-dir /tmp/jarde-enum-body-2-1a-target -p jarde --lib enum_constants::tests --quiet`（28/28）及 `cargo test --locked --target-dir /tmp/jarde-enum-body-2-1a-target -p jarde --test class_source --quiet`（47/47）。`Op`、`Mixed`、`Plain` 的 Java 8 `-g`/`-g:none` 定向测试逐个核对 `<clinit>` 的 `new`/`invokespecial` BCI、owner 与构造描述符，确认 `Op.apply` 仍是物理无 Code 抽象声明，`Op` 的全方法引用可采集，三个类均仍拒绝组证明；预算耗尽和取消均停止。已有 Stage/Measure 正例在 enum 测试集中保持通过。`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-proved-enum-constant-bodies --strict` 通过。

这个验收只覆盖采集。子类关系、独占使用、构造委托和正文闭合仍由 2.1b 起的任务证明，不能据此声称已恢复常量专属类体。
