## Why

[运算符残留巡查](../../evidence/java-syntax-2026-10-05/operator-remainder-patrol/README.md)实证：`(x = x + 1) > 0`（赋值作表达式且值被条件消费）拒绝——javac 发射 **dup-store 舞蹈**：`iadd; dup; istore_0; ifle`（dup 值有两个读者：istore 一份、ifle 一份），诊断 "the copy at BCI 3 has no proved local assignment"。复合+短路链形（`condAssignOld`）与调用方级联同因。

**copy 值家族第 4 员**：数组 dance（dup 跨内层 iastore）、后缀旧值（load 先于 iinc）、putfield 链（dup 跨 putfield）、**dup-store（dup 后 istore+其它消费）**——同一诊断文本下的四种形状，前三种已分别立项。

**jadx 有解**：`return i + 1 > 0;`——参数槽的赋值**不可观察**（无后续读者），jadx 直接消除 store、表达式原样进条件。保守化判据：store 目标在 store 后**无任何读者**时可消除；有读者时需保留赋值语句（拆为先赋值再条件）。

## What Changes

dup-store 舞蹈的呈现分两分支：
- **store 目标无后续读者**（不可观察赋值——参数槽/死局部）：消除 store，表达式直接进消费位（与 jadx 同构）；
- **store 目标有后续读者**：拆为 `x = <expr>;` 语句 + 消费位用 `x`（保持求值序与语义）。

**不放宽多读者**（dup 值本身 >1 个消费方之外的形状保持拒绝——本片处理的正是恰 2 读者：store+条件）。

## Impact

- **代码**：`crates/jarde-java/src/build.rs` dup/局部赋值呈现区（"copy … has no proved local assignment" 发出处——与 #8/#9/#11 片同族，task 1.1 确认四形状是否同落点）。
- **测试**：`OP2` fixture + 家族三既有片零回退。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做通用不可观察赋值消除（仅限 dup-store 舞蹈形内的 store）；
- **不**动家族前三员已立项域（若实现发现四形状同落点可合并实现，但 change 边界不动）；
- **不**处理 dup 值 >2 读者形（保持拒绝）。
