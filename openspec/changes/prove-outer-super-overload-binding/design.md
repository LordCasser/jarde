## Context

见 [proposal.md](proposal.md)、[EM-12 对照](../../evidence/java-syntax-2026-09-27/em12-super-dispatch/baseline/summary.json)与现有 `project-proved-outer-super-bridges`。`member_inner::prove_outer_super_bridge` 已确认 synthetic 桥单调用、准确直接父类目标、参数原序；`prove_outer_super_bridge_use_closure` 已确认所有使用点及词法捕获。最后的 `facade::prove_outer_super_source_binding` 遍历同名方法，却把任何非目标同名项直接当作竞争者拒绝。它没有区分 Java 8 下对生成实参根本不适用的重载。

## Goals / Non-Goals

**Goals:** 对已闭合的单成员家族，证明成员方法的直接形参载入在源码中仍保持与桥参数相同的静态引用类型；只在完整所选类层级证明同名候选不适用时放行词法 `Outer.super`。沿用既有物理身份、来源、预算和拒绝通道。

**Non-Goals:** 完整 Java 重载求解器、泛型实例化/varargs/装箱/拆箱、任意表达式的静态类型推导、跨 classpath 的猜测、改变桥体及使用闭包证书；EM-01 多成员装配和 EM-20 局部作用域另行处理。

## Decisions

1. **在源码绑定门扩展，不改桥和 IR。** 现有桥体与闭包证书先于源码绑定产生。给 `prove_outer_super_source_binding` 提供已证调用集合及成员方法所需的已选事实，在每个调用点核对实参 SSA 是成员方法直接形参载入，且该形参的非泛型 descriptor 与桥目标参数一致、当前 writer 不会改变这一静态类型。否则沿旧拒绝路径。单看桥 descriptor 的替代方案不成立：它不是编译器看到的实参类型。
2. **只证明竞争者不可适用，不猜“最优”候选。** 保留目标准确直接父类定义和方法表完整性。每个同名非目标方法须具备非 generic、非 varargs、非 bridge 的可读声明；若 arity 不同且非 varargs，则本调用不适用。相同 arity 的首片仅接受单个引用参数的严格子类竞争者：在所选环境内有界读取 `NarrowArg` 至 `Arg` 的完整 class 父链、证明它是严格子类；静态 `Arg` 实参不能向下转换为 `NarrowArg`，故该竞争者不适用。接口、多参数混合、来源缺失或可能适用均拒绝。直接“允许不同 descriptor”的替代方案会在 `null` 或更窄静态类型时改绑。
3. **先完成所有使用点与候选，再原子发布。** 任一调用点的实参类型、类层级、候选或 checked exception 证据不足，即保留物理桥及原拒绝。类读取、SSA/方法表遍历用现有 `Budget` 和 `ExecutionReport`，不能从宿主 classpath 偷取层级。证书仅在同轮来源/环境内有效，不新增缓存或公共类型系统。
4. **回归断言与源码重编分层。** 将 `outer_super_method_bridge_is_not_projected` 的旧拒绝期望改为正向家族/来源断言，因为冻结 `OuterReceiverCases` 在原 class、JADX、Jarde 均可重编且验证运行 `20:10:1:3`。EM-12 `Case/Parent/Arg/NarrowArg` 需修后完整源码重编并运行 `number`。负例构造静态 `NarrowArg` 实参、同型显式 `other`、缺类层级与异常/泛型形态，确认仍拒绝。JADX 的算法和文本只作对照，不能替代 Java 编译与原 class 运行证据。

## Risks / Trade-offs

- [把输出实参误判为桥参数类型] → 只准直接成员形参 SSA 载入，核对声明 descriptor 与 writer 的源码签名；任何 cast、phi、泛型或中间生产者拒绝。
- [层级不全仍判竞争者不适用] → 要求所选定义唯一、方法/类头完整、父链到目标参数闭合；预算停止或解析歧义整体拒绝。
- [可适用性被 varargs、泛型或 checked exception 改变] → 保留现有这些门的保守拒绝；只放行单引用参数且严格子类竞争者的首片。
- [旧测试预期与实际正确投影矛盾] → 先用三方完整源码/行为证据改断言，同时保留同型 `other` 的真正拒绝负例。
