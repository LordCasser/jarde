# 多消费者族可恢复性取证（root，2026-10-06，读码+归档渲染）

## 两形状分流（同一诊断文本，机制难度不同）

### 形 1（MVP）：已声明局部的表达式内多读——NI 形

`NI local1 = new NI(); … n.make(3).v + n.make(2).outerTag() + externalMake(n,1)…`——渲染**已提交**
`NI local1 = new NI();` 声明，随后整条表达式被 "the saved producer at BCI 8 has 3 consumers, so
one local binding cannot prove its execution count" + 级联吞掉。

**关键观察（读归档渲染）**：局部声明本身已在文本里（一次 store、SSA 单定义）。消费者读的是
**已提交的局部**——执行次数=1 由声明单次性直接证明，与"无声明栈携带值的 saved producer"不同。
拒绝来自 deferred-value 机制对 saved 声明的消费者计数门**不区分"已提交局部"与"栈携带值"**。

**判别**：NJ（2 消费者）恢复、NI（≥3）拒——阈值即该计数门。MVP：生产者已是**已提交局部声明**
（单 store、SSA 单 def、声明文本在先）时，多消费者直接以 `local1` 呈现（源码本就如此），计数门
只保留给**无声明栈携带值**（那里的执行次数问题真实存在）。

### 形 2（后续）：循环携带引用——BI 形

`for (int x : arr) { if (!ok) return false; }`——数组引用 3 消费者（长度/null/迭代器协议）跨循
环迭代，执行次数=迭代数。这里"one local binding cannot prove its execution count"的顾虑**部分
真实**（源码中 arr 也被增强 for 隐式读多次，但呈现为一次句法引用）。需要增强 for 协议级的消费
者建模（iterator()/hasNext()/next() 的隐式多读），是 `project-proved-enhanced-for-loops` 域的
邻接扩展，不与形 1 混片。

## 门控实验要求（task 1.1）

定位消费者计数门的发出处；区分"已提交局部"与"栈携带值"两个输入类；门控实验：只对前者放宽计
数门，NI 锚翻转、BI 锚不翻（形 2 不受累）；NJ/既有 saved-value 测试零回退。

## 行为验收（预登记）

- NI main 恢复（0 该诊断），剥离编译 exit 0、`-Xverify:all` 输出与原一致（`3/10/5/true`）；
- BI（形 2）拒绝逐字不变；NJ 与既有 deferred-value 顺序测试（preserve-deferred-value-order 10/10 的锚）零回退；
- 负例：无声明的栈携带多消费者形保持拒绝。
