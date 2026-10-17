# Java 8 recovery：保存返回值后的嵌套副作用清理

## ADDED Requirements

### Requirement: Recover a proved return through nested finally cleanup

引擎 SHALL 仅在 Facts/SSA/effects 证明返回值在 cleanup 前已计算并保存、所有 cleanup 操作与出口恰好归属一次、每条物理异常清理路径都保留异常身份与 Java finally 覆盖顺序时，呈现有界的嵌套 `try/finally` 清理。

#### Scenario: 正常嵌套清理后保留返回值

- **WHEN** 支持的方法计算 `10 / n`，且两个嵌套清理调用都正常完成
- **THEN** 恢复的方法只求值一次返回表达式，按顺序各执行一次 `in`、`out`，并返回先前保存的整数

#### Scenario: 成功清理后保留除法异常

- **WHEN** 整数除零抛错且两个嵌套清理调用都正常完成
- **THEN** 两个调用各执行一次，并重新抛出同一个原始算术异常值

#### Scenario: 内层清理异常仍执行外层清理

- **WHEN** `in` 在 pending return 或算术异常期间抛出已知 RuntimeException 或 Error
- **THEN** `out` 仍执行一次；若 `out` 正常完成，则逃逸的是 `in` 异常

#### Scenario: 外层清理异常覆盖待完成操作

- **WHEN** `out` 在待返回、算术异常或 `in` 异常期间抛错
- **THEN** 逃逸的是确切的 `out` 异常对象

#### Scenario: 完整完成矩阵一致

- **WHEN** 独立目标以 `n=非零/零` 调用，且 `in`、`out` 分别配置为无异常、RuntimeException 或 Error
- **THEN** 每个 `n` 的九种组合均保留正确返回或逃逸完成，注入异常的对象 identity 正确，且恰好按顺序执行一次 `in` 和 `out`

#### Scenario: 缺少证明时保留拒绝

- **WHEN** 无法证明 handler edge、throw effect、值身份、cleanup 顺序、block ownership 或嵌套 Region，或分析因 budget/cancellation/deadline 停止
- **THEN** 引擎保留现有 fallback/refusal，不声称恢复了 nested-finally

## Explicitly outside this requirement

finally 内写出的 return/throw、合成的 helper-based primary-exception 方法、以及没有 return 路径的仅异常完成均不属于本变更。本要求只适用于首轮 Facts/SSA 探针支持的有界形状，不推广为任意 catch-all table。
