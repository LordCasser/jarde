# 实施验证

## 结果

最终 `Program.stmts` 上的私有回执逐节点、有预算地检查真实字段操作，`field@1` 的先验 claim 仍供 Builder 使用。`FieldRecord`、未发射理由 `jre_field_not_emitted` 和全方法摘要读取同一回执；原字段证明拒绝码保持不变。旧式 `field++/++field` 由 Builder 的形状证明和最终 `Return` 的 direct 更新 BCI、derived 读取 BCI 互证；`PostIncrement` 由语义 AST 的目标字段和更新节点判定。复合字段 `+=` 的隐含读取由字段赋值节点和其已证明读取来源判定。遍历覆盖全部现有语句及表达式分支，`Fallback` 的来源不算字段操作。

原 `ChainExtraBoundary.assign` BCI 30 已恢复为 Java 字段赋值，保持 `presented=true`。同方法的额外入口和异常边控制均为 `presented=false`，拒绝码为 `jre_field_not_emitted`，摘要为 0 presented / 1 refused，BCI 30 仍有 source-map 来源。essential、all、BCI 30..33 范围所得摘要相同；详细记录仅随请求选择。未修改 Region 的拒绝。

## 执行

- `CARGO_TARGET_DIR=/tmp/jarde-field-presentation-target cargo check -p jarde-java`：通过。
- 定向测试：`p3_field_presentations`、`p3_field_increment`、`p3_eval_context`、`p3_compound_lvalue_updates`、`p3_final_static`、`p3_postfix_lvalue_values`、`p3_region_fallback_origins`、`p3_region_owner_overlap`：全部非 ignored 用例通过。包含普通和嵌套读写、两种 field++、构造器、`<clinit>`、复合更新、预算/取消、来源与选择。
- `p3_accessor_edges::the_two_original_edges_survive_a_recovery_run_field_by_field`：通过，合成 accessor 字段不进入调用方法的 `field@1` 记录。同文件的 `a_bytecode_index_a_clone_repeats_still_names_the_member_it_is_in` 在共享工作树中失败：测试期望克隆子程序的 quoted BCI 重复，实测 `[0, 6, 11, 14]` 没有重复。该行为与本次字段回执无关，另案调查。
- `cargo test -p jarde-java --lib field::tests::`：5/5 通过。新增交错 BCI 的拒绝顺序单测；回执单测将 `AnalysisSteps` 限制停在 AST 遍历途中，得到 `Budget` 错误而无部分回执，预取消得到 `Cancelled` 错误，完整预算返回唯一字段 BCI 7。
- `cargo clippy -p jarde-java --lib`：完成，17 条现有告警均不在本次新增回执代码。`-D warnings` 因这些告警失败；不在本 change 混入修复。
- `openspec validate report-committed-field-presentations --strict`：通过。`git diff --check`：通过。

## 边界

这次回执只回答本方法 `field@1` 已证明且最终发射的字段指令；不重新判定合成 accessor 的 callee 字段，也不从文本或 source map 推导操作。其他规则是否存在同类“计划等于已发射”债务不在本次范围。Cargo 构建使用独立 target，并在任务结束时通过 `cargo clean` 清理。
