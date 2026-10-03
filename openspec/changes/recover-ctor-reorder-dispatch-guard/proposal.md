## Why

[构造器重排回归实证](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)：已验收切片 `recover-synthetic-ctor-super-order`（合并 daa4fb31）的重排判据未检查 super 目标是否可能构造期虚分派，对冻结反例 `anonymous-super-dispatch` 产生**静默行为回归**——原 class `observed=captured-value`/`visibleDuringSuper=true`，其呈现重编后 `observed=null`/`visibleDuringSuper=false`。重排前该产物不可编译（响亮失败），重排后可编译且行为不同（静默），违反 `recover-return-in-do-while-false` 已确立并验收的不变量。项目自 2026-09-25 已冻结该反例，`present-proved-java-structure` 的 2.10 明写"改序不损失任何效果"已被运行反例否定。

## What Changes

- 重排判据加**构造期可见性安全前置**（design 决策 1 三档）：(a) super 目标为 `java/lang/Object.<init>()V`，或 (b) super 类在本快照有物理定义且其 ctor 可证明构造期无法到达子类覆写——两条都要成立：无 receiver 为 `this`（`aload_0`/slot 0）的 `invokevirtual`/`invokeinterface`，**且**不把 `this` 作为实参传给任何调用（被调方可能再分派）时，才允许把 pre-super 合成字段存组移至 super 之后；(c) 其它情况不重排，保持既有逐字呈现与诊断（忠实但不可编译——响亮失败优于静默错误）。
- **判据必须同时保住两类既有正确行为**：`anonymous-super-dispatch`（super ctor 有 `this` 虚分派）落入 (c) 不再重排、回归消除；`anonymous-super-args`/`anonymous-capture`（super ctor 两条均满足，实测）落入 (b) 继续重排、保持当前既正确又可编译。C1/C2 等 Object 正例落入 (a) 逐字不变。
- 以冻结反例建回归测试：断言 `AnonymousSuperDispatch$1` 的呈现中捕获写入序早于 `super()`（即重排未发生），并断言该文本重编运行时保持 `visibleDuringSuper=true` 或明确不可编译——二者皆不产生"可编译且行为不同"。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：构造器合成字段存的重排仅在 super 目标不可能虚分派时进行，避免构造期可见性被改变。

## Impact

`crates/jarde-java/src/build.rs`（重排判据处，daa4fb31 的 +17 行范围内）与 `crates/jarde-java/tests/p3_patterns.rs`、`tests/recover_synthetic_ctor_super_order.rs`（补负例）；无新机制、无呈现层改动。既有正例零回退。
