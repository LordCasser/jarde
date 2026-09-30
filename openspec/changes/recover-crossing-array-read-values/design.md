## Context

[巡查证据](../../evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/README.md)：F2 字节码 write@15 = `iload_1; aload_0; iload_2; iaload; iadd; istore_1`；界门触发条件（crosses_exception）与读侧 phi 链（entry 写@1 → 头 phi；try 写@24 / catch iinc@29 → 汇合 phi）均已在现有 walk 内闭合，唯一失败点是 `presented_int_store_value` 对 `iaload@13` 返回 false。F1 对照（无界门）同表达式呈现 `local1 = local1 + arg0[local2];` 正常。既有先例：Test2 的 `array_of_value`（元素类型细化）、cf15 Slice A 的构造 store 白名单扩展——同族、同一内联纪律（同块、单用途、区间内无插指）。

## Goals / Non-Goals

**Goals:** 白名单接受同块数组元素读（primitive 与引用数组各验一形），数组操作数/下标树均在既有白名单内；F2 恢复、行为一致、整类可编；F1 逐字不变。**Non-Goals:** 跨块数组读；共享（多用途）元素值；`aaload` 链上再取字段/调用的深表达式（随后按需另片）；界门与 phi walk 语义；`arraylength` 值（若 F2 不需要则不做，出现时如实报告）。

## Decisions

1. **形态判据沿用内联纪律**：`Operation` 的数组元素读臂（iaload/aaload/laload/faload/daload/baload/caload/saload），其数组操作数与下标操作数递归过既有白名单；值单用途（`single_use_at_with_budget`）；首生产者到 store 的区间封闭检查沿用（`seen` 集合机制天然覆盖新臂）。引用数组的元素类型经 `array_of_value` 已有答案（Test2 通道）。
2. **验收锚定**：F2（`14`）+ 变体（`double[]`/引用数组累计、catch 内多写、嵌套 try）编译运行对照；F1 diff 断言逐字；负例（共享元素值、跨块读、区间插副作用指令）保持拒绝。
3. **诊断噪声登记**：enumswitch 对非静态字段数组读的 shape 拒绝在巡查 README 已记录为无害噪声，本片不动。

## Risks / Trade-offs

- **内联改变求值序** → 判据要求区间封闭 + 单用途，与既有白名单成员同一不变量；"区间插副作用"负例钉死。
- **引用数组元素类型错拼** → 走 `array_of_value` 既有证明；拼不出即拒绝（不放宽）。
