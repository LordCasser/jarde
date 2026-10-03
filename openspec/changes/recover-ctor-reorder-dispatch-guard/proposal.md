## Why

[构造器重排回归实证](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)：已验收切片 `recover-synthetic-ctor-super-order`（合并 daa4fb31）的重排判据未检查 super 目标是否可能构造期虚分派，对冻结反例 `anonymous-super-dispatch` 产生**静默行为回归**——原 class `observed=captured-value`/`visibleDuringSuper=true`，其呈现重编后 `observed=null`/`visibleDuringSuper=false`。重排前该产物不可编译（响亮失败），重排后可编译且行为不同（静默），违反 `recover-return-in-do-while-false` 已确立并验收的不变量。项目自 2026-09-25 已冻结该反例，`present-proved-java-structure` 的 2.10 明写"改序不损失任何效果"已被运行反例否定。

## What Changes

- 重排判据加**构造期可见性安全前置**（design 决策 1，单档）：仅当 super 调用目标恰为 `java/lang/Object.<init>()V`（owner 与 descriptor 双匹配）时，才允许把 pre-super 合成字段直传存组移至 super 之后呈现；任何其它 super 目标一律不重排，保持既有逐字呈现与诊断（忠实但不可编译——响亮失败优于静默错误）。
- **不做"读 super ctor 体证明无 this 虚分派"的宽档**（design 决策 1 三条实证否证）：跨类读方法体在本仓库无既有先例（bridge 走 `resolve_symbol` 只取声明、outer-super 消费已恢复报告、lambda 伴生读同类成员），引入即把窄修复扩成中大片；headers-only 覆写名代理会反向回归（`anonymous-super-args` 的 `render()` 与 super 同名却实际安全）；单类事实无法区分三处 fixture。
- **收紧不构成对已验收声明的回归**：`anonymous-super-args` 的既有验收证据（`evidence/java-syntax-2026-09-27/anonymous-super-args/report.md`）记录的就是 verbatim 序与"完整源码编译因此退出 1"——不可编译是重排切片之前的既有如实登记状态；收紧后退回该状态，符合 `present-proved-java-structure` 2.10 已声明的项目立场（独立二进制名类视图不声称该构造器可编译，等 5.3 类级匿名语法由 javac 生成前置写入）。
- C1/C2 与 `capture_ctor_class` 生成的全部既有重排正例逐字不变（实测其 super 目标全为 `java/lang/Object`）。
- 以冻结反例建回归测试：断言 `AnonymousSuperDispatch$1` 的呈现中捕获写入序早于 `super()`（即重排未发生），并断言该文本重编运行时保持 `visibleDuringSuper=true` 或明确不可编译——二者皆不产生"可编译且行为不同"。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：构造器合成字段存的重排仅在 super 目标不可能虚分派时进行，避免构造期可见性被改变。

## Impact

`crates/jarde-java/src/build.rs`（重排判据处，daa4fb31 的 +17 行范围内）与 `crates/jarde-java/tests/p3_patterns.rs`、`tests/recover_synthetic_ctor_super_order.rs`（补负例）；无新机制、无呈现层改动。既有正例零回退。
