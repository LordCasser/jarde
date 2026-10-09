# HANDOFF — jarde 当前主线与接续入口

用户明确要求继续在当前聊天推进，不暂停持续目标。root负责架构、OpenSpec和对抗验收，确定性实现交给Luna；以JADX的71单元账本逐片推进，不恢复随机巡查。

## 先核对实际状态

```sh
git status --short
git log -1 --oneline
git rev-parse HEAD origin/main
git branch -av
git worktree list
```

最新已验收产品代码为 **5a6264de1b3481e1cdbef4fd30361940bfac33cf**，已推送main；[CI37947791151](https://github.com/LordCasser/jarde/actions/runs/37947791151)四job、48steps全部success，含双seed、JDK25 oracle、完整Java对照和fuzz。后续文档提交不改变该代码身份；未来产品WIP不能借这次CI结果。实际JSON和源码/CLI身份核对入口为[构造组合root验收](openspec/changes/compose-constructed-reference-array-elements/verification-root.md)。

## 当前下一片：构造参数数值转换

[recover-constructor-primitive-conversion-arguments](openspec/changes/recover-constructor-primitive-conversion-arguments/)规划4/4、任务 **2/7**：前置确切代码CI与完整基线已完成，产品尚未实施。继续前先读proposal、design、specs、tasks及[本片root验收](openspec/changes/recover-constructor-primitive-conversion-arguments/verification-root.md)。

下一步由Luna限定实现2.1/2.2，再交root真实双JDK完整对照：

1. 普通constructor实际argument_dependencies中放行PrimitiveConversion，复用现有15opcode、逐层Cast、源类别/Boolean和constructor descriptor验证；不新增pass/AST/table，不删舍入链。
2. 仅新conversion路径触发既有allocation→constructor→唯一consumer的handler ordinal闭包，不扩旧普通invoke所有handler债务。
3. 普通Site生产入口目前经verify创建budget:None。将report本次Budget传入已有verify_metered，Result原子返回Stop；Refusal照常登记。必要地补这个入口的漏计，不用feature计费开关、预扫或长度估价；测试unmetered入口不能作预算证据。
4. 补真实SSA/extra-dup2、无关conversion、handler不同/同覆盖、ordinary路径预算和取消、Boolean正文拒绝。PrimitiveLongPair实际constructor为(JJ)，复用现有pool entries和dup2，不能假称存在Pair(IJ)。stored-local生产者只执行一次，两个参数各自load/转换。

完整基线从创建起保留两个顶层类及所有成员：五wrapper、Integer对照、六元素Number[]、普通return-new、stored-local、15转换和三种舍入链。原程序两真实JDK均成功；旧Jarde两腿拒绝且compile失败。JADX none/default四腿均compile/run0但删掉两种long经float/double往返的cast，16777217和9007199254740993没有产生原程序的舍入结果，语义 **0/4**。原程序是oracle，不能以JADX exit0替代行为正确。

永久入口：results/baseline-v1/manifest.json、baseline-root-verification-v2.json、baseline-method-review-v1.json、antecedent-freeze-root-v1.json、jadx-cast-chain-audit-v1/audit.json。root独立核对6比较腿零问题，旧完整family110文件hash重查零问题；verifier v1误用另一JDK flags的真实失败与修复保留。基线已冻结，不覆盖或删成员。源码草稿虽位于draft-sources-v1，内容身份已由baseline固定，后续修改需新版本及重新对照。

成员/statement-position news、一般alias/phi、nested covariant child-array和BigDecimal不扩围。当前Luna只读预审已完成，须root明确派发后才编辑；后续实际状态以git diff为准。

## 已完成片与剩余边界

[compose-constructed-reference-array-elements](openspec/changes/compose-constructed-reference-array-elements/verification-root.md) **8/8任务**：保留arrays→sites顺序，准确store BCI/completed ValueId许可，candidate-local子数组/嵌套Site共同闭合后单次移交，无全计划clone。新完整七类两真实JDK成功2/2，原18有效腿由8/18增至16/18；旧factory完整六类2/2，旧direct完整六类仍0/2。BigDecimal两腿compile0但正文拒绝/双流不符，不计成功。首seed暴露公共interval检查遗漏，恢复共同postlude后原负例通过，真实失败永久保留。

本片最终本地双seed各351targets、3328passed/0failed/93ignored；MSRV1.88、fmt、CI-exact Clippy、P5/指纹、显式Java对照、OpenSpec strict及diff全部成功。源码与冻结CLI2 SHA **78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273**一致；实际CI原JSON SHA f7ac30572c1c7b66c6375661c7c1176166c9b9f411f52baf6031865f917ff72e。CLI保留于/private/tmp/jarde-em18-composition-cli-v2。

前置[异构数组类型片](openspec/changes/recover-heterogeneous-array-init/verification-root.md)代码29dcd5e89、CI37926854875全部成功；[引用槽生命周期片](openspec/changes/split-proved-reference-slot-lifetimes/verification-root.md)代码4fba93438、CI37915062977全部成功。完整LG/finalize、同名LVT变体、TWR真实javac8两资源仍各自保留边界，不能把窄片验收当整个单元完成。

数值片之后，优先按已登记的[child-array协变架构审计](openspec/changes/recover-heterogeneous-array-init/results/child-array-covariant-architecture-audit.md)提出独立change：ownGrid的精确component==child_type在候选阶段先断链；reader/value identity及ownership闭合与Javaassignability呈现须分层，后者复用准确store BCI平台/Runtime snapshot事实。类型未证时完整正文fallback，保留真实来源。该片尚未提出/实现，不能把设计审计当成功结果。BigDecimal平台事实另片处理。

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数，612是JADX测试文件数，均不是成功率。EM18仍部分已测，分母及全单元分类不变。

## 工作树、磁盘与执行纪律

只有main/origin/main，无剩余分支占用。14个辅助worktree均detached、干净、main祖先、无target，保留受Codex保护的副本；没有遗留工作等待合并。最新实际审计见数值片results/worktree-audit-before-baseline-commit.json，主仓当时仅本轮文档/基线WIP。

只由root串行执行Cargo/Git/rustfmt；一次一个Cargo，运行期间冻结产品/测试/canonical fixture。使用CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1，debug信息关闭但assertions开启；**20GiB为停建线**，不碰其它项目target。构造组合片中途清13.2GiB、最终清814.8MiB；本轮主仓/fuzz及14辅助树均无target，实时空间以df为准。

全部生成source必须参与空classpath/sourcepath重编，runtime仅用其新编译classes并-Xverify:all；原始exit/stdout/stderr逐字核对，结构事实、正文质量和执行成功分别计证据。不得借原class/helper、剥离失败成员或修剪原始日志。保留所有历史失败、完整输入和冻结CLI。

此前完整handoff已保存在数值片results/handoff-before-primitive-conversions-v1.md。
