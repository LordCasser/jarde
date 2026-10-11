# v2 只读对抗审计

审计对象：`/private/tmp/jarde-compound-condition-impl-luna-v2/{region.rs.base,region.rs.candidate,build.rs.base,build.rs.candidate,compound-condition.patch,README.md,base.json}`。

冻结 pins：region base SHA-256 `34dcf89ccee47c33e0b6e08089ae53567b30ce0f73b50e76b13aec1ddf040025`，candidate `32ff44a16e70a90f9c3503c9754289ac88e98d9b79d96ed2f1298b17577f8bad`；build base `768735cd1cf5d7694d2a1d7b28030475b0bc0864e1d18d9a6350e12c999b1de1`，candidate `cf6fffaca9ae1eb24b3ba45e5e5fd034c886e84644722a5cff3ba5b3bb266670`；组合 patch SHA-256 `870e855af60bf26a1fad090acdada715eb2d9ded0896bc82fbaa76273c916110`。fixture pin 来自 base.json：1123 bytes，BLAKE3 `3ec50697523d585c5a23797922f0289334f6e76e9e83f92596d46fdce4cf0b44`。

本报告为纯静态审查，未应用候选、未编译、未跑 Cargo/JDK/CLI/测试，不能将 README 中源码断言当成运行结果。

## 阻塞问题

1. 归约方法偏离 design 决策 3。`compound_statement_condition` 的 `reduce`（region.rs.candidate 8040 左右）递归地产生 Shannon 公式 `(test && then) || (!test && otherwise)`，memo 保存/clone 条件表达式，再由 `simplify` 的有限 AST 模板做约简/提因子。设计要求依据出口摘要做 then/else 对齐，组合前要求测试集合不相交、共享节点真实前驱被组合完整覆盖；候选没有这些摘要/集合不相交证明。即使末尾计数拒绝重叶、Switch2 期望例恰好被某模板改写，仍不能证明一般共享子图不被展开、排序与副作用顺序安全。Root 已独立发现此阻塞；v3 应替换该归约而不是仅加模板。

2. Stop 会绕过 Walker 回滚。region.rs.candidate 3563–3694 先快照，再把 proof.tests 插入 `self.visited`，然后 `region_at(payload)?`、计费 `charge(...)?` 和 canonical-edge 循环内 `poll(...)?`；rollback 在 3691 之后。任一 Stop 从这些 `?` 提前返回会留下已插入测试及 arm walk 的 visited 状态。可复现方式：预算足以完成 proof/插测试，调到 payload `region_at` 或 edge scan 中途耗尽，然后观察同一个 Walker 的 visited 与调用前快照不同。现有 candidate 测试只测直接调用 proof 不改 visited，没有覆盖这个提交包装器。需保证所有 Err 路径 rollback，成功才 commit。

3. CompoundIf builder 在可拒绝操作之前已经输出 prefix。build.rs.candidate 17918–17931 先 `self.block(prefix)`，之后才逐测试调用可能返回 Refusal/Stop 的 `compound_condition_expr`；Refusal 随后 fallback 会引用 `region_quote(region, ...)` 的完整拥有范围，prefix 已留在 `self.stmts`。证明阶段只检查 branch arity 与 `test_is_pure`，没有预验证 `test_expr` 的 Java 拼写/类型拒绝。具体路径是内部条件使用 `if_icmplt` 比较本层已证明的 boolean 值，`test_is_pure` 会允许 Load，compound proof 可先成功，但 builder `pair_comparison_refusal`（build.rs.candidate 24195–24214）拒绝该测试；此时 prefix 已发出，fallback 再覆盖整区间，产生重复语句/source origins。应该先 stage 所有条件/arms，或在 region proof 中完成 builder 等价的条件可写性核验，再输出 prefix。

4. 内部测试缺少关闭的 effect/producer 归属证明。CompoundIf 只调用根测试的 `test_effects`（build.rs.candidate 17933），内测 branch block 不作为普通 block 输出。候选依赖 `test_is_pure`，但其 `test_expression_instruction`（region.rs.candidate 8285–8315）对 Push/Load/Arithmetic/Negate/NumericComparison 无条件返回 true，未检查这些 BCI 是否属于 `condition_value_bcis`；它也接受 `store_dance_part`。因此 proof 可把不属于内测条件值树的指令当成“表达式自身部分”，而 builder 不会输出内测 effect。

   - 算术边界：int/long Divide/Remainder 可抛 `ArithmeticException`，是可观察 effect。独立 `idiv; pop` 这种最短字节码会被 `pop` 的 `Other` 拒绝，所以目前没有构造出已确认被 candidate 接受的最短运行 witness；不能把它报成已观察到的错误输出。但 whitelist 不检查依赖是确定缺陷，需要求每条非 branch 指令归入该 branch 的 condition dependency 或已证明 dance。不要把 `conditional_dependencies` 原样搬来，它服务于 conditional Phi arm，而非该问题本身。

   - 已观察到 assignment 闭环缺口：`prove_local_assignments` 对 CompoundIf 每个 test 都标 `LocalAssignmentTest::Structure`（build.rs.candidate 7593–7601）。对可观察参数赋值 dance（`dup; istore param; ifeq`），Structure 将其判作 Split；Split 的 test 读取只返回 local 名，不发 assignment（25391–25405）。emitter 只对 root 调 `test_effects`，所以短路可达的内测 store 不会在其原位置输出。随后 `assignment_presentations` 无法计为一次，`unconsumed_local_assignment` 会触发整方法 fallback（8226 以后），所以静态证据指向晚期拒绝/降级，尚未看到错误 Java 被发布；但 proof 已把超出设计范围的内部独立 store 收为 CompoundIf。最小修正是 proof 阶段拒绝内测 `store_dance_part`/任何非条件独立指令，不扩成 assignment-inline。

## 范围/实现限制

候选 proof 明确只接受 exits 集合恰好为 `{payload, join}`，并输出 `then=payload, else=empty join`（region.rs.candidate 7760 以后及 3675–3687）。因此它不表示两个都含语句的普通 arms。若本 change 的“两普通语句出口”要求覆盖两个非空不同 arm，这是范围缺失；若该词指一个 payload arm 加 continuation join，则该限制符合 Switch2 例。此项是静态范围核对，不单独作为安全误码结论。

`prepare_conditional_region` 对 CompoundIf 只递归 arms（build.rs.candidate 16345 左右），没有对自身 tests 构建 specialized conditional-value plan。当前看不出 statement-test 的普通比较必然需要该 plan；`conditional_dependencies` 是 Phi 表达式 arm 专用，所以仅记录为设计边界核查，不主张它单独构成 bug。必须确保内测比较值来源都已由 test dependency/effect closure 证明。

## 其余静态核对

CompoundIf 在 Region block 归属、普通 path、guard/递归以及主要汇总路径已有显式分支；专项仅接收 `Region::If` 的模式匹配大多仍会拒绝新变体。未编译，仍需 root 编译确认遗漏的 exhaustive consumer 和类型/借用错误。README 声明候选尚未跑工具链；没有将其 Switch2 结构断言视为验收。
