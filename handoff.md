# HANDOFF — jarde 主线接续入口

用户明确继续在当前聊天推进。以本地JADX **71验收单元/612测试文件**为清单，逐片追平后再探索；root负责架构、OpenSpec、真实源码/JADX/Jarde整类对照及对抗验收，确定性私有稿交Luna。窄片成功不增加整单元完成数。

## 当前工作：调用结果 pop 来源

`preserve-proved-discarded-call-origins` **4/6**。已应用build.rs两条普通调用语句共用的小型来源消费helper，复用既有discards准确一对一证据，预算/poll后将pop挂到完整Stmt derived；无新IR/Frame/计划/pass，未扩大qualifier/pop2准入。根实际定向5/0/0；完整本地12cmd639/0/0、target峰值701811680bytes；新CLI完整对照26cmd/8runtime腿/28方法profile/18class独立接受，仅pop4/13/15新增来源，正文和旧maps恒同；额外pop2完整三方法恒同回退。详见 [verification-root](openspec/changes/preserve-proved-discarded-call-origins/verification-root.md)。

CLI `/private/tmp/jarde-proved-discarded-call-cli-v1` 0555，SHA251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde；metadata/build在该change/results，真实sourcebase7a1ca930fc6178613df5df86145fff846c79791c，历史uncommitted标志不回写。cargo clean实际3027files/669.3MiB，target不存在；当前有授权产品/证据文档修改，不能冒称工作区clean。strict343/343已实际通过，待产品提交推送、准确自身CI及15worktrees clean验收后关闭本片，再应用typed。

## 后续已有方向

`recover-proved-local-source-types` **2/8**：第一次两轮真实2/1，null/String全来源与atomic Stop通过；char正文正确但仍缺append后pop82/92/105。已完整保存实际patch/test/失败raw并恢复build.rs及自动测试目录，生产尚未再次应用。pop来源片关闭后先冻结独立新的CF12 pretyped基线，准确接受pop造成的control maps delta，再重新应用typed actual product，保留全部physical BCI断言。原CF12 rootv6 raw不覆盖，不让typed candidate充当自己预期。

类型v3实现/永久测试及replay v2私有稿保存待完整审查/实际执行；原边界12cmd/4腿只能算original验收。root已全文读预算observer草稿，因4个test-only实体/thread-local+生产cfg分支过重而不应用；使用临时实际trace保存prefix后移除、已有trusted真实IR永久测试准确Stop/public cancel即可。

`recover-proved-conditional-switch-fallthrough` **1/7**：实际11canonical块/17Normal edge诊断已独立接受，group fall_through不能表达case局部分支break；需要有限Region::SwitchBreak证据叶及最近switch作用域准确消费现有AST Break，不重扫CFG修AST。private v2尚未root全文/API/工具链验收，对抗审查风险待实际scope/预算/所有权反例验证。暂未应用生产。常量名门与computed-init for等保持独立债务，不扩大当前片。

## 已闭合的前片

- `preserve-proved-return-arm-loop-latch-origins` **7/7**，产品0ae30a7c8d217522f2e5f5b954aa86c9b59356dd，自身CI38073837512独立接受4jobs/52steps/双seed3393/0/97；本地566/0/1、CF07完整29cmd/10腿和旧For57cmd/20腿/28profiles。sourcebase41fe336，CLI `/private/tmp/jarde-return-arm-latch-cli-v1` SHA9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f。关闭记录e846实际推送并核15worktrees clean/仅main/无target才应用typed，原始失败都保留。
- `preserve-proved-if-arm-join-origins` **7/7**，产品1f386686c6a31813254a85fed48d2af0af84a61c，自身CI38068627541独立接受双seed3389/0/97；根关闭41fe336主线clean，准确If.join goto来源有限复用。
- `preserve-proved-for-latch-origins` **6/6**、产品d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c/CI38062706229；`recover-prefixed-one-arm-loops` **7/7**，两片自身产品CI与clean都已接受。

## 资源、分支与账本

用户已明确批准 **5GiB最低机器余量/本仓target1GiB上限**，一秒进程组中止及完成cargo clean。root独占Git/Cargo/rustfmt/JDK/JADX/CLI；Luna private准备或只读审计，不占用分支。最近产品应用前7a1推送后真实审计15worktrees clean，仅main，14辅助detached main祖先，main=origin、无target/fuzz target；证据在当前pop片results/prerequisite-clean-root-v1。产品关闭须重新真实审计。

长期入口：[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。CF12五份原上游Java/六方法完整基线已实际确认JADX六腿同原，旧Jarde六同/两异/四编译失败；整CF12尚未接受。另有named Try→Loop丢own_try、fallback physical21缺span、iadd@5来源、flat Signature generic arity、receiver-tail预算、CF16浅层组合、handler ctor goto19/else-if缩进等独立债务。持续目标active，未暂停或宣称完成。
