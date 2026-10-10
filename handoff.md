# HANDOFF — jarde 当前主线接续入口

用户明确要求继续在当前聊天推进。按本地 JADX **71验收单元 / 612测试文件**逐片追平，再探索额外场景；root负责架构、OpenSpec、真实三方完整源码对照与对抗验收，确定性private patch/script使用Luna。窄片成功不改分母，不增加整单元完成数。

## 当前工作与准确状态

- 产品 `d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c` 已提交推送 main；For来源片 `preserve-proved-for-latch-origins` **6/6**，本地与完整类来源已接受，精确自身CI **38062706229** 已由root独立v2接受4jobs/52steps、双seed各3385/0/97，最终干净交付已实际接受。
- 前片 `recover-prefixed-one-arm-loops` **7/7**。ca43自身CI38048944005早已独立接受，其源码能完整运行但缺for@20的历史证据保留；新的For来源依赖已闭合3.1，3.3已按8784e2f07真实clean主线审计关闭。
- 下一片 `preserve-proved-if-arm-join-origins` **规划4/4、strict有效、任务2/7**；只接受基线，未实施生产/新CLI；内部诊断已真实通过并还原50pins。优先按下一节执行，不重复已验收的循环来源片。

## 当前产品实际验收

生产只修改region.rs既有implicit_tail_latch_origin和join gate，复用ForHeader/gateway_origins/OriginSet。精确末尾Straight、唯一natural latch、update block/BCI/slot、真实goto/goto_w与唯一完整canonical Normal边同一证明；无新IR/Frame/pass/依赖。

本地guarded12命令全部成功，558 passed / 0 failed / 1 ignored；50源码/测试/canonical pins前后恒同，target峰值701574616 bytes。准确build-v1基础commit87090b与未提交产品标志是历史冻结身份，后续不能回写。新CLI `/private/tmp/jarde-proved-for-latch-cli-v1`，0555，SHA256 `d2d9773d94011a680e18d968ea1b3684036791ceb7dfa769455adf9247f0d33a`；metadata SHA256 `aeb3a081e3f05e27f64f5afea48d4fc4a2ccaa410209910eb40065436f54e54c`。

新CLI真实57命令、原4/JADX8/Jarde8共20完整类重编运行腿；双JDK8/23、Jarde default/all八份完整生成类raw同原，28方法profile全部物理BCI完整。root独立接受 `candidate-whole-classes-acceptance-root-v1.json`，full_bci_acceptance=true。新旧正文及所有旧来源恒同，仅三方法四profile的12条derived完整for@20新增。

新CF07控制真实29命令/10完整腿、119 inventory members；root独立v2接受所有成员、raw javap重新解析、准确goto、源/Runner及原raw，default/all恒同。`cf07-candidate-acceptance-root-v2.json`明确scoped observations/full_physical_bci_coverage=false；counted@20与lastIndexOf@25仍为独立缺口。原full verifier失败、Runner适配错误导致的v1真实验收失败和其raw均保留。

所有准确来源/脚本/实际调用见 [For verification-root](openspec/changes/preserve-proved-for-latch-origins/verification-root.md)。本产品自己的CI已独立接受d714a6bcc/38062706229；原API、原始日志及v1交错stdout/stderr解析失败保留，v2准确核fingerprint六个名字和各摘要。全量OpenSpec strict339/0已实际通过。

## 下一项：CF07 非空 If arm 汇合来源

上述CI与干净main已完成，If-arm 1.2内部诊断也已关闭；当前推进 [If-arm OpenSpec](openspec/changes/preserve-proved-if-arm-join-origins/verification-root.md) 的2.1私有最小实现。

counted(II)I准确缺的是物理goto20→27；它属于内层If，不能放进Loop.gateway_origins归给外while。现有If.join与Builder OriginSet够用，Builder当前只为empty arm隐藏goto补来源。根已核本地JADX IfRegionMaker的dom frontier/path-cross join选择，但当前没有替换本仓join算法的必要。拟只证明arm最后直接Straight、真实terminal goto、全canonical outgoing唯一Normal到本If join与已有唯一ownership，派生到完整If语句，保留condition14/outerwhile30/全部正文旧来源。

公开RecoveryReport只含扁平RegionRecord，不能证明nested If树；region::recover是pub(crate)。使用临时内部测试诊断同一次真实IR/regions，不新增生产hook、不伪造表，运行后还原准确源码pins。canonical source block17与physical instruction20必须区分；root新诊断实际1/0/0接受join27/then Straight17(含iinc17,goto20)/唯一完整Normal17→27/then-else-method计数1/0/1；第一次稿类型编译失败与raw保留，v3临时patch/v2runner成功，source精确还原。证据见If/results/internal-diagnostic-acceptance-root-v1.json，不能借此代替新CLI验收。

lastIndexOf25嵌套else latch另片；conditional-value折叠、任意内部goto、ForHeader扩展不混入当前目标。先核下一片完整基线与结构证明，再派发最小实现，不能用公开扁平记录或历史probe代替新诊断。

## 资源与主线

用户已明确批准机器 **5GiB最低余量 / 本仓target1GiB上限**，一秒守卫中止进程组、完成cargo clean。历史20GiB记录不再是当前约束。root独占Git/Cargo/rustfmt/JDK/JADX/CLI，Luna只做private准备或只读审计。

已cargo clean本仓3022 files/669.1MiB，root/fuzz target不存在，冻结CLI及全部源/class/raw保留。仅main分支；14辅助worktree经实际预检均clean detached main祖先、无target/待合入/分支占用，保护副本保留。验收与下一规划文档已提交推送8784e2f07，实际delivery-clean-root-v1审计15 worktrees全部clean，main/origin相等、仅main、辅助全部detached，无待合入或分支占用；free68989394944 bytes。闭合记录16f912292已提交推送并再次实际accepted_clean_main，记录在If/results/prior-for-final-clean-root-v1；临时诊断后再清341files/144.8MiB，target仍无。

## 已交付与独立债务

乘法片1ee7c42e自身CI38033367666：4jobs/52steps、双seed3377/0/97，7/7。同类整数数组常量名375dee自身CI38026598963：双seed3374/0/97，8/8。实例共同数组prefix192b自身CI38022628853：双seed3368/0/97，8/8。非final静态阶段6fd自身CI38019682442：双seed3357/0/97，7/7。普通while来源404b自身CI38045578457：双seed3379/0/97，6/6。每片准确范围/冻结身份/验收链接仍见自身verification-root与账本，不重开已验收范围。

Arithmetic producer iadd@5来源、flat Signature generic arity、receiver-tail预算、CF16默认小栈/浅层loop组合、handler双ctor goto19来源和else-if缩进仍是独立债务。完整类必须原样重编、fresh empty CP/SP/classes、-Xverify:all，只允许Runner必要package适配，exit/stdout/stderr逐字同原；不删成员/手改正文/借原helper，不将quality或能运行代替来源证明。

长期账本见 [71单元summary](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。本次收束前完整handoff历史保留在 [handoff history](openspec/changes/preserve-proved-for-latch-origins/results/handoff-history-before-for-delivery-v1.md)，所有真实失败不覆盖。

当前Luna if_join_impl只私有准备最小build.rs+CF07正例/预算/真实指令反例；资源守卫和新CF07完整对照脚本也仅私有准备。root尚未应用产品。先全文读审v2、构造余下真实边界再实跑；新CLI要独立冻结且精确自身CI，不能重复使用For元数据。
