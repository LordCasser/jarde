## Context

见 [proposal.md](proposal.md) 与[冻结的三方执行证据](../../evidence/java-syntax-2026-09-24/array-wrapped-binding/analysis.md)。数组直接元素绑定已由 [project-proved-enhanced-for-loops](../project-proved-enhanced-for-loops/verification-root-array.md) 验收；现有 `array_for_each_candidate` 在 `Region::Loop` 和 `ForHeader` 成立后仍只认循环体首句 `Declare` 的根 `Index`。JADX 的 wrapped `AGET` 替换覆盖更多表达式，却在三个可执行方法中把数组读取移过写数组调用，改变结果。

## Goals / Non-Goals

**Goals:** 在既有计数循环证明之上，允许每轮第一个可观察动作是唯一数组元素读取、该读取又嵌在首条语句表达式中的窄子集，原子投影为增强 `for`；其余形状保留当前可执行计数循环。

**Non-Goals:** 不引入通用别名或副作用分析，不恢复任意表达式子树中的 `AGET`，不修独立的 explanation-only 手写数组循环，不实现 `Iterable` 投影。

## Decisions

1. **复用已有证明入口。** 候选仍只在 `ForHeader` 已建成的 `StmtKind::For` 后检查零初值、+1 更新、同一数组 SSA 身份、长度缓存、索引独占用途、退出边及处理器信息。扩展的仅是元素读取在首条体语句的消费形态；新建通用 loop pass 会重复 Region 的证明，直接照搬 JADX 的指令隐藏会使失败时难以回滚。
2. **用 SSA 和表达式顺序共同证明读取位置。** 元素 `ArrayLoad` 必须唯一，索引不得另用，读取必须位于每轮所有可观察体内动作之前并处于同一异常处理范围。沿首条语句的表达式求值次序检查其前缀；纯局部读取可以通过，调用、写入、解引用、可能抛错的操作及控制流分支均拒绝。读取之后的调用留在原表达式中。该条件是保守充分条件；无法证明未知调用不修改数组，所以不按名称推定其纯度。
3. **局部 AST 替换，原子提交。** 先从原表达式中确定唯一 `Index` 节点和真实 BCI，选取与已声明局部不冲突的元素名，用带来源的 `Local` 替换该节点，形成完整 `ForEach` 候选并计入预算。候选、类型和来源都成立后才移除原计数头与被折叠的元素读取；拒绝时不改变已生成的 `For`。不需要新 AST 语句形态或依赖。JADX 的 wrapped 指令替换可作为子树定位参考，但不能继承它的顺序假设。
4. **逐层验收。** classfile 解析与 JVM 验证沿用现有 reader/JVM 事实；`javac --release 8` 检查输出源方言；`java -Xverify:all` 对比原 class 的值、调用计数、数组最终状态、异常类型和顺序。JADX 只作为语法覆盖参照；它在写数组负例的错误结果不是预期值。

现有 AST 中 `ForEach` 的容器字段名为 `array`。本变更只处理数组，字段统一改为 `iterable` 的长期命名由并行 `Iterable` 子切片负责；实施时若该子切片已改名，使用新字段，不维持兼容层。

## Risks / Trade-offs

- [前缀操作被误判为无副作用] → 只放行已明确证明的纯局部读取和无异常值操作；未知调用及解引用一律拒绝。
- [同一元素读取两次或数组在两次读取之间被改写] → 要求唯一 `ArrayLoad` 和索引独占消费，拒绝合并读值。
- [子树替换后类型、局部名称或来源不完整] → 先生成完整候选与来源、检查 Java 8 元素类型和名称，再原子提交；预算/取消沿用既有停止契约。
- [与 `Iterable` 子切片同时编辑 `build.rs`] → 序列化代码提交与最终重放，保持两个 OpenSpec 的验收边界独立。

## Migration Plan

无持久数据迁移。旧行为是可执行的计数 `for`，新规则只在更窄的完整证明成立时投影；移除该局部候选入口即可退回旧输出。
