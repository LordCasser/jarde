## Context

[CF-02 冻结对照](../../evidence/java-syntax-2026-09-27/cf02-predicates/report.md)把问题定位在 `Predicates.named(Object)` 的正常控制流，而非浮点比较或 `instanceof` 指令解码。原 class 与 JADX 完整源码对 `"x"` 返回 `true`；Jarde 完整源码能重编，却执行到错误的无条件 `return false`。Jarde 报告 `semantic_validation=unproven`，已知 BCI 28 可达但被尾部 `UncoveredBlocks` 注释收容。

当前 `region.rs::region_at` 的 one-armed `if` 递归产生 `(arm_run, arm_next)` 后丢弃 `arm_next`，沿外层 join 继续遍历。这里内层谓词值的合流是 BCI 28，外层早退 join 是 BCI 11；二者不相等。现有 `short_circuit_value` 只接受方法级或受证 try 头，不能直接覆盖嵌在该 arm 内的布尔尾值。JADX 的 `IfRegionMaker`/`IfCondition` 提供条件合并参考，但不证明 Jarde 已拥有相同的区域所有权。

## Goals / Non-Goals

**Goals:** 在发布 one-armed 条件区域前闭合内层所有正常可达后继；对于本片无异常边、已证 SSA 值和单一返回消费者的 Java 8 谓词，复用现有 Region/布尔值生成路径恢复提前返回及尾部返回；无法闭合时不发表可编译但错误的执行前缀。原 class、JADX、Jarde 的完整类源码重编和运行结果是验收门槛。

**Non-Goals:** 通用 CFG 重写器、任意循环/异常处理器/`finally` 的控制流复原、跨方法类型推断、所有 Java 浮点比较形式、CF-03 的 else-if 美化，以及修复 `TestTernary3` 外部 `InsnArg` 层级。

## Decisions

1. **先校验区域后继，再发表候选。** one-armed `if` 不能静默丢弃递归得到的 `arm_next`。若该后继不同于已证外层 join，候选必须先用现有 CFG 支配/可达性、SSA 消费和区域边界证明它由当前闭包拥有，并将它继续纳入走访；不能证明时回退整个受影响闭包，不能先发 `return false` 再追加不可执行的 BCI 注释。沿用当前 `Region` 候选、预算、停止和来源机制，不新增第二套 CFG。
2. **尾布尔值只在窄形状内恢复。** 本片准入的是判空或类型 guard 提前 `return false`，真臂的 cast/调用及比较完整到同一个布尔返回。检查每个正常前驱、布尔生产者和最终 `ireturn` 属于闭包，所有效果按物理路径执行一次，cast 只在类型守卫通过后执行。可复用现有 `Region::If` 与布尔值 builder，或在其现有候选入口扩大受证嵌套形状；不通过仅凭文字将条件重排为 `&&` 的方案。生成 `if (...) return false; return ...;` 与等价短路式都可接受。
3. **未证时保持诚实且原子。** 若异常边、跨区域消费、二义 join、缺失后继或预算停止使闭包不完整，应沿现有拒绝/回退通道保留物理 BCI 和 `unproven` 报告。输出中不得存在一条在真实可达尾路径之前无条件截断执行的返回语句。若当前回退格式无法表达完整 Java 控制流，应拒绝该方法的可信源码恢复，不能把“可编译”当作正确性证据。
4. **将对照证据和回归分层。** 固定 `Predicates` 十行原/JADX/Jarde 运行结果；同时保留 NaN、无穷、负零和数组界限比较作为非回归。增加跨 join、额外副作用、异常边、缺失 SSA 消费者和低预算负例，检验完整恢复或原子拒绝。JADX 算法作为候选参考，最终以原 class 行为及物理来源闭合判定。

## Risks / Trade-offs

- [仅把 `arm_next` 设为下一块仍可能重复或遗漏分支] → 在候选提交前检查其前驱、支配范围、唯一消费者与已访问集合；来源同轮校验。
- [短路文本改变 cast、调用次数或异常顺序] → 固定守卫方向和物理顺序，不移动有副作用表达式；任一效果证明失败则拒绝。
- [回退继续产出错误的可执行前缀] → 验证尾部所有正常可达 BCI 均被区域消费；未闭合时整体降级并报告，不能把单个注释当成语义闭合。
- [过度扩大到 CF-03/CF-04] → 本片仅修复单方法早退加尾布尔返回的已证形状，其他条件结构留在固定队列。
