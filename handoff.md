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

最新已验收产品为 **45b4848c558f4f5d317720535cea32fe431288bf**，已推送main；[CI37981309004](https://github.com/LordCasser/jarde/actions/runs/37981309004)四job/48steps全部success。两个fresh全workspace seed各352test-result记录、3343passed/0failed/93ignored；真JDK25 oracle、显式Java执行、MSRV、Clippy、supply chain/fuzz及OpenSpec全部通过。五源产品blob与冻结CLI相同，原JSON/完整gzip日志/root验收入口见[wildcard返回验收](openspec/changes/recover-reifiable-wildcard-array-return-signatures/verification-root.md)。随后验收文档/下一片规划提交没有产品变化；不把后续docs-only头部的CI冒称已绿。

## 当前接续：BigDecimal→Number 最小事实片

两数组补片已完成：[子数组协变](openspec/changes/recover-covariant-child-array-initializers/verification-root.md) **7/7**、[可具体化 wildcard 数组返回](openspec/changes/recover-reifiable-wildcard-array-return-signatures/verification-root.md) **6/6**。结构ownership与准确store赋值分层接通，完整同次ArrayCreation证据支持原Collection<?>[][]声明，没有新增pass/type service。

五源CLI `/private/tmp/jarde-wildcard-array-return-cli-v1` SHA196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761，fresh24腿22成功：旧18矩阵16/18、完整六类factory2/2、direct2/2、数值2/2，BigDecimal两已知失败保留。root独立1166checks同时验收完整成员、全部生成源、两真实JDK隔离重编/runtime双流、Collection<?>[][]声明与物理Signature/来源/次数。成功证据marker保留且拒绝消失；original/JADX采用逐hash核验的历史基线，未fresh执行。历史四源CLI1116checks只semantic/Signature pending，不能替代本次验收。

325 Java-lib、16 facade focused、5数组完整集成及11项本地严格门禁通过，P5 pins/指纹未变化。全workspace双seed与真JDK25由上述确切产品CI补齐，不冒称本地全仓重跑。本仓target已实际清5926files/845.2MiB，不再存在；冻结CLI和历史可恢复压缩均保留。机器空间一度因外部活动降到约19GiB，等待CI期间未启动Cargo；收尾时实际恢复到约30GiB。20GiB停建线不变，本项目没有待清target，不碰其他项目target。

下一[recover-bigdecimal-number-widening](openspec/changes/recover-bigdecimal-number-widening/)规划 **4/4**、strict通过、任务 **2/7**（前置CI/准确基线），**产品未实施**。当前普通Main-only两BigDecimal腿构造已通过，但aastore15缺BigDecimal→Number事实。root六条显式header诊断在Main字节完全不变情况下，仅提供真实Corretto8 BigDecimal.class即恢复完整Main；四条正例在真实JDK8/23空CP/SP重编、新类-Xverify:all运行/raw双流一致，两个无header负基线仍失败。完整普通StringBuilder调用链已正确保留，concat优化warning不等于正文失败；下一片只补release-8精确类型pair，不改concat白名单。

入口：[proposal](openspec/changes/recover-bigdecimal-number-widening/proposal.md)、[design](openspec/changes/recover-bigdecimal-number-widening/design.md)、[tasks](openspec/changes/recover-bigdecimal-number-widening/tasks.md)。现有Luna待按apply限定实现2.1/2.2，root负责所有Cargo/Git/冻结CLI/24腿完整对照/确切CI/验收。实际header与原始两输入及历史JADX身份、完整诊断/执行均在前片results/bigdecimal-selected-headers*和新片results/release8-header-root-v1.json。v1静态concat方向被真实v2诊断及执行收窄，两版历史保留；不能凭v1另造机制。

[71单元后续队列审计](openspec/changes/recover-bigdecimal-number-widening/results/next-ledger-priority-audit-v1.md)仅建议EM18收尾后复测CF16非直线finally与DT26数组元素lambda捕获；尚未fresh确认或授权为新实现片，不重复已完成窄片。71计数/EM18部分已测分类保持。

本地前次全仓首lib/seed1曾被20GiB guard停止exit-15，不计成功；历史root清target818.3MiB并可恢复压缩17个CLI节省932116990bytes，restore入口见child results/historical-cli-compression-v1.json。平坦Signature真实generic arity仍是[共享债务](openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/flat-signature-arity-debt-v1.md)，未复现非法元数控制不当作已拒绝，不混入当前片。

## 已完成：构造参数数值转换

[recover-constructor-primitive-conversion-arguments](openspec/changes/recover-constructor-primitive-conversion-arguments/)规划4/4、任务 **7/7**，产品、focused、root完整语义对照及本地和确切代码CI全部完成。352目标本地双seed覆盖、MSRV1.88、CI-exact Clippy及显式Java对照通过；前述已推送产品/CI身份是最新验收入口。

普通constructor只接纳真实argument_dependencies内的PrimitiveConversion，复用已有15opcode/Cast、源类别与descriptor；新路径触发既有逐指令+唯一consumer handler ordinal闭包。ordinary生产census/verifier透传本次Budget，Result/Stop直接到停止报告，不发表局部Sites。不新增pass/AST/type table或conversion折叠。receiver-tail收尾扫描的既有漏计另记[债务](openspec/changes/recover-constructor-primitive-conversion-arguments/results/receiver-tail-budget-debt-v1.md)，不能宣称整个census完全计费。

冻结CLI `/private/tmp/jarde-constructor-primitive-conversions-cli-v1` SHA **3e4b615e0131d041ecc47c72c0131a3bbd0f5fc9042d8f91b2aa21f26baf6a95**；前片验收时生产init/build/report/Cargo.lock与它匹配；当前child片build.rs已变化，不能借该CLI身份。完整两个顶层类、全部原始成员、27目标方法/32构造点，两真实JDK隔离全部source重编并-Xverify:all运行，原始exit/stdout/stderr **2/2一致**。root独立核验221checks/0errors。JADX none/default四腿虽compile/run0，却删除long→float/double→long舍入链，语义0/4；本片保留每层Cast，以原程序为oracle。

legacy全部22腿保持18/22成功：旧18矩阵16/18，旧完整六类factory2/2，旧完整六类direct仍0/2，BigDecimal两腿仍失败。boxedDirect五wrapper及Integer已恢复，direct剩余numberGridDirect/collectionGridDirect/ownGridDirect三个子数组场景未恢复。没有删除失败成员或借原class编译。该回放核对旧冻结原流hash，不宣称本次重跑原程序。

focused-init-v3 27pass；focused-integration-v3五条通过；既有15opcode四条含ignored完整原/恢复执行通过。Boolean派生控制直接Boolean load→i2b，完整正文拒绝且保留结构/文本分层来源，不执行mutant。controls-v2 handler正例完整四方法，两真实JDK空CP/SP重编成功，无main没有runtime对照；早期handler作用域失败均保留。

Reader实际census1063/4616/463/2711/8，新增8classes/84bodies/4handlers；P5六case合计IrItems+202、AnalysisSteps+1982，其他七维/文本/outcomes不变，新严格pins通过。指纹新增21条目，regen/verify通过。原始失败、命令、双流与hash均在本片results，不覆盖旧版本。

第一轮全仓seed1-v1在磁盘低于20GiB时中止exit-15，不计通过；cargo clean只清本项目，释放约13GiB。fresh seed-v2使用[runner v2](openspec/changes/recover-constructor-primitive-conversion-arguments/results/run-root-gate-v2.py)，DEV/TEST strip=symbols、debug=0，debug assertions和opt-level保持默认；每2秒检查空间，只终止自己隔离命令组。最终状态按各root-* / result.json核对，未出现result不能报通过。

3.2本地门禁已完成：352目标双seed覆盖，各3336passed/0failed/93ignored；213目标复用双seed成功记录，139目标重新执行，不冒称两次fresh全仓命令。MSRV1.88、CI-exact Clippy、fmt、显式Java两组对照、OpenSpec strict及diff均通过。随后确切代码8f624ea64的CI四job/48steps全部通过，3.3完成；当前本片7/7。

## 已完成片与剩余边界

[compose-constructed-reference-array-elements](openspec/changes/compose-constructed-reference-array-elements/verification-root.md) **8/8任务**：保留arrays→sites顺序，准确store BCI/completed ValueId许可，candidate-local子数组/嵌套Site共同闭合后单次移交，无全计划clone。新完整七类两真实JDK成功2/2，原18有效腿由8/18增至16/18；旧factory完整六类2/2，旧direct完整六类仍0/2。BigDecimal两腿compile0但正文拒绝/双流不符，不计成功。首seed暴露公共interval检查遗漏，恢复共同postlude后原负例通过，真实失败永久保留。

本片最终本地双seed各351targets、3328passed/0failed/93ignored；MSRV1.88、fmt、CI-exact Clippy、P5/指纹、显式Java对照、OpenSpec strict及diff全部成功。源码与冻结CLI2 SHA **78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273**一致；实际CI原JSON SHA f7ac30572c1c7b66c6375661c7c1176166c9b9f411f52baf6031865f917ff72e。CLI保留于/private/tmp/jarde-em18-composition-cli-v2。

前置[异构数组类型片](openspec/changes/recover-heterogeneous-array-init/verification-root.md)代码29dcd5e89、CI37926854875全部成功；[引用槽生命周期片](openspec/changes/split-proved-reference-slot-lifetimes/verification-root.md)代码4fba93438、CI37915062977全部成功。完整LG/finalize、同名LVT变体、TWR真实javac8两资源仍各自保留边界，不能把窄片验收当整个单元完成。

数值片之后，优先按已登记的[child-array协变架构审计](openspec/changes/recover-heterogeneous-array-init/results/child-array-covariant-architecture-audit.md)提出独立change：ownGrid的精确component==child_type在候选阶段先断链；reader/value identity及ownership闭合与Javaassignability呈现须分层，后者复用准确store BCI平台/Runtime snapshot事实。类型未证时完整正文fallback，保留真实来源。该片已提出为[recover-covariant-child-array-initializers](openspec/changes/recover-covariant-child-array-initializers/)，规划4/4、任务2/7，历史基线root独立2503checks/0errors，1.1/1.2已完成。当前已进入限定实现、focused4/7已验收，完整回放/门禁/CI未完；不能把设计审计当成功结果。BigDecimal平台事实另片处理。下一片具体验收与当前代码依据见[只读规划](openspec/changes/recover-constructor-primitive-conversion-arguments/results/next-child-array-covariance-plan-v1.md)。此前规划时collectionGrid的Signature根因未明；实际子数组完整回放已定位独立NewArray泛型返回候选缺口，当前独立片进展见上节。

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数，612是JADX测试文件数，均不是成功率。EM18仍部分已测，分母及全单元分类不变。

## 工作树、磁盘与执行纪律

只有main/origin/main，无剩余分支占用。14个辅助worktree均detached、干净、main祖先、无target，保留受Codex保护的副本；没有遗留工作等待合并。最新实际审计见wildcard片results/worktree-audit-before-commit-v1.json；本轮验收文档与下一片规划提交推送后主仓干净。

只由root串行执行Cargo/Git/rustfmt；一次一个Cargo，运行期间冻结产品/测试/canonical fixture。使用CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1，debug信息关闭但assertions开启；**20GiB为停建线**，不碰其它项目target。构造组合片中途清13.2GiB、最终清814.8MiB；数值片最终clean删除7681文件/894.1MiB；最新wildcard片最终clean删除5926文件/845.2MiB，当前主仓target不存在。fuzz及14辅助树此前已无target，实时空间以df为准。

全部生成source必须参与空classpath/sourcepath重编，runtime仅用其新编译classes并-Xverify:all；原始exit/stdout/stderr逐字核对，结构事实、正文质量和执行成功分别计证据。不得借原class/helper、剥离失败成员或修剪原始日志。保留所有历史失败、完整输入和冻结CLI。

此前完整handoff已保存在数值片results/handoff-before-primitive-conversions-v1.md。
