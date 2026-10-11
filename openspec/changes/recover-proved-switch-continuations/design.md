# recover-proved-switch-continuations

## Context

证据包 `openspec/evidence/java-syntax-2026-10-11/cf12-upstream-remaining-root-v1/` 冻结了此前缺失的五个上游测试样例，以及 original、JADX、Jarde 的完整观察结果。其 `README.md` 和 `jarde-cf12-remaining-acceptance-root-v2.json` 记录 `cf12_complete: false`：TestSwitch2 与 TestSwitchWithFallThroughCase2 在 Jarde 的 default/all 配置下仍整类编译失败；TestSwitch3、TestSwitch4、TestSwitchSimple 的运行结果与 original 一致。证据包还记录 JADX 的 TestSwitch4 检查结果为 2234，而非 1234；FallThroughCase2 虽有重复代码警告，检查结果和 68 个输入组成的矩阵仍与 original 一致。这些是历史对照，不是本变更的 Jarde 结果预期。

root review 确认 FT2 在 `ArmsDoNotMeet@0` 拒绝；175 到 197 的续接尚未证实。TestSwitch2 的候选和重叠解释也仍是假说。修改生产行为前，须针对冻结输入诊断两个方法。

## Goals and Non-Goals

**Goals:** 若诊断证实拟议形状，则恢复受限的 FT2 switch 续接和 TestSwitch2 共享直接目标续接，同时保留精确 block 归属与源码来源位置。复用既有证明和遍历路径，再用完整生成类编译与执行、有界拒绝和 Stop 行为验证。

**Non-Goals:** 不改变无关 switch 模式，不按 BCI 顺序推断汇合点，不放松 canonical 控制流边闭合，不引入 labels，不改变 JVM IR/SSA/frame schema，不增加通用图库或公开 API，也不合并单独排队的常量名 gate。不把旧配置输出记作新通过证据，也不因这两个方法宣称 CF12 或 71-unit 全面完成。

## Decisions

1. **先观察，再实现。** 对冻结方法做临时私有诊断。FT2 先记录已确认的 `ArmsDoNotMeet@0`，再核实内部 switch 是否证明汇合点 175、返回的 `next`、现有 `continue_switch_arm` 的输入和结果、外层边界 197、block 归属与访问差异，以及实际拒绝位置。175 到 197 仅作待验证假说。TestSwitch2 记录直接后支配节点、每个 `switch_forward_join` 候选及拒绝原因、canonical 前驱检查和首次后续重叠或拒绝。验收前移除仅用于诊断的插桩。如果实际原因不在拟议续接点，不得套用猜测的尾部路径或有向无环图修复。

2. **FT2 仅在观察证实后沿用外层 frame。** 复用 `continue_switch_arm` 现有的完整入边、block 归属和访问差异检查。如果实际形状确为已证明的 switch 在 175 结束，后接到外层边界 197 的有界直线路径，则在外层 frame 中消费该路径；175 留在 case body 之外，后续语句在 switch 后只输出一次。不预先假定该形状，也不增加 helper。遇到分支、循环、嵌套 switch、外部入口、非普通控制流边、未覆盖 block 或遍历差异不匹配时拒绝。Stop 保留预算实际原因、维度和位置。

3. **TestSwitch2 复用既有证明器。** 仅当诊断证实汇合点发现是相关拒绝点时继续。枚举有界的直接目标候选：将候选组从 `targets` 移除，把候选作为已知汇合点传给现有证明过程，沿用现有三色遍历、路径闭合、边类型检查和 block 归属证明。仅为此候选路径补上现有结果未提供的证据：哪些已归属路径来源到达候选，以及候选处完整的 canonical 入边。候选入边必须互不重复且均为普通边，来源集合必须恰为 switch dispatch 加已证明的路径来源。还须证明至少一条路径到达候选、至少一个物理 Return/Throw 终端，并且恰有一个候选通过；零个或多个候选都拒绝。更严格的汇合点入边校验只用于候选发现，避免无条件加严现有共享边界调用。循环、嵌套 switch、未知叶子、额外后继、非普通 canonical 边、未证明的 case-entry crossing、外部入边或非汇合点 block 的归属重叠均拒绝。不得复制第二套深度优先遍历，也不得只依赖 normal-flow 投影。

4. **汇合点和终端归属以物理 block 为准。** switch arm 对其内部 branch 和准确的 terminal block 各拥有一次。候选汇合点不属于任何 arm body，由外层续接消费一次。既有 break/return 构造和已接受的 `SwitchBreak` consumer 仍为准。block 不能仅因被访问过就算作已归属；证明失败时不得留下部分消费状态或部分源码产物。

5. **保留共享预算与 Stop 语义。** 证明和遍历使用请求已有的 `Budget`；处理 node、successor、canonical row、predecessor、candidate 前按实际工作计费，并在有界扫描中检查取消。汇合点发现的 Stop 保留实际 dispatch 位置；尾部路径消费的 Stop 保留实际 continuation/`next` 位置，不得统一改写为 switch dispatch。不得发布部分 body/map。不得提高预算或新增预算维度来通过测试样例。

6. **私有候选，root 串行集成。** 两个证明形状可以由不同 Luna agent 分别准备私有候选补丁。agent 不修改工作区、不建分支或 worktree。root 审查两个候选后，逐个集成到同一个 `region.rs`；解决重叠后再验证。最终生产 diff 必须是一个连贯的本地改动。

7. **整类验收由 root 新建比较。** 实现候选稳定后，冻结新的 CLI 及源文件、类文件、辅助文件、运行时 pins。对全部十个 CF12 测试样例的 original、JADX、Jarde default/all 完整类重新编译和运行，保留上游检查与完整辅助类路径；比较标准输出/错误输出和实际生成类集合，并验证两个改动方法的物理源码来源位置。逐配置记录实际通过、拒绝或编译失败。旧 Jarde 失败和旧 CLI baseline 仅作历史观察，不算此候选的正向证据。

## Reuse, Maintenance, and Licensing

本变更不需要引入外部图遍历库。控制流遍历必须读取本项目的 canonical 控制流图、normal-flow 视图和物理 block 归属，并遵守现有 `Budget`/Stop 契约；通用图库无法直接提供这些项目级证明。TestSwitch2 应扩展已有 `prove_switch_fallthroughs` 的扫描结果，避免维护第二套遍历。该证明器当前缺少候选汇合点的入边/来源见证和内部终端结果；这些是明确的能力缺口。FT2 则先复用 `continue_switch_arm` 的归属与访问差异校验，只有诊断确认需要消费外层尾部路径后才做最小扩展。

沿用项目现有 Rust 代码与私有证明器，不新增 crate 或依赖，维护责任仍在现有 `jarde-java` 模块。由于依赖集合不变，不引入新的第三方许可义务。若实现过程中发现必须增加依赖，应先记录该库的维护状态、许可证兼容性，以及现有证明器无法满足的具体能力，再调整设计。缺失能力只补候选路径所需的证明见证，不新增通用配置框架。

## Risks and Trade-offs

- 共享的普通后继可能隐藏 canonical exception/call/subroutine 边。将完整 canonical 入边与路径证明对比；遇到不支持的边类型时继续拒绝。
- 续接可能重复输出或被错误归给 case。发布组合结构前核对 block 归属集合和遍历差异，并检查输出次数与源码范围。
- 宽松的公共节点搜索可能把 loop header、case entry 或 outer join 当作 switch 汇合点。通过候选唯一性、解码目标约束、入边归属闭合、无环证明和显式 frame 边界控制风险。
- 首次观察到的失败可能早于拟议消费路径。诊断是实现前置条件；若证据否定拟议形状，须调整目标或停止该形状的实现。

## Validation and Delivery

conditional 变更独立验收并干净交付后，root 才进行工作区编辑和工具链验证。验证真实方法、既有 switch 回归、对抗性拒绝用例、预算/取消行为、OpenSpec strict checks，以及同一新冻结 CLI 下的十个测试样例完整类比较。保留原始失败和精确 pins，不预填数量，不混淆 Jarde 与 JADX 结果；相关独立验收完成前，不更新 71-unit completion ledger。

## Pipeline Boundaries

读取和指令解码不改；canonical CFG 与 SSA 的物理事实仍取同一次分析。Java 8 方言准入沿用既有 RecoveryProfile，runtime/JDK 选择仅用于固定输入的编译与差分验证，不扩展产品运行时解析。此变更只补源码结构恢复的证明和消费；解析成功、结构恢复成功、完整源码重编通过及行为一致分别记录，不能互相代替。用户已授权自行构造并执行本项目 Java 对照测试，原始上游 check 不关闭。
