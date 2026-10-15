# Tasks

> 顺序约束：先完成 root 的实际 SSA/字节码诊断和范围裁定。以下设计项在前置条件通过前不可实施；诊断若不支持候选条件，应更新范围或关闭本 change，不近似放宽通用 phi/use 门。

- [x] 1.1 在 frozen admission 完整类中复核条件累加方法的实际拒绝链：从原始指令/块、producer、Stack slot、phi 输入 ValueId、`replaced_by` 到真实指令 reader 逐项绑定。证明重复 use 是否恰为两条相同输入 phi edge，并解释报告中的 consumer count；不得以单次诊断文本或猜测 BCI 代替证据。
- [x] 1.2 记录并评审物理 SSA `Value::Int` 候选边界：确认它只覆盖既有 Folded 条件值路径内的一个前向 diamond，且相关区间没有 handler、回边、额外 reader、different/mixed/live phi。将 root 诊断和判定写入 evidence；若有任何不符，停止实现并修订 proposal/design/spec。
- [x] 1.3 先加最小证明测试，包括对象receiver保持既有完整链证明的真实双编译器控制：真实 fixture 正例核验 slot、两个完全相同 ValueId 输入、phi 退休事实、唯一 reader 与方法身份；负例覆盖不等输入、额外输入、live/mixed phi、额外 reader、handler、循环/回边。
- [x] 2.1 仅在 1.1–1.2 及真实正例证明测试通过后，实现局部候选 gate；1.3其余负例在生产接入后继续实测；普通 use-count 规则、全局 SSA API、循环和拼接路径保持不变。通过预算/取消检查且全部证明完成后才允许 owner commit。
- [x] 2.2 验证唯一正例通过既有 Folded 条件呈现路径，完整方法可编译运行，原始语义操作物理 source map 保持（纯goto缺口不在本片）；增加前向副作用次序对照，证明条件求值、producer 与 reader 顺序不变。
- [x] 2.3 对新的真实证明路径测试 budget stop 与 cancellation；分别证明停止发生在候选投影期间，并断言 source/declaration/map/region 不出现部分提交，保留先前已完成的物理facts/artifact，未提交正文按既有NotProduced合同不产出text/map/declaration。
- [x] 2.4 冻结并逐字比较 BI/boolean-loop 当前产品的恢复或拒绝文本、身份、来源和循环拓扑；验证新 gate 对该回边候选在投影前拒绝（已由其他既有证明恢复的整体方法不降级）。
- [x] 3.1 使用完整 admission class 与原 Runner 运行现有冻结 matrix：双 JDK、debug/none、default/all 的完整 16 个 Jarde cells；验证原 baseline 45 行及 admission 33 行 oracle，并按 manifest 记录 compiler/JADX 每项真实成功或失败。不得修改 fixture/helper 来规避方法或改写预期。
- [x] 3.2 完成 focused/full-workspace 必需验证及 corpus/source-map 回归，记录 receipt、source pins 和 frozen result；保留独立 root acceptance。
- [x] 3.3 root 独立审查证明输入、拒绝范围、行为次序、原子停止与全量矩阵。只有全部条件满足才可标记完成；此前保持 pending。

2026-10-14 root：1.1/1.2按 design 实测裁定完成；后续任务仍开放，完整矩阵失败不接受。

2026-10-14 root：按focused-v11 447/0/1、真实physical mixed/live/extra-reader/third-input/backedge控制、最后charge的ledger取消、完整16方法时序probe四cells/12行、BI当前真实before-after回放关闭1.3/2.1–2.4。BI历史误基线见design纠正；matrix-v2主审计accepted，3.1归档复开及3.2/3.3交付仍待关闭。

2026-10-14：3.1按 sibling ordinary-Try change 的 final-matrix-and-forward-order-root-v2 归档关闭，root独立主审计accepted/errors空、665成员逐一复开。当前8/10；fresh workspace与最终root交付仍开放。

2026-10-14 root workspace补充：fresh workspace-forward-v5在第70批捕获ICN.armCall既有whole-refusal回归（exit101），原断言保留，finally exact cargo clean exit0/target absent。旧focused-v11及matrix-v2仅证明其冻结源码，不能代替修复后的最终验收。carry调整为完整terminal/join/dependency证明成功后提交，正在focused-v12重验；双seed及自身CI仍开放。

2026-10-14 root：focused-v13实际451/0/2及ICN两编译器/default-all完整正文与v6逐字相同。1.2/1.3更新为物理Int边界及真实Ref旧前提匹配控制；新冻结CLI需要重新闭合BI及matrix，故2.4/3.1重新开放。原失败日志与旧成功冻结证据均保留。

2026-10-14 root 修复版冻结验证：focused-v13 451/0/2、CLI SHA0a30cf2cef1974006105e141789d5a9a8116497fae12d20479b2644d3a0ad2d7；matrix-v3主审计accepted/errors0，16完整Jarde cells及45/33行oracle。forward-order-v3独立root审计v2 accepted/errors0、64方法map/四cells12行；javac23仅保留原有source/target8的四条options警告，运行stderr空（首次审计误要求编译stderr空的rejected raw也保留）。BI v6/v13两编译器及历史class/default-all共12次render正文/quality/map/身份逐字一致；v3审计误用schema键失败保留，v4按真实outcome.report复核accepted。fresh workspace-forward-v6运行中，不先关闭3.2/最终交付。

2026-10-14 root最终本地验收：focused-v14 629/0/2，CLI逐字同v13，matrix-v3/forward-order-v3/BI/ICN实测接受。fresh workspace-v8 7124/0/196、730 target suites、195命令/390raw；额外6ignored命令9tests、2examples、tree gates通过。final-local-delivery-root-v3归档1642 payload/9 compiled source逐一复开绑定。关闭2.4/3.1/3.2；3.3最终产品自身CI与clean-main交付仍开放。

2026-10-14 root实际源码恢复反例：`carry-dependency-contrast-v1` 的14命令/28raw在两个编译器均观察到 `consume(make().value() + (test(flag) ? arm() : 3))` 原 stdout 为 `false|8|MVT`、`true|7|MVTA`，v14完整恢复类编译/运行exit0却为 `false|0|`、`true|0|`。f014及上述绿验证只覆盖其当时输入，当前交付未接受；重开2.2/3.1/3.2。修复先计划已证明的整数carry，使其receiver依赖在保存initializer处结束；实际保存失败撤销carry并恢复原多消费者整方法拒绝。新完整双编译器fixture保留工厂反例并加this/parameter控制，原baseline/admission/时序probe不改；等待新冻结验证。

2026-10-14追加本地验证：focused-v18独立accepted 637/0/3、4886来源，CLI与v17逐字同（cd5e52dd…/59590432B）。新完整10方法receiver正例四cells/40maps/6行oracle及2方法真实赋值负例四cells/全BCI/ExplanationOnly/compile1-no-runtime接受，原matrix-v4/forward-order-v4/BI/ICN亦实测接受。多维数组下游静态风险经完整7方法/28maps/两行运行证伪，已有NewArray恢复而非Other。workspace-v9第65批在旧P5计费精确断言停止；原ignored recorder重新量六case、原187方法direct/shared已逐text/outcome比较，只有many-method与两arm各+6 IrItems，其他维度不动。按新增预算化扫描更新三个精确计数和原因，不删比较或改shape；原失败/重测raw保留。full-workspace及最终交付仍未接受。

2026-10-15：按 sibling ordinary-Try 的 [final-local-delivery-root-v4](../recover-ordinary-try-list-generic-bodies/results/final-local-delivery-root-v4/README.md) 关闭2.2/3.1/3.2。root实际复开3536 payload及十个编译来源，focused-v19 642/0/4，fresh workspace-v10 7130/0/196，额外九ignored测试/两examples/依赖边界通过；新receiver正反完整类及原矩阵均绑定逐字相同最终CLI。当前9/10，修复产品自身CI和最终clean-main交付3.3仍开放。

2026-10-15：修复产品644882d6db69f7d5b0bc1165c035a12b4f7cd8aa已提交推送main，[自身CI与clean-primary证据](../recover-ordinary-try-list-generic-bodies/results/own-ci-644882-and-clean-main-root-v1/README.md)根实际742项审查接受；4jobs/52steps成功，两seed各365suites、3565通过/0失败/98忽略，55具名门每seed一次通过。primary HEAD/remote main一致、Git clean、无target，4886编译来源逐字同最终冻结版本。关闭既定局部交付，文档提交后再核最终真实HEAD；不增加CF11/CF18/71/612总体计数。活动core候选、新问题与后续规格归档另行处理。
