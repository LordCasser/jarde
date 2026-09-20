## Why

同一轮 benchmark 暴露了四处契约缺口，都是「读法」问题而不是行为问题；不写进文档，下一位消费者会按错误读法使用报告：

1. `quality=Structured` + `content=ContainsStatements` 很容易被读成「这份产物是对的」。它是活的：同一轮 campaign 里有 `Structured` 产物计算出与字节码不同的值（见 [fix-nested-arithmetic-value](../2026-09-20-fix-nested-arithmetic-value/proposal.md)）。载荷里每个平面都是结构性的，没有任何一个是语义等价声明；文档必须明说，并给出调用方该做什么。
2. `produced` 与 `content` 容易混为一谈：一次 sweep 的 **182,883** 个请求里有 **30,452** 个是 Produced 但产物不含语句。
3. 声明诊断词汇的含义随版本变化：`jre_declaration_class_not_in_run` 从单个 artifact 上的 **10,720** 次降到语料全局 **0** 次，而 `jre_declaration` 出现在**每个** artifact 上——跨版本对 code 计数不能读成改善信号。
4. `archive_entries` 在不同请求形状下不可比：两个 flat jar 是 **28** 与 **29**，一个大 flat jar 约 **6,009**，一个 WAR **500–1,296**，另一个 **3,702**（container tree root 会走子容器）。比较行之前必须先说明这个计数在各 scope 下的含义。

本 change 只改文档与契约陈述，**不改任何行为**。

## What Changes

- 把「载荷的每个平面都是结构性的、没有语义等价声明」写进拥有这些平面的 spec，并给出调用方动作：执行受控比较，或把产物当作假设。
- 把 `Produced` 与 `content` 分开表述：写成 README、支持矩阵、benchmark 协议与 owner spec 里可复核的句子，并引用 30,452/182,883 这条口径证据。
- 登记诊断词汇的版本语义：code 是「该版本记录的事实」的身份，不是跨版本可比的量；比较必须带引擎版本与 code 词汇。
- 为计数字段（以 `archive_entries` 为例）规定按请求形状与声明范围解释，比较行必须声明形状/声明/版本，MUST NOT 跨形状直接比较。
- 明确哪些条目可机械断言、哪些只能编辑性复核（见 design 的「机械断言与编辑性条目」一节）；**不新增断言散文的测试**。

## Capabilities

### New Capabilities

无。不新增框架、报告字段或平面。

### Modified Capabilities

- `java8-recovery`：content 与 Produced 的分离读法，以及 content 不是语义声明。
- `recovery-validation`：平面为结构性陈述，调用方必须执行受控对照或把产物当作假设。
- `analysis-contracts`：诊断码是带版本的词汇身份，跨版本计数不是质量信号。
- `measured-execution`：计数字段按请求形状/范围解释后方可比较。

当前仅完成文档与契约规划，实施任务全部待办；历史 benchmark 记录与归档不改写。

## Impact

文档落点：`README.md`、`docs/support-matrix.md`、`openspec/benchmark-protocol.md`，以及上述四个 capability 的 delta。不修改主 specs、不改 Rust、不改报告 schema、不新增测试（机械锚点复用既有测试，见 design）。

依赖：本 change 的第 1 条引用 [fix-nested-arithmetic-value](../2026-09-20-fix-nested-arithmetic-value/proposal.md) 的**措辞与反例结论**，不依赖它的代码落地；因此在 fix change 的反例事实确定后即可实施，两者不需要串行代码路径（本 change 不触碰 `crates/`）。与 [bound-recovery-recursion](../2026-09-20-bound-recovery-recursion/proposal.md) 无依赖。

非目标：不新增 crate、依赖或依赖升级；不引入 verifier；不声称一般语义等价；不重开 R8/R9 或已归档的停止传播修正；不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务；不改动任何既有报告的取值或分类逻辑。
