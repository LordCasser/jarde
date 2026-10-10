# HANDOFF — jarde 主线接续入口

用户明确继续在当前聊天推进。以本地 JADX **71验收单元/612测试文件**为清单，逐片追平后再探索额外场景；root负责架构、OpenSpec、真实源码/JADX/Jarde完整对照与对抗验收，确定性私有patch/script派给Luna。窄片成功不增加整单元完成数。

## 当前明确工作

`preserve-proved-if-arm-join-origins` **6/7**：产品已root应用并通过本地/完整类独立验收，精确自身CI已接受，最终clean交付待关闭。生产只扩build.rs普通If来源：末尾直接Straight、真实goto/goto_w、完整唯一Normal边到准确If.join，复用OriginSet和既有ownership；无新IR/Frame/pass/依赖。全部执行证据见 [If verification-root](openspec/changes/preserve-proved-if-arm-join-origins/verification-root.md)。

- 本地12命令562/0/1；52输入pins恒同，target峰值701667300bytes。永久测试核完整If跨度（含缩进/换行）、physical owner/descriptor/全部23BCI、旧condition/while来源、错误exit target与非transfer、真实Exception反例、Stop at20无部分artifact。
- CF07新29命令/10完整类腿，双JDK8/23/default-all稳定；唯一新增counted@20 derived完整If，counted全部物理BCI已覆盖，正文和所有旧来源恒同。唯一剩余缺口lastIndexOf@25，整CF07计数不变。
- 旧For/Postfix新57命令/20完整类腿、28方法profile全物理BCI；8生成类正文与全部map精确同已接受For版本。真实测试预期错误与verifier作用域错误、原稿和失败raw保留。
- CLI `/private/tmp/jarde-proved-if-join-cli-v1`，0555，SHA7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6；metadata SHA6f785a03e50565bc5d90bd6a6ae85f1bb789e31647811421e5f21d41ee7030ca。冻结source base d9855c2764860c7c2e20e40e700abeaa59f00a55与uncommitted标志不回写。

## 下一步操作

本片产品 `1f386686c6a31813254a85fed48d2af0af84a61c` 的自身CI [38068627541](https://github.com/LordCasser/jarde/actions/runs/38068627541) 已root独立v4接受：4jobs/52steps、双seed各3389/0/97及354记录，完整12gateway名、新If三名/内部异常名/旧integer11名、52live+Git pins恒同。见 results/ci-product-v2/acceptance-proved-if-arm-join-ci-root-v4.json。日志前缀与缩进解析的v2/v3真实失败保留，v4只改传输格式归一化；最终clean交付完成后才应用下一片。
关闭本片3.2/3.3后，下一明确项 **preserve-proved-return-arm-loop-latch-origins** 规划4/4、strict340/0、实现1/7：root同次真实IR诊断v2已实际1/0/0，证明lastIndex Loop5/末尾直接If16 joinNone/thenStraight19 return21/elseStraight22 SSA22,25/唯一natural latch22/全canonical仅Normal5/唯一owner；52pins已恢复，cargo clean144.8MiB。v1私有诊断E0277原始失败与root **owner修正都保留。本片限制单block直接双arm，其中一return/另一唯一exact latch，复用既有证书和gateway_origins，无新实体。Luna已private交付最小patch与新CF07完整collector/verifier，root已读审，另在准备新的validation runner，root前片CI接受后才应用。另有真实独立for呈现缺口：preheader初值门仅Push(Int)+Store，end-1的load/sub不满足，不能把来源缺口关闭算整CF07追平。

## 已闭合前片

For来源片 `preserve-proved-for-latch-origins` **6/6**、产品d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c，准确自身CI38062706229已root独立v2接受4jobs/52steps、双seed3385/0/97和干净主线。前片`recover-prefixed-one-arm-loops` **7/7**，自身ca43/CI38048944005和For来源依赖均已关闭。历史CLI/raw/源码与旧失败不覆盖，详情见各自verification-root和71单元账本。

## 资源与分支

用户已明确批准 **5GiB最低余量/本仓target1GiB上限**、一秒进程组中止、完成cargo clean。root独占Git/Cargo/rustfmt/JDK/JADX/CLI；Luna只做private准备或只读审计，不创建占用分支。

本片已实际cargo clean3022files/669.2MiB，CLI与所有源码/class/raw保留。前次d985主线真实审计15 worktrees全部clean，main/origin相同，仅main，14辅助worktrees均detached main祖先，无target/待合入/分支占用；保护副本保留。当前提交后须再次实际核这些条件；不能拿前次审计代替最终交付。

## 独立债务

本片真实反例显示外层named Try→Loop→If：Frame::protected设置own_try，Frame::loop_body清为None，普通named catch没有现有finally/guard证书恢复它，故If then以ExceptionEdge拒绝。整方法fallback还只有blockleader来源、physical21缺span。这两项单列在If/results/exception-case-architecture-root-v1.md，不扩大本片。还有arithmetic iadd@5来源、flat Signature generic arity、receiver-tail预算、CF16浅层组合、handler ctor goto19和else-if缩进，均按原独立债务推进。

长期账本：[71单元summary](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。先完成当前精确自身CI与clean交付，再推进下一项；持续目标仍active，不暂停也不宣称已完成。

并行方向只读清单在下一片results/next-ledger-triage-luna-v1.md：CF-12整数switch、DT-02静态成员类、EM-23字段更新按本地JADX活动断言/已有完整基线扩验；CF-12已实跑五份原上游Java fixture/六方法，见 [完整基线](openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/README.md)：JADX六腿全类编译运行同原，Jarde六同/两异/四编译失败。类型恢复 OpenSpec recover-proved-local-source-types 为1/8（root v6已独立核验冻结基线观察），Luna私有实现准备中；条件fallthrough草稿已交付待真实IR，常量名门另记债务。尚未接受整CF12。
