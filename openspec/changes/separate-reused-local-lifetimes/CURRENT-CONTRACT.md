# Reused loop-slot dependent refusal

本次修正负例测试契约，不改变恢复准入。冻结输入 `SlotReuseBoundaries.class` 的 SHA-256 为 `e2b174e41766a41525401d46c1738371d8f835ced88718997d5d4fe0de2c37cb`；原 class 与历史 baseline 均逐字节保留。

`loopBodyReuse(I)I` 每轮在 BCI 13 向槽 2 写入数组引用，再于 BCI 23 写入整数。累加器在 BCI 19、27 的写入依赖这些值，BCI 31 的回边返回 BCI 2。若拒绝上述写入，却保留可执行循环与返回，行为就会丢失。历史 `evidence/analysis.md` 明确记录旧局部拒绝文本返回 0，而原方法返回 8；不得恢复这种不安全输出。

当前契约要求 `ExplanationOnly` 内容、`Fallback` 质量与 `Mixed` 表示，不含可执行语句，全部实际指令均有闭合来源。独立反汇编确认本方法共有 **29 个指令起点**；31 是回边的 BCI，不是指令数。声明、依赖读取、副作用及返回必须一并拒绝。essential/all 必须得到相同源码和分类。每段诊断引用必须精确对应其列出的指令、同一方法身份及合法 UTF-8 范围。本地拒绝诊断保留在正文中；不能把空 region fallback 列表误称为已恢复。当前方法解释另存于新契约夹具，历史 baseline 不改写。

七个负例分别命名执行，避免首个失败掩盖 ParameterHeader、HeldUse、RefLoopBack。比较 SlotReuseBoundaries 的其余类文本时，只允许已经独立证明的 loopBodyReuse 完整拒绝变化及既有 category2Adjacent 归一化；其他负例契约保持不变。

受限 JVM driver 使用精确冻结原 class，验证负数、零、1、2、3 次循环及所有兄弟方法，其中输入 2 的期望值为 8。完整、未经编辑的 Engine 类单独重编，仍是编译拒绝。原 class 执行不是恢复文本的语义等价证据；不得抽取或修补方法后冒称完整 Engine 输出通过。正向恢复需要另行实现结构性的循环局部生命周期证明，并验证完整生成类的运行行为。

本候选不改变生产源码、历史证据、issue 38 契约或冻结夹具，不处理另行等待选择的架构审计问题。sealed24 的完整 inventory 与慢速 issue 30 结果仅作为精确绑定的既有基线复用，不冒称为新执行的 candidate25 测试。
