## Context

现有 `guard::monitor` 在整个方法收集 `monitorexit` 并要求恰好 `[normal_exit, handler_exit]`，从一条正常退出后的 `return`/`goto` 推导单一正文、单一 `returns: Option<u32>`。`Plan::body` 是半开指令范围，`build::body_range` 将其平铺发射；`Region::Guard` 当前没有受保护的子 region。样例 `choose` 有两段受保护范围共享一个异常 handler、两条正常 `monitorexit; ireturn`；仅放宽 exits 计数会把条件分支和值求取平铺，产生错误程序。

普通 `Region::If` 已能表示双臂结构，但 `Walker::leaving_edge` 会把每个受保护块的异常边交给 guard/refusal，所以在证明 handler 前不能把现有 `If` 直接嵌入。当前架构需要一个**受已证明 handler 边界限制的 guard-body 递归**，或等价的已证明双臂映射；这是本问题所需的新局部机制，不是新的 Java AST 语法节点或一般 CFG 求解器。

## Goals / Non-Goals

**Goals:** 恢复一个锁、一个 `if` 两臂返回、两条正常退出共享一条异常清理的有限形状；完整类值、效果、异常对象、监视器归属和来源一致。

**Non-Goals:** 任意数量/嵌套退出、循环 break/continue、同步块内嵌 try/finally、资源关闭、非结构化分支、其它多出口 guard。后续可以复用被证明安全的 body 机制，但本项不宣称一般嵌套恢复。

## Decisions

1. **先证明 monitor 协议，再恢复内部形状。** 从真实 SSA/异常表证实进入锁的复制/槽身份、两条正常退出读同一锁、每个返回读各自臂内先求出的值、两段保护范围覆盖其所有可抛错位置、共享 handler 退出同一锁并重抛捕获原异常。额外或竞争 handler、未覆盖出口、不同锁与重复退出都拒绝。既有单出口规则不退化。
2. **给已认证 body 一个受限结构化表示。** 在同一 `Region::Guard` 内保留两个正常 arm 的条件/返回；复用 `Region::If` 或其已有 AST 构造，而不是独立再造一个 if emitter。区域递归只忽略本 Plan 已证明拥有的异常边，其它异常/子例程边仍导致拒绝；子 walk 受原 body 范围、已认领块与 `MAX_REGION_DEPTH`/Budget 限制。旧的 fused block 的 header/body 半开 BCI 边界仍由 Plan 负责，不能把 header 重写一次。
3. **正文按控制流而非 BCI 顺序写。** 两个 `return` 留在各自分支、同步块内；`monitorexit` 和 handler 的物理指令不作为额外语句发射。借现有 `return_expr` 在原返回消费点呈现值，要求调用/字段/数组生产者单次归属。若内部区域未能形成唯一结构，引用整个候选，不输出半个同步块。
4. **来源与停止沿现有路径。** Plan 的事实/owned 集合涵盖进入、两段 protected rows、每个退出、分支、返回、handler；发射来源把两条物理解锁路径都归到同一 `synchronized` 结构。取消和预算发生在证明或子 walk 时不发布部分 AST。

## Risks / Trade-offs

- 两个 normal exit 与一个 handler 的数量相符，却可能保护不同锁或漏掉一臂异常；逐出口 SSA 与每段范围验证，不能以数量作证明。
- 在已保护 body 内直接调用普通 region walk会把其异常边再次当作外层 guard，递归错误或遗漏；只在已认证 ordinal 的边上建立受限子 walk，其它边继续拒绝。
- 平铺半开 BCI 指令范围会把两臂都执行；必须把每个返回和它的值绑定到原分支 arm，且在完整类执行中核实未选臂不调用。
