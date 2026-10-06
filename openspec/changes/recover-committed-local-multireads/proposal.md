# 已提交局部的多消费者呈现（recover-committed-local-multireads）

## Why

[多消费者巡查 + root 可恢复性取证](../../evidence/java-syntax-2026-10-05/multiconsumer-local-soundness-patrol/recoverability-forensics.md)：单表达式内消费**已声明局部** ≥3 次（NI 形：`NI local1 = new NI(); … n.make(3).v + n.make(2).outerTag() + externalMake(n,1).outerRef().tag + (… == n)`）整条表达式被 "the saved producer at BCI N has M consumers, so one local binding cannot prove its execution count" 吞掉——现在该形由 soundness 守卫保证不可编译错，但可恢复性缺口在：**已提交局部声明的执行次数=1 由声明单次性直接证明**（单 store、SSA 单 def、声明文本在先），计数门把这类值与"无声明栈携带值"混同。jadx 完整解（源码本就直接引用局部多次）。判别：NJ（2 消费者）恢复、NI（≥3）拒——纯阈值效应。

## What Changes

消费者计数门区分两个输入类：生产者是**已提交局部声明**（其 store 已作为声明语句呈现、SSA 单定义）时，多消费者直接以该局部名呈现（表达式内多次读取=源码形态），不触发计数门；生产者是**无声明的栈携带 saved 值**时，计数门逐字保留（那里的执行次数顾虑真实）。

- 复用既有 deferred-value 呈现机制（不新建表达式通道）；只改计数门的输入分类；
- BI 形（循环携带数组引用，`for(x:arr)` 协议隐式多读）**明确出范围**——增强 for 协议消费者建模是 `project-proved-enhanced-for-loops` 邻接扩展，另片处理；
- `preserve-deferred-value-order`（10/10）的顺序判据零触碰（多读不改变求值序证明）。

## 硬不变量

1. 无声明的栈携带多消费者形拒绝逐字不变；
2. BI/循环携带形拒绝逐字不变；
3. NJ 与既有 deferred-value 顺序锚零回退；
4. 不得产出"可编译且行为不同"文本（守卫四族清单不触碰）。

## 验收

- NI main 恢复（0 该诊断）、剥离编译 exit 0、`-Xverify:all` 输出与原一致（`3/10/5/true`）；
- 门控实验先行（task 1.1）：只对已提交局部放宽，NI 翻转、BI 不翻；
- 全门禁 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：已提交局部声明在单表达式内的多次读取按源码形态呈现，方法行为完整。
