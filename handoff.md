# HANDOFF — jarde 主线接续入口

用户明确继续在当前聊天推进。以本地JADX **71验收单元/612测试文件**为清单，逐片追平后再探索；root负责架构、OpenSpec、真实源码/JADX/Jarde整类对照及对抗验收，确定性私有稿交Luna。窄片成功不增加整单元完成数。

## 刚关闭：调用结果 pop 来源

`preserve-proved-discarded-call-origins` **6/6**。已应用build.rs两条普通调用语句共用的小型来源消费helper，复用既有discards准确一对一证据，预算/poll后将pop挂到完整Stmt derived；无新IR/Frame/计划/pass，未扩大qualifier/pop2准入。根实际定向5/0/0；完整本地12cmd639/0/0、target峰值701811680bytes；新CLI完整对照26cmd/8runtime腿/28方法profile/18class独立接受，仅pop4/13/15新增来源，正文和旧maps恒同；额外pop2完整三方法恒同回退。详见 [verification-root](openspec/changes/preserve-proved-discarded-call-origins/verification-root.md)。

CLI `/private/tmp/jarde-proved-discarded-call-cli-v1` 0555，SHA251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde；metadata/build在该change/results，真实sourcebase7a1ca930fc6178613df5df86145fff846c79791c，历史uncommitted标志不回写。产品c5941c890b352b281fc2bcf4102e45243691bddb已实际提交推送，自身CI38079005388已独立接受4jobs/52steps、双seed各3398/0/97及355summary；五新增名/lib337/gateway15与72分类条目（71唯一路径）Git/live pins均核。接受记录SHA16abfae27106be6a81ae8d641b9f5427df797456bbfe54b806b98ab50669a8e2，两次verifier格式/计数失败保留。cargo clean实际3027files/669.3MiB，target不存在。全部验收/独立基线已随35cdf3cb8531e70744e3c893e2c2e1128ad2a0e0提交推送；随后实际audit接受main=origin、15worktrees全clean/14辅助detached祖先、本地与远端仅main、无target，free59346087936bytes。准确pop CLI再次0555/SHA核对。证据在results/clean-delivery-checkpoint-root-v2，SHA14a484971f83886fcc9df9a2c62fd38035f75fe08a674557436823191c5e8594。strict343/343再次通过；关闭文档提交后继续private final audit，不借文档CI。

## 当前推进：局部类型恢复

`recover-proved-local-source-types` **6/8**。在关闭后的5c2c06f main重新应用类型决策扩展，无新IR/Frame/pass。真实class永久9/0/0，char pop82/92/105完整物理来源闭合；debug/no-debug 22个structured边界方法profile全来源与default/all正文/map相同。真实slot-conflict精确fallback不放宽。证明内Stop@29/1无content/binding/text/map，公开预取消保持原子性；临时trace已移除。完整本地14cmd645/0/2、target峰值701852711bytes；p3_patterns旧预算61→63两物理写成本修正，失败保留。新CLI `/private/tmp/jarde-proved-local-source-types-cli-v1`0555，SHAe6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403；meta/results和validation-build-root-v2已冻结。

完整源码实际99cmd、两锚8条双JDK/profile runtime与原/JADX一致，8其他控制profile保持独立post-pop基线，全部class/check/Inner/SDK保持；9cmd准确append(I)重载完整类对照通过。组合边界类仍slot-conflict缺return，不计运行通过；JADX全null println重载歧义已在仅改Runner package的实际双JDK编译中独立确认。收集器schema/globals及root并行清理错误、verifier路径/日期表示失败均保留，最终接受在complete-source-root-v4和int-overload-root-v1。cargo clean3042files/669.3MiB，target不存在。产品8b8997781自身CI38084415019在旧p3_meeting int局部断言失败，第二seed跳过；现断言改核已证char局部与int返回，追加focused6/0/0、完整Meet双JDK×原/JADX/default/all八腿逐值65536通过，生产无修订。后续还定位RequiredConversions旧int局部断言与NullThenBuilder旧Object拒绝分类；两项准确改核char/StringBuilder，保留七其他拒绝边界和历史baseline。最终focused v7三组22/0/0；完整Required13成员/Null3成员双JDK×原/JADX/default/all运行实际通过，独立复核待完成（Null原源码不可得，oracle是canonical class）。生产冻结CLI/pins未改；全workspace小批补完、新修复提交自身CI及最终clean待完成，整CF12未接受。[详细验收](openspec/changes/recover-proved-local-source-types/verification-root.md)。

`recover-proved-conditional-switch-fallthrough` **1/7**：实际11canonical块/17Normal edge诊断已独立接受，group fall_through不能表达case局部分支break；需要有限Region::SwitchBreak证据叶及最近switch作用域准确消费现有AST Break，不重扫CFG修AST。root已全文读private实现v2及真实positive永久测试草稿v4，但未应用或工具链/API接受；scope/预算/完整canonical边及所有权反例待实测。五个边界完整基线已实跑21命令：JADX双JDK各五行行为差异，Jarde整类四个missing-return而未进入runtime。公开真实IR十profiles已独立接受：partial为7块9Normal无环，innerLoop真实57→38回边，innerSwitch嵌套多路，terminal准确返回/抛出，caught含三条Exception；仍非新产品验收。完整证书私有函数抽取尚未编译/应用。常量名门与computed-init for等保持独立债务，不扩大当前片。

## 已闭合的前片

- `preserve-proved-return-arm-loop-latch-origins` **7/7**，产品0ae30a7c8d217522f2e5f5b954aa86c9b59356dd，自身CI38073837512独立接受4jobs/52steps/双seed3393/0/97；本地566/0/1、CF07完整29cmd/10腿和旧For57cmd/20腿/28profiles。sourcebase41fe336，CLI `/private/tmp/jarde-return-arm-latch-cli-v1` SHA9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f。关闭记录e846实际推送并核15worktrees clean/仅main/无target才应用typed，原始失败都保留。
- `preserve-proved-if-arm-join-origins` **7/7**，产品1f386686c6a31813254a85fed48d2af0af84a61c，自身CI38068627541独立接受双seed3389/0/97；根关闭41fe336主线clean，准确If.join goto来源有限复用。
- `preserve-proved-for-latch-origins` **6/6**、产品d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c/CI38062706229；`recover-prefixed-one-arm-loops` **7/7**，两片自身产品CI与clean都已接受。

## 资源、分支与账本

用户已明确批准 **5GiB最低机器余量/本仓target1GiB上限**，一秒进程组中止及完成cargo clean。root独占Git/Cargo/rustfmt/JDK/JADX/CLI；Luna private准备或只读审计，不占用分支。最新35cdf3cb8提交推送后实际审计15worktrees clean，仅main，14辅助detached main祖先，main=origin、本地及真实远端heads只有main、全部无target/fuzz target；证据在pop片results/clean-delivery-checkpoint-root-v2。关闭文档的最终HEAD仍用private实际audit确认。

长期入口：[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。CF12五份原上游Java/六方法完整基线已实际确认JADX六腿同原，旧Jarde六同/两异/四编译失败；整CF12尚未接受。另有named Try→Loop丢own_try、fallback physical21缺span、iadd@5来源、flat Signature generic arity、receiver-tail预算、CF16浅层组合、handler ctor goto19/else-if缩进等独立债务。持续目标active，未暂停或宣称完成。
