# Design

## Context

动机见 `proposal.md`。已完整读取现有 `java8-recovery` 主规格及 scenarios、`prove-interface-super-source-legality` 全量 delta；后者已要求预算/取消停止如实报告，但尚不能由其规划或底层 helper 测试推出公共门面正确。

真实调用链为 `Engine::recover_target` → 方法分析 → `recovery_from_with_class_candidates` → `interface_super_calls_presented` → `jarde-java` → `RecoveryPresentation`。证明 helper 能返回 `Error::BudgetExceeded`，包装却将预算/取消转为空证书，注释声称 presentation 会保留停止。普通 `Budget::new` 无 ledger，失败的局部深度检查不留 sticky 状态；`poll` 只检查取消/期限。Java 恢复不消费 dependency depth，于是原停止不会自行回来。presentation 与 CLI 只消费报告字段，也没有可恢复原 Error 的旁路。

`results/repro-v1/` 保存测试 SHA `f285e92259533d5a55c2c351436c5107b68433d92774181d7285b6ac8b83420f` 与关键原始流。私有 HEAD `f01402867ffa20085e421a680a6ebbe878e63338`、13 候选文件和 build `b973da39…` 的组合实跑：宽预算 1 pass，depth0 1 fail；后者 analysis Complete、正文只含缺证明说明、recovery/presentation Complete 且 stop=None。该证据不绑定 primary 644 的编译，也不含取消、期限或 class-source 公共复现。

本变更处理源码恢复之前的执行边界；不改变 classfile parse、方言判定、runtime selection、dispatch 或 verifier 的事实，也不把 JVM 能运行视为源码 qualifier 合法性证明。

## Goals / Non-Goals

**Goals:**

- 在接口证明停止的那个分支结束恢复，保留原 `Error` 的预算维度/取消/期限语义与已完成的 analysis；复用既有停止产物，不新增平面。
- 单方法与 class-source 共用该边界，presentation/CLI 继续直接消费报告，不增加文本解析补丁。
- 成功证明与完整读取后的普通拒绝仍走现有路径，证明之前已合法发生的收费保留。

**Non-Goals:**

- 不改 `Budget` 全局行为，不增加 sticky stop、预算维度、通用 pipeline/type/hierarchy 服务或跨层反向依赖。
- 不改 snapshot/direct-parent/reference 证明包装、重跑文本筛选、class-view 或 emitter；snapshot 静态债务继续指向 `../recover-covariant-child-array-initializers/results/snapshot-proof-stop-debt-v1.md`。不能把这些路径称为本次已实测缺陷。
- 不把整个 class-source 的可靠前缀和兄弟成员清空；本方法在产物提交前停止，与允许交付已提交文本的 evidence-phase stop 不同。
- 不修改当前四工程或 primary 644 的交付/CI 门槛。新 fix 使用自身最终产品源码验收。

## Decisions

1. **在证明结果处显式分流。** 将 `prove_interface_super_calls` 的最窄私有结果通道调整为保留原 `Error` 或 resolver 原 `ExecutionReport` 与完整诊断集合；接口依赖读取显式选择该通道，其他 legacy caller 保持原行为；`Ok(empty)` 表示没有得到源码证书，预算/取消 `Err` 必须进入停止分支。普通 missing/ambiguous/redundant/abstract 结论不转换为执行停止。反对仅删除 catch 后直接向公共入口 `?`：合法请求仍须带 analysis 和恢复停止报告，不能变成裸 Error。

2. **最小 crate 边界停止构造。** 由拥有原 Error 的门面在正文恢复前选择停止路径；复用 `jarde-java::report` 的 `stopped` 字段初始化，通过一个最小的跨 crate 停止构造入口交付 `RecoveryReport`。该入口只组织现有请求身份、evidence pending、not-produced artifact、诊断和执行平面，不运行 Region/AST/emitter，也不重新读 class 或重跑分析。不要在 facade 手工复制一整份 RecoveryReport 默认字段。

   映射必须从原 Error 或原 resolver report 做，不能再试一次 `charge` 或 `poll` 重建理由。v3 counted 运行实际观察到原预算维度断言失败，raw未打印当前迭代维度或实际execution；结合有序测试与源码可归因：ClassHeaders 的 resolver report 停止后 ledger 已将共享 token 置位，再调用 legacy helper 会将原预算停止改成 Cancelled。v4的两个迭代已实际通过原维度与无后续entry断言；接口读取必须在这次报告返回时直接传播，不继续调用该 helper。原 Error 的 counted 数字原样保留；report-only 通道没有结构化 limit/consumed/requested，使用现有 `StopReason::Interrupted`，同时原样交付 report 的 execution（包括原预算维度）和全部诊断，不能解析诊断文字或从调用者预算推造 Error 数字。现有 `StopReason::Budget` 只容纳 counted dimension，`DependencyDepth`/`ElapsedMillis` 不能强塞为 OutputBytes 或 IrItems。选定方案是保留既有 `StopReason::Interrupted` 词汇用于这些非 counted 中断，由最小停止构造同时使用原 Error 生成明确的 `ExecutionReport::Partial/BudgetExceeded { dimension }` 与原 Error 诊断；取消保持 Cancelled，counted 预算保持原维度。报告初始化时直接写入这套一致状态，MUST NOT 在正文工作完成后再把 Complete 涂成 Partial。实现时应复用/对齐 facade 既有错误到 execution 的映射，避免复制第二套不一致规则；如需抽取，只抽取这一错误映射，不建通用运行框架。位置只保留真实已有的 BCI/物理方法，无法定位指令时声明无 BCI，不能伪造。

3. **保留分析与停止点的记账。** `RecoveredMethod.analysis` 仍是本次已发布 report；停止 recovery 只改变尚未交付的恢复部分。请求不执行之后的 reference/hierarchy proof、正文恢复及 optional evidence，不为诊断重新申请已耗尽的结果预算。class-source 继续使用已有成员 execution 合并，兄弟成员/可靠 raw prefix 保留；测试需证明停止后无 recovery 工作，而不仅检查末态标签。

4. **复用维护边界，不新增依赖。** 所需能力是本仓库 Error、物理身份、预算与 recovery artifact 的精确交接，现有 reader/JVM/Java 类型已齐。外部 parser、泛型 error crate 或层级库不能提供本请求的停止来源，也不能消除最小报告构造。依赖清单、维护和许可证面保持不变；不增加网络、JDK、宿主协议或第三方依赖。原 graph/reader 实现与许可不受影响。

5. **以真实公共路径验收。** 实现阶段将最小复现移入适当既有测试或保留独立 integration target；positive 必须真有 `DefaultProbe.super.value()` 和 Complete，negative 必须真到证明边界。确定性取消优先复用已有 caller-owned ledger observer，在一次真实 proof Charge 中置 token；证明初始 token clear、实际 Charge 已发生并被后续 checkpoint 观察，不能用预取消/sleep 代替。期限控制使用已有时钟/observer 能力，在真实 proof checkpoint 观察实际过期；必要测试 seam 只能定位该边界，不能注入一个假 Stop 取代 Error。记录期限状态和先前原因为何先发生，避免把 IR0 或入口停止误作证明期限证据。

6. **分层门禁与最终源绑定。** 先运行 bounded public regressions、正常拒绝与来源控制，再执行最终源码的适用格式/Clippy/workspace 等现有 gate 和必要完整生成源/JDK 对照。完整源门复用 special-dispatch/qualified-super 既有 fixture、wrapper、runner 与受控 JDK，不以单段正文或 JVM verifier 通过代替生成源编译。最终产品自身 commit 的当前项目 CI 成功才可写完成；已通过的 644 CI 或私有反例不代替新 fix CI，不发明额外无关门禁。失败 raw、确切 argv、环境、编译输入与 source pins 均保留。

## Risks / Trade-offs

- [非 counted 预算与既有 StopReason 不同粒度] → outcome 继续使用既有 Interrupted 词汇，execution/诊断必须保留原完整预算维度；用公共 DependencyDepth 与 counted 控制逐字段检查，禁止压成正文维度。
- [把正常缺证误作执行停止] → missing/ambiguous/abstract/redundant 输入在未超预算时继续走原保守拒绝，核对调用 BCI、receiver/argument origins 和效果；不给普遍停止留通过路径。
- [末态虽变红但停止后仍烧预算] → 在证明停止点记录 usage 与恢复工作观察量，断言没有之后的正文/evidence 计费；原准许的 Charge 不要求退还。
- [取消或期限控制只停在入口] → 使用初始 clear、真实 proof Charge/checkpoint 证据；不得以预取消、sleep 抽样或 prove-only helper 替代公共请求。
- [私有反例被当作主线成功] → 复现 manifest 明确组合基线；最终实现另绑最终源码、raw 和自身 CI，所有实现任务保持未勾选。

## Migration Plan

无数据迁移。先在实现基线上固定本次最小反例和成功对照；接入最小停止报告入口与门面分流；验证原始停止、记账和原合法/拒绝来源控制；完成最终生成源与现有适用门禁，取得新产品 commit CI 后再评估完成。失败时保留报告与前后 source pins，撤回该局部接线即可，不修改 Budget 生命周期。规划完成后停止，不在本轮实施。
