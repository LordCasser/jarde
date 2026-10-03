## Why

[构造器重排回归实证](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)：已验收切片 `recover-synthetic-ctor-super-order`（合并 daa4fb31）的重排判据未检查 super 目标是否可能构造期虚分派，对冻结反例 `anonymous-super-dispatch` 产生**静默行为回归**——原 class `observed=captured-value`/`visibleDuringSuper=true`，其呈现重编后 `observed=null`/`visibleDuringSuper=false`。重排前该产物不可编译（响亮失败），重排后可编译且行为不同（静默），违反 `recover-return-in-do-while-false` 已确立并验收的不变量。项目自 2026-09-25 已冻结该反例，`present-proved-java-structure` 的 2.10 明写"改序不损失任何效果"已被运行反例否定。

## What Changes

- 重排判据加前置条件：**super 目标为 `java/lang/Object.<init>()V`** 时才允许把 pre-super 合成字段存组移至 super 之后（Object 构造器不可能虚分派到用户代码，故移动无行为风险）；目标为任何其它类时不重排，保持既有逐字呈现与诊断（忠实但不可编译——响亮失败优于静默错误）。
- C1/C2 与全部既有重排正例逐字不变（实测其 super 目标均为 `java/lang/Object."<init>"`）。
- 以冻结反例建回归测试：断言 `AnonymousSuperDispatch$1` 的呈现中捕获写入序早于 `super()`（即重排未发生），并断言该文本重编运行时保持 `visibleDuringSuper=true` 或明确不可编译——二者皆不产生"可编译且行为不同"。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：构造器合成字段存的重排仅在 super 目标不可能虚分派时进行，避免构造期可见性被改变。

## Impact

`crates/jarde-java/src/build.rs`（重排判据处，daa4fb31 的 +17 行范围内）与 `crates/jarde-java/tests/p3_patterns.rs`、`tests/recover_synthetic_ctor_super_order.rs`（补负例）；无新机制、无呈现层改动。既有正例零回退。
