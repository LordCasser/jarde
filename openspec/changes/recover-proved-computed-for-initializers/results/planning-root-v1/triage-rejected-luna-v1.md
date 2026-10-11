# CF12 常量名 gate 后的三个窄切片

只读排序；不代表任何 CF 单元完成。优先级依据是当前 ledger 中仍明确登记的可复现差距和现有实现接缝。`benchmark-ba2076a/handoff.md` 记录的是性能复测协议裁决与未测范围，没有 Java 恢复缺口，故不把 benchmark 项混入产品切片。

先校正状态：inventory `summary.md` 中 CF12 行保留了 2026-10-11 类型/条件工作尚未实施的历史文字。后续证据显示 `recover-proved-local-source-types` 已完成其局部差距，`recover-proved-compound-statement-conditions/verification-root.md` 已接受 Switch2 与 FT2 的整类结构/行为对照；它仍明确“不宣称整 CF12 或 71 单元完成”。本排序把已解决的 char/null-first、条件 fall-through 和早退共享 continuation 排除。旧 integer constant-name gate 的嵌套根成员门仍是实测残差。

## 1. CF-12：根类 `TestSwitchLabels.test` 的常量名投影

- **精确目标：** JADX `jadx.tests.integration.switches.TestSwitchLabels.test`，真实上游 class 为 `TestSwitchLabels$TestCls`，并保留 `Inner`。对应基线表见 `openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/README.md`。default/all 行为一致，但 case/return 仍为数字；渲染 Inner 为独立完整类。
- **Jarde 事实与任务：** 旧 `recover-int-array-constant-names` 已证明同类唯一 `static final int ConstantValue` 的 switch/return 常量名投影；新真实类差距具体是 projection 被 `member_family` Refused 跳过。先独立诊断“family 未发布时是否能安全投影根类方法”，不要一并把 Inner 的 `CONST_ABC` 跨成员绑定当作既成事实。Inner 的私有 `CONST_CDE_PRIVATE` 明确不能作 alias。证据称后者属于额外绑定范围。
- **入口与复用：** `src/class_source.rs::integer_constant_projection_text` / `integer_constant_projection_span`，整数标签输出 `crates/jarde-java/src/emit.rs::switch_key`，完整类 family 准入在 `src/class_source.rs` 的 member-family assembly。现有唯一值、字段表、作用域遮蔽及精确来源门可复用。第一阶段不必新 IR/Frame/pass；跨成员绑定是否需新证明尚待核验。
- **边界：** 只做根方法投影并保持 Inner 原呈现可作为窄首片；不等于 CF12 完成，也不处理 enum switch (`project-proved-enum-switch-labels` 是另一类跨类事实)。

## 2. CF-07：计数 `for` 的 if 汇合跳转来源

- **精确目标：** 单元 CF-07。上游代表测试集包含 `loops/TestLoopCondition2.java`、`TestLoopCondition5.java`、`TestLoopDetection2.java`、`TestComplexWhileLoop.java`、`TestLoopConditionInvoke.java`（`control-flow.md` 表格）。缺口映射到受控完整类的 `LoopCases.counted(II)I`，不要声称它是上游某一条断言的精确方法映射；这是需要重验的地方。
- **Jarde 事实与任务：** `cf07-loop-latch-baseline/README.md` 记录行为已一致，但 `counted` 的 `goto@20→27`（if 汇合）及 `goto@30→6`（回跳）曾缺来源。return-arm 来源片只补回跳：最新 `control-flow.md` 仍列 `counted@20` 未覆盖；本项仅瞄准汇合边的来源归属，不改循环形状或文本。现有 LoopCases 与双 JDK 完整类对照可作冻结回归锚。
- **入口与复用：** `crates/jarde-jvm/src/region.rs` 的 `loop_arm_join_source` / gateway-origin 收集与 `crates/jarde-java/src/build.rs` 的 `gateway_origins` 来源并入。可沿用 canonical edge 与现有 join-owner 证明；不应按“缺 BCI”给任意语句补来源。新 proof predicate 是否必要取决于当前真实 `counted@20` 边在 walker 中的 owner，尚未插桩复验。
- **关联任务：** `preserve-proved-return-arm-loop-latch-origins` 已关闭，但明确把 counted@20 和 lastIndexOf@25 留为独立来源债务；后者已由后续 return-arm 片补齐，避免重复选它。

## 3. CF-13：switch-local break 与外层 continue 共存时的共享出口归属

- **精确目标：** 单元 CF-13；ledger 代表测试含 `switches/TestSwitchBreak.java`、`TestSwitchContinue.java`、`TestSwitchInLoop4.java`。其中 `TestSwitchBreak` 有带标签 loop-exit 的 TODO，`TestSwitchContinue` 只有文本断言，不能把上游测试通过当作行为 oracle。
- **Jarde 事实与任务：** `openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/README.md` 的完整类 `SwitchLoopExits.walk(I)I` 在 selector BCI 9 因 Region 多重 owner 拒绝，虽然原 class 输出 38；JADX 生成源码在 continue 后留不可编译的 unreachable break。已有 `recover-switch-arm-loop-exits` 修复 W1/W2 的普通 switch-arm continue，但当时把嵌套 switch/shared-tail、labeled continue 和空 then 呈现列为后续边界。优先做该冻结完整类的 ownership-only 诊断/切片，不先扩成一般嵌套 switch。
- **入口与复用：** `crates/jarde-jvm/src/region.rs` 的 `switch_region`/loop-exit arm ownership 与 `Region::LoopContinue`；既有 `p3_switch_loop_exits` 正负边界和 canonical CFG ownership 可复用。证据建议分别证明 switch 内 join 与 loop-update edge 的唯一 owner。暂没有证据要求新 Region/IR；是否需要一个局部 exit classifier，需看实际拒绝阶段后决定。
- **关联任务：** `recover-switch-arm-loop-exits/design.md` 把 labeled continue 和更复杂交叉留作 Non-Goal；其简单 continue 回归已通过，不能据此宣称 CF-13 整体完成。

## 排序建议

先做第 1 项的只读诊断，因为它最直接承接 constant-name gate，而且可以把根类与跨成员两种证明责任切开。第 2 项适合作为低范围来源修复；第 3 项有真实完整类失败，先定位 owner 再拟局部扩展。三项都应独立冻结/验收，不更新 CF-12/07/13 整单元完成数。

尚未复验：本稿只读取既有 ledger、OpenSpec 和源码入口；没有重放 JADX 测试、class-source、编译或运行。尤其第 2 项 `counted(II)I` 与具体上游 JUnit 断言的映射，以及第 3 项当前 HEAD 首个拒绝阶段，仍待单独受控核实。
