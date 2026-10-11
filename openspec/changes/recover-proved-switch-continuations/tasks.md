## 1. Preconditions and real-path observation

- [x] 1.1 等待 `recover-proved-conditional-switch-fallthrough` 完成自身 CI 精确验收并记录干净交付；在此之前不应用本变更。验收方法：检查该变更的 CI 结果和最终交付记录均可追溯且通过。
- [x] 1.2 重新读取已验收的 `cf12-upstream-remaining-root-v1` README、root acceptance、冻结 source/class pins 和两结构计划；冻结新的只读候选 baseline，并把旧 Jarde 失败保留为历史失败。验收方法：候选记录包含新 baseline 的输入标识/pins，并明确旧失败没有被记为新通过。
- [ ] 1.3 仅在私有诊断运行中定位两个方法的实际首个拒绝。记录已知 FT2 `ArmsDoNotMeet@0`，再确认或推翻 `next=175` 到外层 boundary 197 的假说；确认或推翻 TestSwitch2 候选/重叠假说后再选生产补丁。验收方法：诊断记录展示首个拒绝位置、各相关 proof 输入/结果和 ownership delta，且不包含仅供诊断的生产改动。

## 2. Local proof changes and focused tests

- [ ] 2.1 为 FallThroughCase2 准备私有的有界 tail continuation 候选；复用既有 switch-arm/continuation 校验并保留 enclosing-frame ownership。验收方法：真实类聚焦用例证明 join 留在 case body 外、tail 只恢复一次，超出直线 tail 的形状仍拒绝。
- [ ] 2.2 为 TestSwitch2 准备复用 `prove_switch_fallthroughs` 的候选；仅增加候选范围内的 join-source/canonical-incoming 和物理 terminal 见证，并要求候选唯一。验收方法：真实类用例证明唯一候选、准确 join incoming source 集、terminal outcome 和空 fallthrough 结果；零个/多个候选均拒绝。
- [ ] 2.3 由 root 审查两个候选并串行合并进同一 `region.rs`；不建分支或 worktree，保持最终生产补丁连贯。验收方法：审查记录确认两个候选已逐个集成，最终 diff 无重复证明器或互相覆盖的实现。
- [ ] 2.4 增加真实类聚焦覆盖，证明 FT2 内部 join 不归 case 所有、外层直线 tail 只输出一次，且 outer condition 与 switch-break origins 精确。验收方法：对固定 FT2 输入检查结构化输出、block-owner/visited delta 与物理 source origins。
- [ ] 2.5 增加真实类聚焦覆盖，证明 TestSwitch2 shared continuation 只输出一次、每个 early return 仍位于原路径且 source origins 精确。验收方法：对固定 TestSwitch2 输入检查每条 arm 的结构化结果、输出次数和物理 source origins。
- [ ] 2.6 只添加这些证明所需的负向 ownership 边界：歧义候选、外部 predecessor、共享非 join block、无效 case-entry crossing、cycle/nested switch/unknown terminal 和非 Normal canonical edge。验收方法：每种构造均触发既有拒绝/coverage 契约，且没有部分消费或重复输出。
- [ ] 2.7 在新增扫描和 tail consumer 中检查真实预算与取消 Stop；核实适用时的 dispatch 或 continuation/`next` 位置、实际 usage，以及部分 source/map 的原子缺失。运行既有 straight-fallthrough、grouped-label、switch-break、loop 和 control-flow 回归。验收方法：Stop 用例逐项核对 reason、dimension、location、usage 和空部分结果；指定回归全部通过。

## 3. Root-owned complete validation and delivery

- [ ] 3.1 在 root 资源限制下运行格式化及精确适用的测试/lint 检查；原样保存每个失败，不扩大预算。验收方法：提交格式化与检查命令、退出状态和原始失败记录，所有要求项通过。
- [ ] 3.2 冻结新 CLI 及 source、class、helper、SDK、JDK 和 invocation pins；比较全部十个 CF12 fixture 的 original/JADX/Jarde-default/Jarde-all 完整类，保留上游检查、完整生成 class sets、runtime stdout/stderr 和物理 source origins。逐 profile 记录真实结果，不预设成功。验收方法：可复放的冻结记录包含完整 pins、十 fixture 的四组结果、class-set 差异、运行输出和两个改动方法的 origins。
- [ ] 3.3 独立核验候选回放、OpenSpec strict validation、产品 CI 和干净交付；仅依据验收证据更新 verification/ledger/handoff，不宣称本里程碑完成全部 CF12 或 71-unit ledger。验收方法：独立核验记录逐项引用已通过证据，并确认 ledger/handoff 没有超出证据的完成声明。
