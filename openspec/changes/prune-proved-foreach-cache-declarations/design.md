## Context

见 `proposal.md` 和 [架构定位](../../evidence/java-syntax-2026-09-24/foreach-cache-declarations/analysis.md)。`declarations` 在构建 AST 前按 `LocalVariable` 身份与 RegionPath 把跨区域变量的无初值声明提升到方法/区域头；`array_for_each_candidate` 后来以 SSA 独占使用、数组同一性、handler 和执行顺序证明，把长度赋值、下标初值/更新及元素读取折为增强 `for`。候选成功时已把这些语句 BCI 合进新循环的来源，却只移走赋值和原循环，未移走此前发射的前置声明。

## Goals / Non-Goals

**Goals:** 在现有数组增强 `for` 证明成功后的同一提交里，清除确实失用的两个前置声明，保持精确 `LocalVariable` 身份、来源与预算原子性；直接/安全包装读取正例均覆盖。

**Non-Goals:** 不重新决定哪些循环可变增强 `for`，不按名称做一般死代码消除，不删除带初值或后继变量的声明，不改 `Iterable` 的投影证明、泛型元素类型或全局声明规划。

## Decisions

1. **消费现有证明，不另造活跃性 pass。** `array_for_each_candidate` 已经证明长度值只供该循环测试、归纳 phi 只供测试/读取/更新及允许的合流，并对索引逃逸拒绝。成功候选应附上它实际消费的长度 `Store` 和归纳初值的槽、BCI；用既有 `reuse.variable_at(slot,bci)` 定位 `LocalVariable`，再与 `Declarations::at_region` 的同一身份及 `HoistedDeclaration.at` 核对。只有匹配的无初值声明是本次可清理对象。替代的全 AST 名称扫描既重复证明，又会把同槽/重命名局部混在一起。
2. **候选与声明在一个已付费变更中提交。** 先核对候选证明、两个变量对应的声明位置和新 `ForEach` 的来源集合确实覆盖声明锚点；再同时移去缓存赋值、原循环和对应前置声明，发布一条 `ForEach`。若某声明本来就是初值内联而非前置，不凭空要求删除。若同一个变量在后继仍有消费，或 `reuse` 不能区分前后变量，放弃这次声明清理；另一个已区分的 `LocalVariable` 不匹配，被保留。Budget/取消发生在变更前应返回现有停止结果，不能部分修改 `self.stmts`。
3. **来源与文本双重验收。** 新循环的 `OriginSet` 目前由 `length_stmt`、`init`、`update` 和读取/测试表达式逐项合成；实现时把待删声明的主 BCI 明确列入或确认已存在。源映射中被折叠指令仍可由 `ForEach` 找回。现有 `p3_array_foreach`、`p3_wrapped_array_foreach` 和 `indexAfterLoop`/错配数组负例负责行为门槛，再加同槽复用夹具防误删。原/JADX/Jarde 的 Java 8 重编执行仍是独立验收，不用“文本更短”替代。

## Risks / Trade-offs

- **同槽异类型局部已有独立缺陷** → [ReuseAfterForEach](../../evidence/java-syntax-2026-09-24/foreach-cache-declarations/analysis.md) 的槽 2 在有无调试信息时都被当前命名层合成一个 `int[]` 变量，后继却写入 `int`，基线完整类不可编译。本 change 只要求不进一步误删槽 3 后继声明；不能把修好槽 2 的身份/类型问题当成此声明清理的成果。
- **被删声明携带唯一来源** → 清理前对 BCI 和新循环 OriginSet 交叉核对；source-map 测试逐项检查。
- **预算不足时修改了半个 AST** → 候选阶段完成计费/轮询和可删除声明收集，提交阶段一次性改动，保留原计数循环拒绝路径。
- **看起来多余的其它局部仍存在** → 本项只清理当前证明覆盖的长度/下标两项；其它源级死代码需独立证据，不能扩大为通用优化器。
