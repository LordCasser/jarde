# ForHeader producer match：只读对抗审查

审查对象是当前已应用的 `crates/jarde-java/src/region.rs`（SHA 见 `pins.json`），范围只覆盖此次新增 match alternatives。依据为 change 的 proposal/design/spec/tasks、`match-application-root-v1/acceptance-root.json` 与已接受的 `controls-baseline-root-v1` 归档。没有执行 Git、Cargo、JDK、JADX、CLI 或任何测试；因此下面给出静态证据与要回归的测试清单，不把未运行的实现称为已验证正确。

## 静态路径结论

唯一生产路径在 `region.rs:12762-12783`：`header_tested_loop` 先对同一 header/test 调 `test_is_pure(header, test_bci, true)`，成功后再跑 `prove_for_header`。条件值 BCI 集由 `condition_value_bcis` (`region.rs:9890`) 从 branch 的 SSA reads 向同 block producer 回溯。`test_expression_instruction` (`region.rs:9530-9590`) 对 Invoke 和 Field Read 先要求 producer BCI 属于这个条件依赖集；ArrayLength、ArrayLoad、ArrayElementLoad 也要求该依赖关系，且其输出值在本 block 的 `reads_per_value` 恰好一个 reader。数组长度另外受 `allow_array_length=true` 控制。

Invoke 与 Field Read 没有单独的 `uses().len() == 1` 检查。这是需要防守的边界，但当前 match 还不接受 `Operation::Duplicate`、`Store`、`Copy` 或其它未列出的 stack-copy/store 操作；`decode.rs:169` 把 `dup` 显式标成 `Duplicate`，且 `for_header_candidate` (`region.rs:13674-13708`) 对任何不匹配的指令返回 `None`。因此目前没找到能在有效 producer 路径里把同一个调用/字段 SSA 值复制到两个表达式位置、再由 Builder 重复执行的已证反例。`test_is_pure` 对调用/字段确实只证明“被 branch 值依赖”，未显式声明全局单消费者；建议专门固定共享 effect 的拒绝控制，避免后续扩大 Duplicate/Copy 准入时破坏此前提。

Builder 的 loop condition 仍逐条 `test_expr(test_bci, taken)` (`build.rs:18813` 附近)，`condition` 用同一 branch 的 SSA operand 调 `render_value`；加 for 只改变头部语法，没有条件缓存或新表达式遍历。`render_value` 在 `build.rs:25923` 递归展开，`Invoke` 交给 `invoke_expr`，Field、ArrayLength、ArrayLoad/ArrayElementLoad 均在消费位置构建表达式。Pair condition 先 render 左操作数再 render 右操作数；Arithmetic 也按 stack operands 左到右 render。归档的 `ForConditionEffects` 把 `offset()` 与 `limit()` 记录为不同事件，并包含正常、零/多次迭代及首次/后续调用抛错；controls acceptance 记录成功完整类运行与原字节码 oracle 的行为一致。它证实旧 while 的先后与异常基线，不能代替应用后真实 for 的验证。

`Negate` 本身是无副作用的一元运算，Builder 读取一个 stack operand 并在消费点构成 `ExprKind::Neg` (`build.rs:26300` 附近)。若其嵌套 operand 是调用，当前没有搬动点；其条件内顺序仍是 operand 后取负。`NumericComparison` 有更强 Builder 门：`numeric_comparison_reader` (`build.rs:22517-22590`) 要求一个 stack result、恰好一个 use、同 block 的紧邻 zero branch，并且该 branch 直接读取该结果；`numeric_condition` 按比较指令的左右 stack operands 构造表达式。数值比较两端可能有调用，静态树按左到右 render，没有观察到会把比较移动到初始化或正文的路径。归档 controls 目前仅含 int 比较与整数取负，不能覆盖 long/float/double 比较、NaN 关系或 `MIN_VALUE` 取负边界。

数组 producer 的单 reader 门直接针对“会抛异常的读取”避免表达式树复制 ArrayLength/ArrayLoad。Builder 先渲染 array 再 index，保持 receiver/index 顺序；表达式仍在 branch 消费位置，所以 NPE、AIOOBE 和正文修改 array/value 后的逐轮读取位置没有静态变化。归档 `ForConditionReads` 已含 array length 正常/null，array element 正常/越界/null，以及正文改变 field bound、负字段和负参数的 int negate，原行为记录齐全；应用后仍需要用完整重编类核查 for 结构、异常类型和来源映射。

Continue 的更新路由未由本 match 修改。`header_tested_loop` 在 `ForHeader` 成功时把 `proof.update_block` 作为 `body_frame` 的 continue target (`region.rs:12805-12818`)；Builder 在 body walk 前把 `update_bci` 设为 settled，再把那一条准确更新语句构造成 `StmtKind::For.update` (`build.rs:18858-18960`)。Java `for` 的 `continue` 会先执行 update 再检查条件，这与旧 bytecode 的 update latch 路径相符。归档有 `sumWithFunctionBoundsAndContinue`，但它只验证当前 while 输出；S5、Overlap、ThreeLevel、OuterContinue 现有夹具可守住本改动与嵌套/外层 continue 的交界。静态走读未发现目标被改到 header 或重复执行的路径，必须由应用后源码/运行回归确认。

## 发现和回归清单

没有确认此次 producer match 会改变一次性 producer 的副作用次序、异常位置、数组边界行为或 continue 更新次数。当前需特别固定的隐式前提是 Invoke/Field Read 的唯一消费由 `Duplicate`/Store/Copy 不准入间接保证，而不是 `test_is_pure` 本身显式证明。新增共享调用/字段值的真实负控制，并确认应用后保持旧表示；若 fixture 以可执行源码表达困难，可在已有字节码变换控制中构造 duplicate SSA consumer，需检查 recovery 不误投影并保留证据。

以下回归尚未由本审查运行：

1. controls archive 的 `ForConditionEffects`：`sumWithFunctionBounds` 的 0、2、4 次边界检查；offset/limit 首次与后续抛错；`sumWithFunctionBoundsAndContinue`；`sumWithChangingFieldBound`。两种 JDK 编译，default/all 恢复，完整生成类 `-Xverify:all` 重编执行，比较 events、calls、返回值与异常。
2. controls archive 的 `ForConditionReads`：field bound 动态修改、array length 正常/null、array element 正常/越界/null、negated field/parameter；再加 int MIN_VALUE，以及 long/float/double NumericComparison 的 unordered/NaN 控制。检查 producer/init/update/branch 所有物理 BCI 和 UTF-8 span。
3. `ForConditionRefusals.sumWithLeadingTickAndBreak` 与 `sumWithAssignedCallBound` 作为旧拒绝边界；前者已有 fallback/source-map gap 是独立债务，本审查不将它算入此片结论。
4. 共享调用/字段读取的多消费者控制，确保 `Duplicate`/Store/Copy 仍使 ForHeader 不能证明；分别覆盖 volatile field read 与可能抛异常的 call，核调用次数和读取次数未因构造表达式而增加。
5. Continue 回归：归档 `sumWithFunctionBoundsAndContinue`，加 `tests/fixtures/shared-latch-nested-loops/{S5,Overlap,ThreeLevel}.java` 与 `tests/fixtures/p3-loop-transfers/OuterContinue.java`。核 direct continue、内层 continue、标号外层 continue 都命中目标 loop 的 update 一次后再评估条件。
6. 既有 unit/fixture 回归：`region.rs` 中 `cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir` 与 `shared_stack_initializer_has_two_real_dup_outputs_before_builder_refusal`；`build.rs` 的 `ordinary_if_and_real_loop_keep_their_region_forms` 和现有 numeric-comparison/array-read controls。前两者保护现有 ForHeader/dup 的 proof/refusal 语义，但不单独覆盖新 condition operation 清单。

controls archive 是候选应用前的 baseline：57 命令、114 raw streams、19 oracle lines；acceptance 记录 12 次 generated compile success、8 failures（failures 无 runtime）、所有成功行为一致。其 4 个 `ForConditionRefusals.sumWithLeadingTickAndBreak` source-map gaps 已明确标为既有债务。该归档没有新候选的完整生成 for 输出，所以它不能证明当前应用已经通过上述行为检查。
