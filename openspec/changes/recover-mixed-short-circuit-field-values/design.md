## Context

同极性 `ShortCircuitValue` 目前持有有序测试列表；列表中的共享出口直达同一 producer，继续边只到下一个测试，最后测试再分到 1/0 producer。冻结 `andOr(Z)V` 的 BCI 1 假边与 BCI 7 假边均到第三测试 BCI 10；`orAnd(Z)V` 的 BCI 1 真边与 BCI 7 真边均到 BCI 10。两图均无回边、只有一个 `putstatic Z`，但测试 10 的两条真实前驱打破线性不变量。普通 `If` walk 因而重复占有 producer/consumer，并不是这两段 bytecode 的语义有循环。

## Goals / Non-Goals

**Goals:** 只在局部可达、有向无环、边及 Phi 完全闭合的条件图上恢复字段值；逐路径保住 `b()`/`c()` 惰性调用；反例完整引用、无重复 owner。两测试与纯三测试现有正例、异常边和异常局部作用域回归不变。

**Non-Goals:** 反推出唯一源拼写，构建全方法 Boolean CFG 重写 pass，支持 ternary 里的 transfer-only 额外入口、任意多 producer 或多字段消费者，照搬 JADX 的极性合并。`(gate ? extra : left) || other || rhs()` 已是已知边界，须继续拒绝，不能因图放宽而变成误恢复。

## Decisions

1. **复用既有 Region，改变私有形状。** 从外层测试局部探测两个正常出口，按 canonical 身份收集比较测试节点；仅允许边去其他测试或两个候选 producer，且测试子图必须无环、无 handler/call 边。生产者各唯一进入同一消费块，消费块先有静态字段写入锚点。每个内部测试的真实前驱必须恰等于已收集测试的出口；不能有外部入口或 transfer-only gateway。只有整图闭合、预算已收费后才一次提交 `visited`。存储每个测试 BCI 和两个后继身份；不引入新 Region/AST 变体。
2. **值证明仍由 CFG、SSA 和字段三重约束。** 解码 target 决定每条 taken/fallthrough 的真实后继，不从地址远近猜极性；逐测试证明分支读取的 SSA 值树恰覆盖本块独立指令，拒绝额外效果。两个 producer 必须精确是 1/0、同栈深、只供一个消费块的唯一同槽 Phi；Phi 只有一个 `putstatic Z` 读者，字段身份与已有 field plan 相同。任何真实异常边、另一个消费者、未证明的前驱/出口均不发射。
3. **按图递归折叠已有 `Conditional`，逐路径惰性。** 从 outer test 开始，构造 `test ? takenExpr : fallthroughExpr`，叶子是已证明 1/0。共享后继可在互斥分支内复制 Java 表达式，不额外执行：例如两路到 `c()` 的图在任一实际路径只调用一次。构建前按节点数、最大展开深度和共享子图复制次数计费，并使用现有输出预算；超过限制时整段引用而非发射部分表达式。生成一个字段赋值，后缀仍在原消费点。
4. **拒绝先行、相邻语义分开。** 先完成逐指令来源和重叠所有权 change，再开放此正例；混合测试若不能封闭，回到那两项已验证的原子 quote。JADX `IfRegionMaker.mergeNestedIfNodes/mergeIfInfo` 只提供沿哪条边继续探测的线索；[其三元 OR 误极性](../../evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/analysis.md)证明路径相等启发式不足以决定值。

## Risks / Trade-offs

- [共享后继表达式复制导致副作用次数错或体积指数增长] → 路径互斥证明加 JVM 调用计数对照；预估展开大小、深度和输出预算，失败整体引用。
- [把外部入口或回边当局部 DAG] → canonical 精确前驱集合、已见节点/拓扑检查、异常边检查与额外入口永久负例。
- [与旧同极性证明重复实现] → 同一节点/边表和 SSA/field proof 支持线性子图作为特例；不并列建新机制。
- [过早开放导致 fallback 丢来源] → 依赖两个独立基础修复的验收；正例不能替代负例结果。
