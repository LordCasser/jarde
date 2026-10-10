# HANDOFF — jarde 主线接续入口

用户明确继续在当前聊天推进。以本地 JADX **71验收单元/612测试文件**为清单，逐片追平后再探索额外场景；root负责架构、OpenSpec、真实源码/JADX/Jarde完整对照与对抗验收，确定性私有patch/script派给Luna。窄片成功不增加整单元完成数。

## 当前明确工作

`preserve-proved-return-arm-loop-latch-origins` **7/7**：root已应用有限直接return/latch If候选扩展，本地12命令566/0/1、新CLI CF07完整29命令/10类腿全部物理BCI及旧For/Postfix57命令/20类腿/28方法profile已独立接受。唯一四profile新增lastIndexOf@25完整while来源；正文/旧来源恒同。自身产品CI38073837512已独立接受（4jobs/52steps、双seed3393/0/97），6118e565 clean交付已实际核对，详见 [return-arm verification-root](openspec/changes/preserve-proved-return-arm-loop-latch-origins/verification-root.md)。新CLI `/private/tmp/jarde-return-arm-latch-cli-v1` SHA9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f，实际source base41fe336462448eae3547cafa2a141d52e85a08e0。全BCI来源覆盖不能算整CF07追平，computed-init for仍独立待办。

## 已闭合 If 来源片

`preserve-proved-if-arm-join-origins` **7/7**：产品已root应用并通过本地/完整类独立验收，精确自身CI与8cbc468 clean交付已接受。生产只扩build.rs普通If来源：末尾直接Straight、真实goto/goto_w、完整唯一Normal边到准确If.join，复用OriginSet和既有ownership；无新IR/Frame/pass/依赖。全部执行证据见 [If verification-root](openspec/changes/preserve-proved-if-arm-join-origins/verification-root.md)。

- 本地12命令562/0/1；52输入pins恒同，target峰值701667300bytes。永久测试核完整If跨度（含缩进/换行）、physical owner/descriptor/全部23BCI、旧condition/while来源、错误exit target与非transfer、真实Exception反例、Stop at20无部分artifact。
- CF07新29命令/10完整类腿，双JDK8/23/default-all稳定；唯一新增counted@20 derived完整If，counted全部物理BCI已覆盖，正文和所有旧来源恒同。当时剩余缺口lastIndexOf@25已由当前片补全，整CF07计数不变。
- 旧For/Postfix新57命令/20完整类腿、28方法profile全物理BCI；8生成类正文与全部map精确同已接受For版本。真实测试预期错误与verifier作用域错误、原稿和失败raw保留。
- CLI `/private/tmp/jarde-proved-if-join-cli-v1`，0555，SHA7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6；metadata SHA6f785a03e50565bc5d90bd6a6ae85f1bb789e31647811421e5f21d41ee7030ca。冻结source base d9855c2764860c7c2e20e40e700abeaa59f00a55与uncommitted标志不回写。

## 下一步操作

本片产品 `1f386686c6a31813254a85fed48d2af0af84a61c` 的自身CI [38068627541](https://github.com/LordCasser/jarde/actions/runs/38068627541) 已root独立v4接受：4jobs/52steps、双seed各3389/0/97及354记录，完整12gateway名、新If三名/内部异常名/旧integer11名、52live+Git pins恒同。见 results/ci-product-v2/acceptance-proved-if-arm-join-ci-root-v4.json。日志前缀与缩进解析的v2/v3真实失败保留，v4只改传输格式归一化；8cbc468 clean checkpoint已实际接受15worktrees全clean、仅main/无target、free65,230,618,624；关闭记录推送后再次实核，才应用下一片。
If关闭记录41fe336已实际推送并再次审计15worktrees全clean/仅main/14辅助detached main祖先/无target；证据在当前片results/if-prerequisite-clean-final-root-v1。return-arm片已经实施，真实budget永久测试与错误target/nonterminal/nonreturn和异常/嵌套/多回边边界均已核，失败raw保留。当前片自身精确CI与6118e565 clean交付已接受；等待远端CI时root已经临时实跑CF12两项真实IR诊断并立即还原全部52pins、cargo clean341files/145.0MiB，未应用下一片生产。证据见 openspec/evidence/java-syntax-2026-10-11/cf12-real-ir-diagnostics-root-v1/README.md。


## 已闭合前片

For来源片 `preserve-proved-for-latch-origins` **6/6**、产品d714a6bcc86c7dfd757f7ff70f1f86a5b46a230c，准确自身CI38062706229已root独立v2接受4jobs/52steps、双seed3385/0/97和干净主线。前片`recover-prefixed-one-arm-loops` **7/7**，自身ca43/CI38048944005和For来源依赖均已关闭。历史CLI/raw/源码与旧失败不覆盖，详情见各自verification-root和71单元账本。

## 资源与分支

用户已明确批准 **5GiB最低余量/本仓target1GiB上限**、一秒进程组中止、完成cargo clean。root独占Git/Cargo/rustfmt/JDK/JADX/CLI；Luna只做private准备或只读审计，不创建占用分支。

本片已实际cargo clean3022files/669.3MiB，CLI与所有源码/class/raw保留。前次d985主线真实审计15 worktrees全部clean，main/origin相同，仅main，14辅助worktrees均detached main祖先，无target/待合入/分支占用；保护副本保留。6118e565提交推送后已实际核15worktrees全部clean、仅main、main/origin相同、14辅助detached main祖先且无target/fuzz target，free68,071,964,672 bytes，证据在return-arm/results/clean-main-checkpoint-root-v1。关闭记录推送后再次private实核，才能应用下一片。

## 独立债务

本片真实反例显示外层named Try→Loop→If：Frame::protected设置own_try，Frame::loop_body清为None，普通named catch没有现有finally/guard证书恢复它，故If then以ExceptionEdge拒绝。整方法fallback还只有blockleader来源、physical21缺span。这两项单列在If/results/exception-case-architecture-root-v1.md，不扩大本片。还有arithmetic iadd@5来源、flat Signature generic arity、receiver-tail预算、CF16浅层组合、handler ctor goto19和else-if缩进，均按原独立债务推进。

长期账本：[71单元summary](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)。当前片已完成，关闭记录再次核clean后推进局部类型片；持续目标仍active，不暂停也不宣称已完成。

并行方向只读清单在下一片results/next-ledger-triage-luna-v1.md：CF-12整数switch、DT-02静态成员类、EM-23字段更新按本地JADX活动断言/已有完整基线扩验；CF-12已实跑五份原上游Java fixture/六方法，见 [完整基线](openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/README.md)：JADX六腿全类编译运行同原，Jarde六同/两异/四编译失败。类型恢复 OpenSpec recover-proved-local-source-types 为2/8（root v6基线与同次真实IR/架构审查已接受，生产未应用），Luna私有类型v3实现/永久测试准备完成待root审查与真实工具链；原始边界12命令/4原腿已独立核，不能算候选验收。条件fallthrough真实11块/17Normal诊断已独立接受，OpenSpec recover-proved-conditional-switch-fallthrough为1/7；静态消费审查发现group-level fall_through不能表达局部break，root已决定必要SwitchBreak证据叶、最近作用域准确消费现有ASTBreak；Luna private准备中，不能只发map。常量名门另记债务。尚未接受整CF12。

## 最新活动片：调用结果 pop 来源

return-arm已7/7闭合，关闭记录e84600a65实际推送并private再次审计15worktrees全部clean、仅main/无target，才应用typed。typed首次实际2轮2/1：null/String全物理来源与atomic Stop通过；char正确正文但全来源仍缺append后pop82/92/105，失败raw保留，尚未验收。已完整保存typed实际diff/永久测试并恢复build.rs到HEAD，测试移出自动测试目录，typed仍2/8。

独立OpenSpec preserve-proved-discarded-call-origins已完整规划strict通过，复用现有discards证据在两条成功普通调用语句路径追加完整Stmt derived pop并计费/poll；不扩证明计划，不混入局部类型。先用无typed依赖的完整真实类独立验收及自身CI/clean，再重新应用typed，保留全物理断言。Luna private准备patch/fixtures，root独占工具链。条件switch v2草稿已保留尚未root全文验收；typed replay v2修正合同后尚未执行。
