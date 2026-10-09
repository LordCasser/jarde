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

## 当前接续：BigDecimal→Number产品已实现，本地验收通过，确切CI待验收

当前已推送产品 **6997c9f8b2515cb18c359fe55559a486fe47fff1**，[CI37989319644](https://github.com/LordCasser/jarde/actions/runs/37989319644)已经failure：stable seed1命中过时BigDecimal闭集负例；其余MSRV/supply/fuzz成功，seed2/显式BigDecimal未执行。两个旧test的准确预期已按fresh四份报告修正，五产品源未改，修正提交的新确切CI待验收。

当前产品仅在现有release-8 NUMBER_FAMILY补BigDecimal→Number准确row，不改concat、不新增pass。Luna实现，root验收；[本片](openspec/changes/recover-bigdecimal-number-widening/)任务 **5/7**。两ordinary生产结构测试、325 Java-lib、双真实JDK显式ignored执行（8次完整source重编/runtime）全部成功。

新五源CLI `/private/tmp/jarde-bigdecimal-number-cli-v1` SHA **b79520629443a0211cd656d1374e415f76ac6adf400d79cb86154954caba9cb0**；当前source与metadata一致。完整原24腿本次fresh **24/24**，root独立 **1226checks/0errors**，102commands/384closedfiles，两真实JDK空CP/SP编译所有生成source、只运行新classes且-Xverify:all，匹配原exit/raw双流；original/JADX采用逐hash核对历史基线，非fresh执行。完整两BigDecimal、旧八家族/factory/direct/数值均过；准确Collection<?>[][]声明/成功Signature marker与物理BCI仍保持。不能把24控制腿称作71单元完成。

reader census测量更新为(1067,4626,463,2711,8)，只新增4classes/10直线Code body；指纹仅增11条输入；P5严格pins不变。MSRV1.88、CI-exact Clippy、fmt、OpenSpec all strict、diff、finally两专项均过。原旧pin/指纹失败、首CLI20GiB停建exit-15及finally verifier v1误要求essential可选source-map的失败均保留。CLI v2成功；末次仅清本仓target，冻结CLI/raw证据保留。机器空间实时以df为准，20GiB守卫继续有效。其它项目Cargo使空闲约19GiB，本仓target无可清；当前只完成小规模Java夹具/补丁准备，暂不本地Cargo。

下一步：先核对CI修正提交的新run，不重用failed37989319644。保留其原JSON/gzip log/ci-failure-root-v1.json；用版本化CI verifier核验**确切修正产品**的四job、全部steps、两个fresh workspace seed及新增BigDecimal显式JDK执行。CI原JSON/完整gzip日志未落盘前，不勾3.2/3.3，不借45b前置CI；本地未重复全workspace双seed。验收入口：[verification-root](openspec/changes/recover-bigdecimal-number-widening/verification-root.md)、results/local-root-acceptance-v1.json、candidate-root-verification-v1.json。完成CI后再更新任务/账本/handoff并提交验收文档。

## 71账本队列：避免重复已实现片

CF16 ImplicitCleanup已经恢复：当前CLI完整类all/essential文本相同，双真实JDK四腿16路径全部匹配原行为；原cleanup-over-return trace29，历史JADX输出本次双JDK重编仍trace299。all保留17个来源BCI，essential CLI按请求不携带可选map；widened保持引用且不执行。current-finally-v1与独立v2证据在本片results。尚不能从语义正例推断structured子Region停止/回滚全部边界任务已完，但不要重复实现其主体。

DT26原生int[] capture也已由旧片恢复。准确下一缺口是P02_multianewarray中lambda helper的二维数组元素compound更新，capture本身成功。当前新CLI两真实编译器原class均stdout6\n，候选完整类两JDK均compile1/helper fallback；不是旧README所写的compile0/打印0。新baseline在results/nested-array-update-baseline-v1，尚未freshJADX、未立新实现spec。先核对现有prove_array_update/array_of_value/IndexAssign的准确行对象ValueId、dup2复制及求值次序闭包，再判断是否需机制。71/EM18部分分类不变。

两数组补片已完成：[子数组协变](openspec/changes/recover-covariant-child-array-initializers/verification-root.md)7/7、[wildcard数组返回](openspec/changes/recover-reifiable-wildcard-array-return-signatures/verification-root.md)6/6。前产品45b的确切CI已验收，raw证据保留。平坦Signature真实generic arity是独立债务，不混入本片。17旧CLI可恢复gzip记录在child results/historical-cli-compression-v1.json，旧失败和所有冻结输入保留。

下文是已完成窄片与历史边界，若旧阶段状态与上述最新入口冲突，以本节为准。

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
