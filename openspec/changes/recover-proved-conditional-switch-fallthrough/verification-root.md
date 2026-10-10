# Root verification — recover-proved-conditional-switch-fallthrough

本片当前 **5/7**；条件switch生产候选已在前类型片干净交付后应用，真实 helper/caller 与预算测试已有实际观察。完整 workspace、冻结 CLI 的完整源码对照及自身产品 CI 继续验收；以下历史记录按其当时范围保留。

## 两个完整物理负例的实际观察

[physical-boundaries-root-v1](results/physical-boundaries-root-v1/README.md) 保留原class完整副本及两个offset-only变体，两个JDK各original/A/B共六runtime腿，每腿36行，16条实际JDK命令全部通过。root审阅私稿后纠正四字节/五member的错误期待；每变体实际只改两个低operand字节，完整构造器与五业务方法保留，无匹配变体的Java源码。

[独立public IR接受](results/physical-boundaries-root-v1/public-ir-acceptance-root-v2.json) 只接受观察：四partialBreak profiles均6块/8条Normal完整multiset、empty clone paths、完整incoming与SSA membership；A case0 exits57/67，B只有67且57在物理入口次序中间。物理50/51/53/56不在canonical区间，canonical.unreachable原报告为[]，两事实分别保留。实际BLAKE3/owner/snapshot/name/descriptor、decode37/47、raw和86source pins已核。临时observer移除，守卫峰值180560145bytes，cargo clean348files/172.2MiB，target不存在。

helper/caller gate与新产品尚未接受；private tests v1/v2均未编译，不能当任务完成。A须实际到达多出口拒绝，B须分开证明helper的唯一map与真实caller的adjacency拒绝；不复制生产门逻辑作为测试。先前完整五边界原/JADX/Jarde观察及十个原class public-IR profiles分别保留在boundary-baseline-root-v1和boundary-public-ir-root-v1，新负例不改变那些历史结论。

## 应用后的真实恢复与边界

前类型片8/8关闭，2d70da515896c25ce022b8c28f4935ff2e105026最终实际main/origin与15worktrees全clean，之后才应用三源private补丁。首轮两处caller变量名错误、随后默认149块store@169自然落join171被过严拒绝，真实失败与临时refusal trace均保留。最小修正只在同arm存在case fallthrough时要求各join出口可呈现break，普通无fallthrough arm复用组尾break；不改变全边/ownership/adjacent边界。root测原真实helper map32→117、A双出口None、B helper36→67而公开caller准确overlap@0拒绝。正例全文/两goto@89/114准确break、case117仅一次、join171外置及全physical来源均实际通过。

真实public caller三phase入口分别137/154/193 steps、BCI0；逐项fresh预算实测AnalysisSteps written0/at0，无content/binding/text/map。永久测试三限额再次通过；预取消仅证明入口原子性，不冒称运行中取消。新增Region计费使旧类型proof固定限额移位，root实际write@29=365、null proof@1=130，更新限额后9项类型测试仍在原证明内部Stop，原321/94失败保留。

四边界方法同次公开reader/recovery：innerLoop真实回边但正文ExplanationOnly/uncovered38等，innerSwitch arms-do-not-meet@0，caught保留三Exception边且uncovered47，不生成错误break；这三项早于Builder消费阶段拒绝，不能冒称其最近scope guard已触发。terminal原class完整return@47/88和throw@66准确来源、全physical覆盖/配置正文和显式map一致，永久测试实际通过；这仍不是整个组合class可重编运行。

证据在results/candidate-validation-root-v1，库全340/0/0已通过，七个旧switch/string/loop-switch目标及新proof/boundary永久测试通过。任务4/7；Builder控制叶作用域扰动、其他证明拒绝分支、全workspace/整类CLI/自身CI及clean交付继续，不接受整CF12或长期目标。

## 补齐证明和消费边界

`results/candidate-validation-root-v2` 已独立冻结 110 项完成记录。真实 innerLoop cycle 到达有限图灰节点拒绝，unknown-terminal 使用同次解码事实别名扰动到达未知终止事实拒绝；两项在 v3 完整库 342/1/0 中通过，唯一失败是新增 Builder scope 包装。root 修正 Endless Loop 的实际 header body ownership 后，v6 Builder 指定测试实跑 1/0/0，正例、无 active owner、错误 branch/join、跨内层 loop/switch 均到达预期消费边界。后两类是完整真实 Region 树中的 consumer metamorphism，不冒称物理 class 的该早期拒绝已到 Builder。

已有 P2 legacy class 完整冻结并经实际 Reader 输出 pin BCI7 clone `[0]`/`[3]`、全部 Call/Return 行；路径不匹配测试 v6 实跑 1/0/0。case 参数是 caller-side metamorphism，不冒称合法 decoded switch。当前库总计 344 尚待完整跑过；两次 Clippy 失败、v1-v5 Builder 失败保留，未借用前类型片结果。完整 workspace 正运行，源输入冻结期间不修改 Rust。

完整 workspace v2 在21成功批后因守卫扫描 Cargo 临时文件的 TOCTOU 失败，root 修正为一次 stat，历史 v9 原字节不改。v3 从头运行62批通过，第63批P5计费仍5/0/1通过，corpus fingerprint 因六新增class未登记退出101；失败与无源变更都保留。已有生成器实际1/0/0更新清单，2062旧条目与其他分类字段恒同，仅新增六项至2068，独立blake3/SHA核实。v4从头重跑，FS读取的manifest额外作为明确input pin；证据分别在workspace-attempts-root-v1/v2，不把部分批数计作整workspace成功。

2026-10-11 root实际v4从头完成90批，最新jarde-java全库344/0/0及全部条件/作用域/预算回归通过，满足2.2；全workspace在reader库177/1/0旧fixture人口断言失配停下，不能称全量通过。独立六fixture javap实测delta6/31/5/167/2恰好解释新1093/4861/468/2882/10，只改准确普查tuple与说明，未改reader逻辑或P5成本。v4/独立javap/原README误写恢复字节记录在results/workspace-attempts-root-v3（209 files、7860583bytes）；v5重新从头95批且单独pin运行时corpus manifest，不借旧执行。validation新source pin包含reader/src/classfile.rs，17product/21test/162literal fixtures。CLI/CI仍待验收。


2026-10-11 root本轮实际接受 **5/7**。fresh v5全workspace95批/359个实际Cargo目标，3422/0/97，2210源输入Git/live待产品CI绑定，target峰值914810494bytes；完整原始证据和独立观察在results/full-workspace-root-v1。随后fmt、CI同范围Clippy和专项13命令563/0/0通过，冻结CLI SHA39d5699c1665b16e0c4a46934f0e773aeedf392f5bc60869a7b2762680dd10f1，17product/21test/162fixture pins。守卫5GiB free/target1GiB一直满足，峰值1067909023bytes；cargo clean5445files/1018.4MiB，target不存在。

完整五上游fixture/六方法actual95命令，在双JDK条件锚/两个类型锚及其余JDK23控制分别编译运行原/JADX/default/all；新conditional verifier实际接受4conditional matches、14control profiles、11default/all maps，各完整生成class集合与原一致，目标(IZZ)Ljava/lang/String;按owner/name/descriptor/ordinal和javap全physicalBCI绑定。结果results/complete-source-root-v1/acceptance.json；独立invocation及清理记录在results/delivery-precommit-root-v1。首次verifier误把累计budget当独立方法计费的失败原稿/raw在results/replay-verifier-attempts-root-v1；root查prepare_physical_class_source同一Budget，固定目标delta AnalysisSteps311/IrEdges17/IrItems default212-all211/OutputBytes469，要求后续check和类总snapshot恰为同一delta，其他字段不变，所有无关正文/map/质量/诊断仍严格一致；没有改生产或CLI。OpenSpec strict实际343/0/0通过。自身产品commit/CI和最终clean交付尚待，不接受整CF12。
