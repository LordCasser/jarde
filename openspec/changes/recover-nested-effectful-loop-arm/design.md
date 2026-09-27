## Context

见 [冻结基线](../../evidence/java-syntax-2026-09-27/cf08-endless-loops/nested-effectful-baseline/report.md)。当前 `effectful_dual_exit_loop` 已证明三个自然循环块、效果臂 `Push/Invoke/Store/Transfer`、直接 `break`、唯一内部 join 的 CFG/Code/SSA 关系，但在检查这些事实之前排除任何 `frame.boundary`。新样例外层 `if` 的非空臂以 BCI 50 为边界，循环 BCI 11/26/38 的两出口在 BCI 44 汇合，BCI 44 的直线尾段再到 BCI 50。Region 未覆盖 `[11,17,26,35,38,44]`，随后出现的局部引用诊断只是下游拒绝。

## Goals / Non-Goals

**Goals:** 只让一个已证明的外层条件臂承载现有带效果双出口循环及其唯一内部直线尾段；保持两层汇合、效果次数、SSA 局部值、来源与预算原子性。

**Non-Goals:** 任意嵌套层数、循环内虚调用或对象构造、内层长度分支、异常处理、多入口、局部声明防线放宽、`TestNotIndexedLoop` 整体恢复，以及新 AST/Region 类型。

## Decisions

1. **先复用原证书，再证明父臂边界。** 只在 `Frame::arm` 给出一个条件分支的独占边界、无外层循环 scope/shared tail/异常或 switch 所有权时，允许既有 `effectful_dual_exit_loop` 继续检查其完整 CFG/Code/SSA 证书。内部 join 必须仍有准确两个正常前驱，内部直线尾段只从该 join 到父边界，不能由任一出口绕过。候选循环及其两个出口均由该父臂唯一可达和拥有。不能仅删去 `frame.boundary.is_some()` 拒绝，因为那会使循环吞掉父臂尾段。
2. **保持现有 Region 组合和局部安全门。** 既有 `Region::Loop`、`LoopBreak` 返回内部 join，父臂 walker 从该 join 消费一次直线尾段，并在外层边界停止；外层 `If` 继续拥有最终汇合。SSA φ 与效果调用仍由原证书验证，`build.rs` 的跨引用局部检查不改。若父臂不能在现有 Frame/Region 关系中闭合，返回原有拒绝，不通过新增通用 region 节点绕过证据。
3. **以物理三方回放验收。** 先固定输入源、class、JADX revision/算法文件哈希和当前 Jarde 拒绝。修后要求完整类源码 Java 8 重编与四组验证运行一致、无 `@bytecode`、BCI 11/17/26/35/38/44/50 各有且仅有可解释的 Region/源码来源；用额外入边、异向出口、绕过内部 join、异常边、预算/取消验证保守边界。重放已验收顶层效果双出口和单出口循环臂，固定 `TestNotIndexedLoop` 仍为红门槛。

## Risks / Trade-offs

- **父边界被误当作循环出口** → 要求两个出口只到内部 join，join 的独占直线续接才到父边界；检查每条正常前驱与后继。
- **效果调用被重复或跳过** → 保留原效果臂指令/SSA 证书，并用空数组、命中、未命中分别计数运行。
- **局部值因区域缺口被错误放宽** → 先闭合 Region 覆盖；现有局部引用安全门与其拒绝语义保持不变。
- **将首片误报为整个固定测试完成** → `TestNotIndexedLoop` 的构造、虚调用及内层分支继续单列，不把这个切片计作 CF-08 全量追平。
