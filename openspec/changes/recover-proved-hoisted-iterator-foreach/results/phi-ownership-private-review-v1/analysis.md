# Hoisted iterator Phi ownership：局部 proof 设计审阅

本稿只读审阅 root 的真实 v3 same-IR 结果和现有 SSA/foreach proof。没有改共享源码，也没有运行工具链。

## 实际观察

冻结输入是 JDK 23 原始 `cf11/UpstreamNestedLoops.class`，method `test(Ljava/util/List;Ljava/util/List;)V`。root v3 guarded run 为 2 commands，test 和 clean 均 exit 0，恢复两份临时源码并确认 target 已清理；原始 command evidence 和 streams 见 `pins.json`。以下 IR 事实来自 `/private/tmp/jarde-hoisted-iterator-same-ir-diagnostic-root-v3/0.stderr.raw`，不是静态猜测：

- inner loop 的 init/test/next 在 BCI 33/35/45，三处都映射到 `LocalVariable { slot: 5, index: 0 }`；placement 是 `Elevated { owner: [1] }`，该声明已写入 owner region。越过这个已证明的 owner 查找后，真实首拒是 `iterator_value_and_phi_use_ownership`。
- init Store 定义 iterator `V37`。`V37` 有普通读 BCI 35、45，以及 None-use：header block 35 一条、block 97 三条。
- header `Local(5)` Phi `V12` 的 inputs 为 `[Value(V37), Itself]`，`V12.uses=[]`、`V12.replaced_by=V37`。
- block 97 的 `Local(5)` Phi `V22` inputs 为 `[Value(V37), Value(V37), Value(V37)]`，`V22.uses=[]`、`V22.replaced_by=V12`，即有界 replacement 链 `V22 → V12 → V37`。三条 `None@97` 记录与三条 `V37` input occurrence 数量吻合。
- block 97 还有 `Local(6)` Phi `V23`，inputs 是 `V43` 三次，`uses=[]`、`replaced_by=V43`。不能因为找到 `Local(5)` 的一个匹配 Phi 就忽略同 block 的其他 Phi；也不能把这个无 reader 的历史 alias 当作 `Local(5)` 的一条 use。

SSA 的 `SsaUse` 把 phi operand 标成 `bci=None`，只保留 block，不保留 phi slot 或 operand ordinal。`SsaTable::replace` 的文档把 uses 明确称为 occurrence multiset；被替换 phi 保留 inputs 与 origin 作为它代表过的 merge，而所有实际 reader 移到 replacement value。因而 `V37` 的三条 `None@97` 是旧 merge operands 留下的记录，不是三条新的 JVM 指令读取；但它们仍是必须逐 occurrence 对账的证据，不能裸忽略。

## 最小局部门

建议只在 `iterable_for_each_candidate` 内替换当前 None-use 分支，不增加 `ssa` / `reuse` API，不做全局 dead-code 或 liveness 分析，也不追踪一般 alias 运算。复用当前 `resolves` 的 `0..=phis.len()` replacement 上界；循环上界仍由当前 SSA facts 决定。

对当前 iterator value 的每条 use，保留原规则：带 BCI 的 use 必须是已证明的 test/next local load；`bci=None` 的 use 按 block 分组。每个 None-use block 都执行一次完整核验：

1. 扫描该 block 的全部 `SsaPhi`，筛出 inputs 中包含当前 iterator value 的 matching Phi。每个 matching Phi 必须至少含一个该值；`phi.value()` 必须有 replacement、`uses()` 为空，并由 bounded `resolves(phi.value())` 到同一 iterator value。所有 `Value(input)` arms 也必须 bounded-resolve 到该 iterator；`Itself` 可以保留。这样既核 replacement target，也拒绝 mixed arms。
2. 精确对账 occurrence multiset：当前 iterator value 在该 block 的 `bci=None` use record 数必须等于所有 matching Phi 中直接等于该 value 的 `PhiInput::Value` occurrence 总数。没有 matching Phi、数量少/多、重复记录未被重复 input 一一说明，都拒绝。不能只比较 `contains` 或按 block 去重。
3. 同 block 的其他 Phi 也必须逐个审查。若 Phi output 有 reader、没有 replacement，或其输入不是同一个 replacement representative（允许 `Itself`，Value arms 必须一致），则拒绝。这里允许的只是可证明无剩余 reader 的 trivial historical alias；它不算 iterator use。真实的 `Local(6) V23 → V43` 是这个控制：若此 Phi 改成 live 或 mixed/nontrivial，必须令整项拒绝，即使 `Local(5) V22` 本身完全匹配。
4. 所有 block 核验完后才允许原 iterator value ownership predicate 通过；后续调用、投影与移除清单仍须通过现有精确 call、store/load、Phi、effect/handler、body cast、consumer 检查。任何局部门失败仍是普通 `None` refusal。

`SsaPhi.inputs` 的 Value arms逐条 bounded-resolve 到 iterator，而 occurrence 对账仍只累计与当前 use record 同一 ValueId 的 operand occurrence。replacement pass 会把被替换值的 use/input 记录转移到目标，因此按 ID 对账能对上 `SsaUse` 多重集；若事实不相符就拒绝。

## 对抗性否例

- block 97 的 matching Phi 有一个 arm 指向另一个 iterator、null/unknown producer，或 output replacement 最终不等于 `V37`：拒绝。
- Phi output 虽 replaced 但仍有 `uses()`，或 input arms 混合不同代表：拒绝。
- 三条 `None@97` 只对应两条或四条 `Value(V37)` operands；一个 input 被计数多次却只用 `contains` 检查：拒绝。
- 同 block 另一个 slot 的 Phi 是 live（output 有 reader）、没有 replacement，或是无法证明单一代表的 nontrivial/mixed Phi：拒绝；存在一个安全的 Local(5) Phi 不能掩盖它。
- None-use block 找不到 Phi、存在 stack Phi/multiple candidate Phi 不能把所有 occurrence 解释完、replacement chain 有环或超过 bound：拒绝。
- 在 header 35，原始 `[V37, Itself]` self-carry 仍须作为一条独立 Phi 形状通过；不应把 `Itself` 算作 `V37` occurrence。
- Ordinary `Some(bci)` use 在允许的两次 local load 之外，或未来代码多出一次 same-BCI operand use：拒绝，不以 Phi alias 通过覆盖。

建议测试把允许和拒绝固定在小型 SSA/fixture 场景：当前 header + downstream trivial Phi 正例；缺/多一条 use record；matching Phi 的 mixed arm；same-block other-slot live/nontrivial Phi；nontrivial Phi 无 replacement；multiple matching Phis 的重复 occurrence。避免只测“找到一个 Local(5) Phi”。

## 现有证明边界与计费

`build.rs` 的 iterator proof 已有 bounded replacement closure 和 header-local Phi 形状检查；array foreach 的 length/index proof 同样限定特定 local Phi、输入 arms、replacement、loop writes 与消费者位置。最小改动应沿用这个风格，只增加 per-None-block 的完整 occurrence ownership check，不把其提升成可复用 liveness API。

`ssa.rs::replace`（约 1822 行起）说明了 replacement 的 occurrence multiset、Phi 输入改写和被替换 Phi 保留 operands 的规则。`reuse.rs::reference_values_stay_before_boundaries` 与 `trivial_phi_outputs` 通过 alias replacement、全 Phi arms、use-boundary 和允许操作做一项特定的引用生命周期证明；它们是私有函数，且不负责一般 Phi liveness，本地 foreach gate 不应复制成共享层功能。

当前 foreach proof 在 candidate closure 外先对完整方法一次性 `poll/charge(AnalysisSteps)`，使用 instructions/phis 的保守上界。新 gate若继续用纯 `Option` closure，不可在其中新增可能被吞掉的 budget/cancellation `Err`。应先计算并计入该局部门最坏访问数（每个 None-use block 扫描所有 Phi、对应 operand arms、每个 replacement hop最多 `phis.len()+1`），由外层 `charge(...)?` 返回真实 `StopReason`；或把局部 proof 提到带 `Result<bool, StopReason>` 的位置。不要把 budget stop 伪装成一般拒绝。实际新上界应由代码明确计数，不能只假定旧乘数一定覆盖新增 operands。

## 变更边界建议

这个 proof 是当前 hoisted-iterator foreach 特性的内部 owner/use gate，使用现有 SSA 语义，不需要改写 SSA 或 reuse 层；若证据与否例测试落实在接受中的 foreach change 内，不必另拆产品变更。未来如果要把“历史 trivial Phi ownership”抽成多个 recovery pass 共用的通用服务，再单独评估其 API、budget 与语义范围，避免当前改动引入泛化 liveness 框架。

以上设计是对 root 真实 v3 证据和当前源码的架构建议，尚未实现或验证该局部门。真实 block97 其他 Phi 的完整约束需在实现的同 IR 验证中确认；不得从本稿声称它们已通过任何产品 gate。
