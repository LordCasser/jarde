# CF12 后续切片：纠偏与冻结证据补充

本稿补正前一版，不修改仓库。CF12 RefusedNone 门已经有独立变更规划：`openspec/changes/project-integer-constant-names-with-refused-member-family/`。它不是 gate 之后的 next slice。精确落点是 `src/facade.rs::project_class_source_integer_constant_names` 的现有 member-family gate；不是 `src/class_source.rs` assembly。该变更尚待 root 按其门禁实施/复验。

Selected Inner boundary 也需按设计更正：读 `TestSwitchLabels$TestCls$Inner` 时，本类私有 `CONST_CDE_PRIVATE=3294` 可由现有同类候选命名；`return 2748` 是 outer-only，单 class 不能推断/命名，必须保留数字。新 gate 不绑定 outer 字段，不发布 child 文本。这是该变更 `design.md` 明文范围。

## 前稿第 2 项：CF-07 `counted@20` 已有精确后续闭合

- 冻结输入/观察器：`preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/original-sources/LoopCases.java` 与 `Runner.java`；原 class 路径 `.../cases/javac8-original/classes/cf07/LoopCases.class`，另有 javac23 original class。Runner 输入为 `andWhile(false/true)`、`counted(1,4)/(6,9)/(5,5)`、`lastIndexOf` 三种数组/范围调用；`counted(II)I` 的 join 源码观察是其中三组值。最初的共享输入也冻结在 `openspec/evidence/java-syntax-2026-10-10/cf07-loop-latch-baseline/baseline-root-v2/jadx-input/cf07/LoopCases.class.jar`。
- 已关闭记录：`preserve-proved-if-arm-join-origins/verification-root.md` 明确 29-command full-class replay 中四个 Jarde profile 新增准确 `counted@20` If 来源，`counted` 的所有 BCI 完整；正文、原来源恒同。其专用 replay README 将新增来源限制在 physical `goto@20→27` 的完整 If span。故该项不再列为待实现 gap；root 要用 bafb frozen CLI 复验，未复验前只说旧验收已通过，不断言当前 HEAD。

## 前稿第 3 项：CF-13 `SwitchLoopExits.walk(I)I` 已有精确后续闭合

- 原输入/观察器：`openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/original/cf13/SwitchLoopExits.class`；源输入 `input/SwitchLoopExits.java`；唯一 Runner `input/SwitchLoopExitsRunner.java` 调 `walk(6)`，期望 `38`。这与 2026-09-24 同名的 `run(...)` fixture 不同，勿混用。
- 已关闭记录：`recover-switch-local-join-before-loop-continue/proposal.md` 精确引用此 `walk`、BCI 49/55/58；`implementation-report.md` 核 selector/join/continue/source-map，完整 Java 8 重编运行输出 `38`。后续永久测试 `tests/p3_switch_loop_exits.rs::switch_local_join_precedes_current_loop_continue_without_duplicate_ownership` 直接 `include_bytes!` 该原 class，并保留 `nested_switch_and_continue_without_a_unique_local_join_stay_quoted` 作为不同形态负例；冻结边界 fixture 是 `tests/fixtures/p3-switch-loop-exits/v8/SwitchLoopLocalBoundaries.class`。故旧 `walk` gap 已被对应 OpenSpec 关闭；仍待 bafb 当前 CLI 独立重放，不称当前存在回归或仍有差距。

## 可以继续保留的候选

1. **CF12 outer constant binding**：作为 RefusedNone root gate 完成后的独立债务，仍是单 class 设计明确排除的范围；只针对 Inner 中 outer-only `2748`，先确认 full family snapshot/physical outer field 事实入口。不得把本类私有 3294 当负例。
2. **CF07 computed-init `for`**：`TestLoopCondition5` 是上游活动文本断言（一个 `for`、一个 `return -1`）；冻结完整类/Runner 是 `LoopCases.class`/`LoopCases.java`/`Runner.java`。`preserve-proved-return-arm-loop-latch-origins/results/computed-init-for-debt-root-v1.md` 记录 Jarde 仍输出先赋值的 `while/if`，运行相同；`region.rs::for_header_candidate` 的 `Push(Int)+Store` 门拒绝真实 `end-1` producer，另有 induction-slot 外部读取第二门。没有后续 OpenSpec 声称已修该精确呈现债务；先用 bafb 确认后再立项，不要把来源缺失或语义错误混为一谈。
3. CF13 若要排第三，先从账本残留（nested-switch shared-tail / 空 then 呈现）挑有真实正例的精确方法；`SwitchLoopAdjacent.nested` 现有证据本来是无唯一 join 的负例，应继续引用，不能改称待恢复 gap。现有本稿未找到能把该负例提升为新正例的冻结上游 class/observer，也未确认更晚 OpenSpec 已关闭所有残留，需先找真实 oracle 再排期。

尚未复验：本稿读取冻结输入、Runner/源码与 OpenSpec 历史记录；不运行 bafb CLI。前两项旧精确缺口是否在当前 frozen CLI 保持通过，由 root 后续实测裁决。
