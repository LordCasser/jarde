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

最新已验收产品为 **a738a8941**（构造参数转换），及属性文档修正 **8f624ea64cd9d8e73810cc3a2384c773fa7f87bd**，均已推送main；[CI37968418985](https://github.com/LordCasser/jarde/actions/runs/37968418985)四job、48steps全部success，包含双seed、真JDK25 oracle、完整Java对照、MSRV、supply chain与fuzz。源码blob及冻结CLI核验入口为[数值片root验收](openspec/changes/recover-constructor-primitive-conversion-arguments/verification-root.md)。未来产品WIP不能借这次CI。

## 当前接续：子数组协变初始化器

[recover-covariant-child-array-initializers](openspec/changes/recover-covariant-child-array-initializers/verification-root.md)规划4/4、任务 **2/7**：前置确切CI身份与历史六类完整基线已验收，本片产品未实施。root历史基线2503checks/0errors；两真实JDK的五有效类加载控制和一个无效返回自检均通过，未调用目标方法，不计恢复成功。继续前读该片proposal/design/specs/tasks及focused-control-map；先解除child ownership类型相等早截断，Builder仍使用准确store BCI/source/target事实。Luna限定实现、root串行测试/完整六类双JDK对照与验收。

## 已完成：构造参数数值转换

[recover-constructor-primitive-conversion-arguments](openspec/changes/recover-constructor-primitive-conversion-arguments/)规划4/4、任务 **7/7**，产品、focused、root完整语义对照及本地和确切代码CI全部完成。352目标本地双seed覆盖、MSRV1.88、CI-exact Clippy及显式Java对照通过；前述已推送产品/CI身份是最新验收入口。

普通constructor只接纳真实argument_dependencies内的PrimitiveConversion，复用已有15opcode/Cast、源类别与descriptor；新路径触发既有逐指令+唯一consumer handler ordinal闭包。ordinary生产census/verifier透传本次Budget，Result/Stop直接到停止报告，不发表局部Sites。不新增pass/AST/type table或conversion折叠。receiver-tail收尾扫描的既有漏计另记[债务](openspec/changes/recover-constructor-primitive-conversion-arguments/results/receiver-tail-budget-debt-v1.md)，不能宣称整个census完全计费。

冻结CLI `/private/tmp/jarde-constructor-primitive-conversions-cli-v1` SHA **3e4b615e0131d041ecc47c72c0131a3bbd0f5fc9042d8f91b2aa21f26baf6a95**；当前生产init/build/report/Cargo.lock与它匹配。完整两个顶层类、全部原始成员、27目标方法/32构造点，两真实JDK隔离全部source重编并-Xverify:all运行，原始exit/stdout/stderr **2/2一致**。root独立核验221checks/0errors。JADX none/default四腿虽compile/run0，却删除long→float/double→long舍入链，语义0/4；本片保留每层Cast，以原程序为oracle。

legacy全部22腿保持18/22成功：旧18矩阵16/18，旧完整六类factory2/2，旧完整六类direct仍0/2，BigDecimal两腿仍失败。boxedDirect五wrapper及Integer已恢复，direct剩余numberGridDirect/collectionGridDirect/ownGridDirect三个子数组场景未恢复。没有删除失败成员或借原class编译。该回放核对旧冻结原流hash，不宣称本次重跑原程序。

focused-init-v3 27pass；focused-integration-v3五条通过；既有15opcode四条含ignored完整原/恢复执行通过。Boolean派生控制直接Boolean load→i2b，完整正文拒绝且保留结构/文本分层来源，不执行mutant。controls-v2 handler正例完整四方法，两真实JDK空CP/SP重编成功，无main没有runtime对照；早期handler作用域失败均保留。

Reader实际census1063/4616/463/2711/8，新增8classes/84bodies/4handlers；P5六case合计IrItems+202、AnalysisSteps+1982，其他七维/文本/outcomes不变，新严格pins通过。指纹新增21条目，regen/verify通过。原始失败、命令、双流与hash均在本片results，不覆盖旧版本。

第一轮全仓seed1-v1在磁盘低于20GiB时中止exit-15，不计通过；cargo clean只清本项目，释放约13GiB。fresh seed-v2使用[runner v2](openspec/changes/recover-constructor-primitive-conversion-arguments/results/run-root-gate-v2.py)，DEV/TEST strip=symbols、debug=0，debug assertions和opt-level保持默认；每2秒检查空间，只终止自己隔离命令组。最终状态按各root-* / result.json核对，未出现result不能报通过。

3.2本地门禁已完成：352目标双seed覆盖，各3336passed/0failed/93ignored；213目标复用双seed成功记录，139目标重新执行，不冒称两次fresh全仓命令。MSRV1.88、CI-exact Clippy、fmt、显式Java两组对照、OpenSpec strict及diff均通过。随后确切代码8f624ea64的CI四job/48steps全部通过，3.3完成；当前本片7/7。

## 已完成片与剩余边界

[compose-constructed-reference-array-elements](openspec/changes/compose-constructed-reference-array-elements/verification-root.md) **8/8任务**：保留arrays→sites顺序，准确store BCI/completed ValueId许可，candidate-local子数组/嵌套Site共同闭合后单次移交，无全计划clone。新完整七类两真实JDK成功2/2，原18有效腿由8/18增至16/18；旧factory完整六类2/2，旧direct完整六类仍0/2。BigDecimal两腿compile0但正文拒绝/双流不符，不计成功。首seed暴露公共interval检查遗漏，恢复共同postlude后原负例通过，真实失败永久保留。

本片最终本地双seed各351targets、3328passed/0failed/93ignored；MSRV1.88、fmt、CI-exact Clippy、P5/指纹、显式Java对照、OpenSpec strict及diff全部成功。源码与冻结CLI2 SHA **78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273**一致；实际CI原JSON SHA f7ac30572c1c7b66c6375661c7c1176166c9b9f411f52baf6031865f917ff72e。CLI保留于/private/tmp/jarde-em18-composition-cli-v2。

前置[异构数组类型片](openspec/changes/recover-heterogeneous-array-init/verification-root.md)代码29dcd5e89、CI37926854875全部成功；[引用槽生命周期片](openspec/changes/split-proved-reference-slot-lifetimes/verification-root.md)代码4fba93438、CI37915062977全部成功。完整LG/finalize、同名LVT变体、TWR真实javac8两资源仍各自保留边界，不能把窄片验收当整个单元完成。

数值片之后，优先按已登记的[child-array协变架构审计](openspec/changes/recover-heterogeneous-array-init/results/child-array-covariant-architecture-audit.md)提出独立change：ownGrid的精确component==child_type在候选阶段先断链；reader/value identity及ownership闭合与Javaassignability呈现须分层，后者复用准确store BCI平台/Runtime snapshot事实。类型未证时完整正文fallback，保留真实来源。该片已提出为[recover-covariant-child-array-initializers](openspec/changes/recover-covariant-child-array-initializers/)，规划4/4、任务2/7，历史基线root独立2503checks/0errors，1.1/1.2已完成。产品尚未实施；不能把设计审计当成功结果。BigDecimal平台事实另片处理。下一片具体验收与当前代码依据见[只读规划](openspec/changes/recover-constructor-primitive-conversion-arguments/results/next-child-array-covariance-plan-v1.md)。collectionGrid的Signature refusal尚未证明是独立根因，先解除子数组正文失败再完整复测。

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数，612是JADX测试文件数，均不是成功率。EM18仍部分已测，分母及全单元分类不变。

## 工作树、磁盘与执行纪律

只有main/origin/main，无剩余分支占用。14个辅助worktree均detached、干净、main祖先、无target，保留受Codex保护的副本；没有遗留工作等待合并。最新实际审计见数值片results/worktree-audit-after-local-acceptance-v2.json，主仓仅本轮清理/只读审计/下一片规划证据。

只由root串行执行Cargo/Git/rustfmt；一次一个Cargo，运行期间冻结产品/测试/canonical fixture。使用CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1，debug信息关闭但assertions开启；**20GiB为停建线**，不碰其它项目target。构造组合片中途清13.2GiB、最终清814.8MiB；本轮最终门禁后的cargo clean已实际删除7681文件、894.1MiB，主仓target当前不存在；fuzz及14辅助树此前已无target，实时空间以df为准。

全部生成source必须参与空classpath/sourcepath重编，runtime仅用其新编译classes并-Xverify:all；原始exit/stdout/stderr逐字核对，结构事实、正文质量和执行成功分别计证据。不得借原class/helper、剥离失败成员或修剪原始日志。保留所有历史失败、完整输入和冻结CLI。

此前完整handoff已保存在数值片results/handoff-before-primitive-conversions-v1.md。
