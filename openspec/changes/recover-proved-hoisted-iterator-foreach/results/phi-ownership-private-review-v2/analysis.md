# Hoisted iterator Phi ownership：必要条件修订

本稿修正 v1 的 same-block Phi 约束。review 只依据 root 的真实 same-IR v3 raw 和当前 SSA/foreach 源码；没有写产品实现、修改共享源码或运行工具链。

## 实际证据

冻结输入是原始 `cf11/UpstreamNestedLoops.class` 的 `test(Ljava/util/List;Ljava/util/List;)V`。root v3 的 guarded exact test 和 clean 均 exit 0，临时 build/facade 两文件恢复精确、target 已清理。具体命令、raw 和源码哈希见 `pins.json`。

真实 raw 记录：init/test/next BCI 33/35/45 都映射到 `LocalVariable { slot: 5, index: 0 }`；placement 是 `Elevated { owner: [1] }` 且 owner 声明已发出。越过 owner 查找后，首拒为 `iterator_value_and_phi_use_ownership`。iterator store 定义 `V37`，其 None-use 记录在 header block 35 有一条、block 97 有三条。

- header `Local(5)` Phi `V12` 输入 `[Value(V37), Itself]`，`V12.uses=[]`，`V12.replaced_by=V37`。
- block 97 `Local(5)` Phi `V22` 输入 `[Value(V37), Value(V37), Value(V37)]`，`V22.uses=[]`，`V22.replaced_by=V12`，所以 `V22 → V12 → V37` bounded-resolve 到 iterator。
- block 97 的 `Local(6)` Phi `V23` 使用 `V43` 而不含 `V37`。它可以是 live 或 nontrivial；仅凭该 Phi 不得拒绝 iterator 的 owner/use proof。若某个其他 slot Phi 实际包含 `Value(V37)`，它就属于 matching set，必须满足与 Local(5) Phi 相同的 source-input 证明。

## 修正后的最小局部门

只在 `iterable_for_each_candidate` 的 iterator-value ownership gate 内核当前 source 的 None-use。保留直接指令 use 只能是已证明 test/next local load 的规则。对 `ssa.value(iterator_value).uses()` 里的每个 `bci=None` record，按其 `use.block()` 分组；`SsaUse` 没有 slot/operand ordinal，因此依赖 SSA replacement pass 保证的 occurrence multiset 来核验：

1. 对该 block 扫描全部 Phi，matching Phi **仅指其 `inputs()` 中出现精确的 `PhiInput::Value(iterator_value)`** 的 Phi，不按 slot 名或 replacement representative 扩大集合。
2. 每个 matching Phi 必须有当前 source 的实际 input occurrence；其 `phi.value()` 必须有 replacement、`uses()` 为空，并通过现有有界 `resolves` 到 iterator value。每个 Value arm 通过同一有界 replacement 链 resolve 到 iterator value；`Itself` 允许。任何其他代表、未替换/仍有 reader 的 output 或 mixed arm 都拒绝。
3. 将这个 block 中 source 的 None-use record 数，与该 block 所有 matching Phi 的 `Value(iterator_value)` 精确 operand occurrence 总数比较，要求相等。重复 operand 按重复次数计数；只有 set membership/contains 不够。若 block 有 source None-use 却没有 matching Phi，或数量多/少，拒绝。
4. 不对完全不含精确 `iterator_value` input 的其他 Phi 增加“必须 inert”门。它们不产生这个 ValueId 的 use record，也不参与当前 source 的 occurrence 对账。若某个 other-slot Phi 含有精确 source input，它自然会被匹配并按第 2 项检查；因此 live/nontrivial source-carrying Phi 仍拒绝，而无关 live/nontrivial Phi 不阻挡本项恢复。

当前 header Phi V12 与 downstream V22 分别以 one occurrence 和 three occurrences 对上 None-use multiset；实际 Local(6) V23 是无关控制例，应排除在 V37 ownership 判定之外。后续所有 iterator call/store/load、handler/effect、body cast 与输出删除条件仍沿用现有 proof，不因这个局部门而放宽。

## 精确 ValueId 计数与 bounded replacement 的区别

计数阶段必须用精确 `ValueId` 相等：只数 `PhiInput::Value(V37)`，因为 `SsaUse` 是该 ValueId 的 def-use occurrence 记录。不能把所有 `representative(input) == V37` 的输入都计成 V37 的 use；那会把不同 alias ID 的记录并进来，可能重复或错配 multiset。

语义证明阶段则允许 bounded replacement：一旦某个 Phi 已由精确 input 匹配，要求 Phi output 和它的每个 Value arm resolve 到 V37，确认它没有把不同运行值并到 iterator 上。`Itself` 表示该 Phi 自身的循环 carry，不是一条 `Value(V37)` occurrence，也不增加 source use count。

`crates/jarde-jvm/src/ssa.rs::replace`（约 1822-1940 行）明确说明并实现这一点：use records 是 occurrence multiset；`None` records 逐 Phi operand occurrence 计数；替换遍历实际 Phi inputs，按精确 `from` ValueId 改写并核对 `operands_named == operands_rewritten`。被替换 Phi 的 operands/origin 刻意保留，用于记录其代表的 merge；replace 的 `replaced_by` 说明该 Phi 不再是实际定义，实际 users 已移给 replacement。这解释了为何 V22 可留下三个 historical source operands 和 V37 None records，同时 V22 本身没有实际 reader。该协议支持按 block 精确计数，不支持不经证明地忽略这些 records。

## 对抗性控制

- 正例：header `[V37, Itself]`、downstream `[V37,V37,V37]`，相应 use 记录分别为 1 和 3；bounded output chain 都到 V37。
- matching Phi 缺 replacement、output 仍有实际 uses、Value arm resolve 到其他 iterator/value、或有无法 bounded-resolve 的环：拒绝。
- 当前 block 有 3 条 source None-use，但 matching exact input 数只有 2 或 4；或重复 input 只用 `contains` 覆盖：拒绝。
- other slot Phi 不含精确 V37（例如 V23）：不影响 V37 gate，即使该 Phi 自身 live/nontrivial。反向控制是 other-slot Phi 含精确 V37，但为 live 或 mixed/nontrivial：必须匹配并拒绝。
- source None-use 所在 block没有 matching Phi：拒绝；同 block多个 matching Phi则全部检查、合并精确 occurrence count。
- `Itself` 不计作 `Value(V37)`，但 matching Phi 的其他 arms必须都 resolve 到 V37。
- `Some(bci)` use 超出已经绑定的 test/next loads：拒绝，不受 None-use 通过影响。

## 边界与预算

现有 iterator proof 已有 `resolves` replacement bound，并核 header local Phi 的 input 形状；array foreach 也对 induction/length local Phi 逐项核 input、replacement、实际 load/update readers 和 loop writes。应沿用此局部证明风格，不添加 SSA/reuse 通用 API。

当前候选 proof 在 Option-returning闭包外先以 `poll`/`charge(AnalysisSteps)` 为整个 SSA/Phi proof预付成本。新增按 block 扫描、Phi operands和每个 replacement hop必须在该预付上界内显式计数；若预算无法保证，应把局部helper放到可返回 `Result<bool, StopReason>` 的边界并传播 cancellation/budget Stop，不能把 Stop 转为普通 `None`。不要声称已有乘数自然足够。

## 变更边界

这是当前 foreach owner/use proof 的局部补充，适合留在已接受的 hoisted-iterator foreach change 中，连同上述正反控制一起验证。它没有必要要求不相关 Phi inert，也不需要产品级 liveness 机制。若未来要为多个 recovery 统一抽取 Phi ownership/deadness 服务，应另拆提案并单独证明它的接口、budget 与语义边界。

本稿是架构建议，不是已实现的证明或产品验收结论。block 97 的现有事实来自 root v3 raw；新局部门尚未实现或验证。
