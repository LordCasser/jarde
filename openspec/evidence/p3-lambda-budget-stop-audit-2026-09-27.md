# P3 lambda IR budget 停止点审计

审计分支从 `dc35268d` 创建。两项回归测试在 `2ad29cee` 首次加入 `tests/p3_lambda_adaptation.rs` 时已存在；在该提交、`94879dac` 及当前 `dc35268d` 上分别定向重跑，均因找不到测试指定的停止点而失败。因此 EM11、EM12、EM20、CF02 与 CF03 近期改动都不是这两项失败的引入者。提交范围内的生产代码与两项测试之间存在更早的预算契约/断言不一致，当前提交仅记录证据，没有改生产代码或测试。

在 `2ad29cee` 上对测试的 IR 限额循环作临时诊断（改动未保留）后，`captured_adapter_budget_stops_inside_nested_value_rendering` 在 `ir_items=85` 确实停止于 `IrItems`、BCI 5；此时已计 `ir_items=84`，输出字节预算使用值低于完整运行，但 `analysis_steps=148`，而完整运行是 `157`。测试唯一错过该实测停止点的条件是要求限额运行的 `analysis_steps` 等于完整运行。预算停止会提前结束后续工作，因此不能要求停止运行已经支付完整运行的分析步数。这里是测试谓词过严，不是停止实现缺少 BCI 5 的 IR 计费。

`adaptation_budget_stops_and_precancellation_do_not_publish_a_partial_body` 选取的 `strings()` 返回 `LambdaAdaptationSupport::pick` 方法引用。它不是带 SAM 参数的 lambda AST。`build.rs::lambda_expr` 仅在 `LambdaForm::Lambda` 分支创建并计费 adapter 的 Local/Cast 节点；方法引用不经过该路径。因此测试注释要求在该方法引用的 BCI 0 停在“两节点 Local/Cast adapter”与真实形状不符。旧提交的限额扫查虽能遇到 BCI 0 的其他 `IrItems` 停止，但不能证明不存在的 adapter 批次。该测试随后才会执行的预取消断言也因前面的 `adapter_stop` 失败而无法到达。

当前 `94879dac` 的失败摘要为 `strings` `ir_items=39 / analysis_steps=133`、captured `ir_items=96 / analysis_steps=160`；在 `dc35268d` 是同样的两处断言失败。较早 `2ad29cee` 的对应摘要为 `39 / 132` 与 `96 / 157`，停止行为与生产实现已在该基线存在。完整定向命令：

```sh
cargo test --test p3_lambda_adaptation adaptation_budget_stops_and_precancellation_do_not_publish_a_partial_body -- --exact
cargo test --test p3_lambda_adaptation captured_adapter_budget_stops_inside_nested_value_rendering -- --exact
```

后续若要恢复这两项检查，应把方法引用排除在 Local/Cast adapter 批次断言之外，并让 captured 测试只断言到停止点之前实际支付的分析工作；预取消行为需放在独立、可到达的断言中。此审计不重写或放宽固定测试。
