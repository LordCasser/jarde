# new 与既有引用消费者的组合边界

2026-09-23 用已验收Throw的CLI继续巡查new消费链。`run_audit.py`固定了一个自写主类和source-only构造器效果/runner。原class的14行执行与JADX完整输出相等；jarde有11处引用，完整javac失败。CLI哈希见summary.json，未把随后生产文件变更当作该可执行文件的能力。

| 组合 | 当前结果 | 已观察的关键行为 |
| --- | --- | --- |
| new → checkcast → return | 构造未被认领，引用new/dup/构造调用/cast/return | 合法引用转换，构造一次；构造器自身抛错优先 |
| new → aastore | 构造未被认领，引用new/dup/构造调用/store | 即使数组null或越界，构造仍先执行一次；构造器抛错时不发生后续数组检查 |
| new → instanceof → return | 构造消费与instanceof本身均未恢复 | 构造副作用不能因类型检查结果已知而省去 |
| new → getfield / invokevirtual | 正常呈现为new表达式的字段/方法访问 | 作为已有消费者的正面对照；本次完整jarde类不编译，因此未冒称已做jarde执行对照 |

## 架构定位

`init::renders_its_reads`只认领已有消费者，仍明确排除cast及数组；该注释形成时依赖这些后续规则的认领结果。现在普通checkcast已由build直接呈现，ArrayStore也有既有呈现分支，而构造证明的旧准入未跟进。`build::renders_the_value_it_reads`已包含这些消费者，因此两处判断存在阶段性的能力差异；不能仅见两个名单不同就无条件合并，它们分别回答“构造可被认领”和“生产值可延期”两个问题，InvokeDynamic等仍有不同所有权。

补齐这些组合不需要新AST、栈清理或通用类型解析。后续可在既有初始化证明中接通已经能够直接呈现的消费者，同时复核失败消费是否保留new/dup/constructor及构造实参效果。必须保留现有唯一消费与最终at证明，不把dup、未知读者或尚未实现的instanceof提前当成已接受消费者。未认领时当前文本明确引用，不存在本次把错误正常表达式冒充恢复的结论。

本项先独立记录，待instanceof实现后形成有限组合任务。不会为消除11处引用而把构造折叠、所有数组规则或所有准入名单合并成新机制。
