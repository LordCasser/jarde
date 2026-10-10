## Why

JADX 活动用例 `TestFieldIncrement2` 的 `this.a.f *= n` 在当前 Jarde 中因未认领 `dup@4` 而只产生解释，完整生成源码无法重编。2026-10-10 的同形类族对照已经实际记录原程序 2/2、JADX 4/4 成功与 Jarde 0/4；内部类折叠成功，缺口在字段乘法更新。

## What Changes

- 沿用本次 SSA、字段计划与共同 `dup` 的位置证明，将非静态 `int` 字段更新的准确 `imul` 纳入现有复合赋值路径。
- 既有赋值 AST 增加乘法拼写，保持 receiver 一次求值、读写字段身份、唯一消费者、顺序与完整来源。
- 保留显式双 receiver 加法为普通赋值，补完整类族与对抗验收；不从字段名称或等价文本推定 receiver 身份。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 同次证据证明的一次 receiver int 字段乘法更新能输出 `*=`。

## Impact

涉及 `jarde-java` 的 `AssignOp`、`prove_field_update`、`Builder::field_write` 与已发射字段证据登记，以及窄片测试；沿用现有 emitter 和 class-family writer。前置为同次真实 CFG/frames/SSA、准确字段计划与既有预算。无新 pass、字段表、框架或依赖。

本片不实现静态乘法、数组乘法、其它运算符、非 int 字段、更新结果返回、跨块合并、双 receiver 的结构等价折叠、内部类通用能力或格式债务。不宣称 EM23 整单元追平。
