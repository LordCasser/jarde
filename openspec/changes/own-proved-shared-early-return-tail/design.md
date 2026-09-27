## Context

区域 walker 已有 `Region::If`、`forward_join`/`forward_join_predecessors`、`Frame` 边界及最终所有权验证；它们能处理一个直接后继就是汇合点的早退形状。CF-03 的 BCI 0 两侧起点是 4/14，而共同尾 BCI 24 是更深处的前向块，且两侧各有 `ireturn`，所以 BCI 0 没有通常的 post-dominator。内层 BCI 8/18 的递归目前各把 BCI 24 纳入自己的 arm，最终所有权验证正确拒绝。`region-probe.log` 保留了相关边界事实。

## Goals / Non-Goals

**Goals:** 先证明共同前向尾部，再把它用作外层 `If` 的唯一续接点；递归 arm 不得越过该边界，尾部只归属续接区域；仍由 builder 逐条证明条件、返回及副作用。三方完整源码重编和验证运行是验收门。

**Non-Goals:** 一般 CFG 结构化、不可约图、异常/循环/switch 内共享尾、布尔值合并成表达式、CF-04 整数 ternary、跳过区域所有权验证以换取输出。

## Decisions

1. **从现有 CFG 事实选候选，而非按地址猜测。** 对两后继的共同正常可达块，限定同 jsr path、严格前向、非 loop header，要求唯一最近共同入口；全部进入候选的边需为正常边、由外层 branch 支配、无回边，且从两侧各有到达路径。拒绝多候选不可比、侧入口或异常/保护边界。现有 `forward_join_predecessors` 可复用入边/支配证明，但自身不足以证明“这是两臂共同尾”，需要两侧可达和唯一性检查。JADX `IfRegionMaker` 的 out block 选择只作对照，不照搬其可能把非编译 Smali 当正例的门。
2. **在区域 walker 中表达一次所有权。** 候选作为外层 `If` 的 join；嵌套 arm 的走访仍须继承外层候选的停止界，不能因内层另有早退 join 就走入共同尾。优先使用 `Frame` 已有边界/作用域约束或最小的父界传递，不引入新的全局 CFG rewrite 或第二套 AST。`Region::If` 之后只从未领取的 join 续走一次。保留 `validate_region_ownership` 作为最后一道强制门，绝不能把双归属从检查中豁免。
3. **先验拒绝与计费。** 新候选扫描按已有分析预算逐边/逐块收费，取消/停止原样传播；遇到异常边、循环、多个入口、无法闭合的 arm 或构建阶段来源缺口，回退整个受影响方法，保留 BCI 与原因。只在完全证明后提交 `visited` 和区域。`BranchShapes.guards`、`ChainOnly` 与既有 P3 预算/来源测试作为非回归对照。

## Risks / Trade-offs

- [将早退路径误认为也会执行尾部] → 用物理 CFG 分别证明每条 arm 的终止或抵达 join，三方运行覆盖 `false` 和 `true`、检查 `hits`。
- [嵌套 frame 丢失外层停止点] → 在测试中直接断言 BCI 24 恰有一个 Region owner，且 class-source 仅有一次 `hits++`。
- [额外图扫描破坏预算固定用量] → 限定候选查找范围并单独验证预算停止/取消，不借改动全局用量掩盖漂移。
