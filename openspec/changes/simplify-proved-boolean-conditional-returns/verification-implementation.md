# 实现侧验证：已证明的布尔条件返回

实现只扩展 `jarde-java` 的既有 conditional-value 提交边界。已有 `ConditionalValueProof` 成功后，builder 还要求 consumer 指令是 `ireturn`、方法描述符为 `Z`、条件呈现为 Boolean、两臂 SSA 值未被替换且分别由精确的 `iconst_1` / `iconst_0` 产生，才按真实臂极性输出 test 或 `!test`。其余值图仍建立原 `?:`；`return_expr` 仅让 `conditional_values` 中已提交且呈现 Boolean 的 Phi 直接进入 `Z`，普通 int 仍交给 `integer_low_bit_boolean`，原有 `boolean_value` 路径保持。新表达式携带条件根节点、分支、producer、transfer、Phi consumer 与 return 的来源。

新增 `crates/jarde-java/tests/p3_proved_boolean_conditional_returns.rs`，直接读取冻结的 `InstanceOfMerge.class` 和 `BooleanMergeControls.class`，检查反极性 `0/1`、正极性 `1/0`、直接 `instanceof` 对照、一次调用及 BCI 4/7/10/11/14/15 来源。测试在内存副本中将正极性两臂改为 verifier 类型仍为 int 的 `2/3`，确认返回继续出现低位适配；输出预算耗尽和预取消都不发布文本或来源映射。

以下实现侧命令通过：

- `cargo test -p jarde-java --test p3_proved_boolean_conditional_returns --target-dir /tmp/jarde-boolean-conditional-target`：3/3。
- `cargo test -p jarde-java --test p3_conditional_values --target-dir /tmp/jarde-boolean-conditional-target`：2/2。
- `cargo test --test p3_integer_boolean_returns --target-dir /tmp/jarde-boolean-conditional-target`：2/2。
- `rustfmt --edition 2024 --check crates/jarde-java/src/build.rs crates/jarde-java/tests/p3_proved_boolean_conditional_returns.rs`：通过。

通过 `jarde-cli class-source --policy single-class --evidence all` 重建冻结的 `InstanceOfMerge` 和 `BooleanMergeControls` 完整类文本；两份 Jarde 输出均经 `javac --release 8 -g:none -Xlint:-options` 编译。用冻结 Runner 在 `java -Xverify:all` 下运行原 class、JADX 全类和 Jarde 重编类，`InstanceOfMerge` 八行及 controls 八行逐字一致；正反极性各只求值一次。Jarde 目标返回文本为 test / `!test`，不含 `% 2`。

全仓 `cargo fmt --all -- --check` 当前被并行 typed-catch 工作树中的 `tests/p3_typed_catch_boundary_return.rs` 格式差异阻断；本轮未改该文件。额外 Phi consumer 与外部入口的专用改码负例留给 root 的独立验收补齐；投影仍以未修改的 `prove_conditional_value` 成功为前提，该 proof 已要求唯一 Phi consumer 并闭合双臂物理入口。
