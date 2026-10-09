# Root Verification — Constructor Primitive Conversion Arguments

本片7/7任务：基线、限定实现、root完整语义对照、本地门禁及确切代码CI均已验收，产品a738a8941与属性文档修正8f624ea64已推送main。前置代码5a6264de的CI37947791151四job/48steps全部success，不能借前片CI判本片最终成功。实际基线对照允许在CI等待期完成，产品改动在前置CI全部成功后才开始。

## Frozen Complete Baseline

从创建起两个完整顶层类ConstructorPrimitiveConversionControls与PrimitiveLongPair，不剥离任何成员。原源码四类mark、五wrapper、Integer对照、fresh六元素Number[]、普通return new、stored-local重复参数、全部15转换及三层舍入链全部由main显式观察。两真实JDK原始输入source8/target8或release8、g:none、显式空classpath/sourcepath；全部原/反编译source独立编译，只在各自新编译目录-Xverify:all运行。

baseline-v1 manifest SHA `0afb246902e6a6b76c7ff17939313d22810dcc586eca63f2741f7e806aa5858d`，冻结CLI2 SHA78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273，源码两份分别4e5aff3dced47b6493a3db64b8fe2998a0c9328fde22ce5c7fd745d1e03e2c1e、904ecf774d469580f0afe716d82b1c12744fdee721274fe69e31f3507cb36e60。

| profile | 完整compile | verified run exit0 | 原始双流一致 | 语义验收 |
|---|---|---|---|---|
| 原程序 | 2/2 | 2/2 | 两原腿一致 | 2/2 |
| 冻结Jarde CLI2 | 0/2 | 不运行失败source | 0/2 | 0/2 |
| JADX none | 2/2 | 2/2 | 0/2 | 0/2 |
| JADX default | 2/2 | 2/2 | 0/2 | 0/2 |

Root独立verifier核查实际文件闭集、hash/bytes、原源码复制身份、两class jar entries、两个JDK与CLI2、实际全部source参与空CP/SP编译、私有runtime目录与原始exit/stdout/stderr；全部6比较腿核验零问题，见results/baseline-root-verification-v2.json（SHA `1c200d0989599e059d73373fd921105ef7e446073931a64108694eeab2e866b6`）。初verifier错误沿用javac23 flags核对javac8候选，真实exit1及3错误永久保留于v1脚本/JSON；v2按腿重绑定，未回写baseline或失败证据。

## Actual JADX Cast Loss

完整已安装JADX四腿都删除longViaFloat的l2f/f2l和longViaDouble的l2d/d2l；输入16777217L及9007199254740993L，原程序返回16777216L及9007199254740992L，JADX返回输入值。doubleViaFloat仍保留float cast并匹配。两原class的javap BCIs10/11明确是两个转换，constructor在12；不是源码常量折叠推测。结果及57个installed jar/launcher hash、参考SimplifyVisitor/InsnDecoder/JavaInsnsRegister源码hash见results/jadx-cast-chain-audit-v1/audit.json（SHA `0c8add88e5388db1d3c473ab1ac640ae65626d798d86b8139f439e3cd2fb4b24`）。SimplifyVisitor的processCast/shadowedByOuterCast是算法审查参考，未插桩，不断言运行时精确触发分支。Jarde复用已有Cast保留每层，不照抄删除策略；compile/run0不能充当语义oracle。

## Necessary Budget Plumbing

真实ordinary Site入口verify当前创建VerifyMeter budget:None。数组seam已使用Some(budget)，不能泛化该预算证据。下一实现将单一report调用的同一Budget传入已有verify_metered/census并Result返回Stop，Refusal仍记录；Stop时丢弃局部Sites、report直接stopped，不进入region/build/materialize。必要地修普通construction verifier整个入口既有漏计，不建立feature开关、conversion预扫、长度估价或其它规则迁移。预算/取消须在实际ordinary return-new路径证明，测试unmetered入口不作预算证据。

extra-dup的真实SSA是一读两distinct ValueId，依赖集合不证明single-use，保守拒绝来自StatementFree。stored-local在BCI3 markInt一次、6 istore、7 new、11/13两load、12/14 i2l、15 constructor、18 areturn；旧CLI仍在12结构拒绝，不能把markers数组为空误算该正文成功（真实source有@bytecode）。成员/statement、一般alias、nested covariance与BigDecimal不扩围。

前置冻结结果见results/antecedent-freeze-root-v1.json：CLI/metadata及确切CI绑定，旧factory/direct四完整六源腿文件hash逐项再次匹配；factory2/2、direct0/2，物理BCI/拒绝保留。前置产品门禁已解除，下一步骤为2.1/2.2限定实现。

## Candidate Gate History — 2026-10-10

root串行运行，所有命令的argv/env/源文件hash/exit/raw streams见results/run-root-gate-v1.py及各root-*目录。fmt-v1/v2成功；focused-init-v1编译exit101，新增负控误用Gap接口，修正为测试可访问的精确Refused记录。focused-init-v2实际26pass/1fail：旧second_unsupported_element把i2l当拒绝，本片接通转换后该完整输入正确提交候选，旧断言失效。保留全部原始失败，不覆盖日志，也不修改冻结旧fixture。

当时迁移该旧断言为转换正控，并在第二个new/dup后派生不属于参数依赖的栈中性iconst_0/i2l/pop2负控，继续验证任何元素失败均不提交数组/pending Sites。只读对抗审查另外发现Boolean仅有类别helper单测，遂补构造Site到完整正文拒绝的仅分析控制；该历史阶段尚未勾选2.1/2.2和root完整对照，后续实际通过见下文。

## Root Candidate Acceptance

候选CLI `/private/tmp/jarde-constructor-primitive-conversions-cli-v1` SHA `3e4b615e0131d041ecc47c72c0131a3bbd0f5fc9042d8f91b2aa21f26baf6a95`，源码及实际build身份见results/candidate-cli-v1.json。错误bin名的build-v1 exit101保留；build-v2成功。当前init/build/report/Cargo.lock仍逐项匹配冻结CLI源码。host focused-integration-v1编译借用错误、v2两个边界失败均保留，修正后的v3五条integration全部通过；focused-init-v3为27pass。原15opcode语料四条测试含完整原/恢复源码执行通过。

完整两类数值家族两真实JDK **2/2语义成功**：全部原始成员/source参与显式空CP/SP重编，runtime仅新classes并-Xverify:all，exit/raw stdout/raw stderr相同。27个目标方法、32个唯一构造点及cast/new/dup/init/argument/AASTORE来源都闭合。results/candidate-root-v1/manifest.json SHA `1b6c1f6f44920d7e849d26576687bfefc76409c66599ca0daa592c5a733cfa71`；root独立重算文件闭集、两个JDK、物理class身份、完整方法signature及正文、源码复制、私有compile/run、双流和完整Sites，**221 checks/0 errors**，results/candidate-root-verification-v1.json SHA `c2b9966ffc2be1998dcbde9aad425a36d7a13064b6666d3113854aa184f13892`。这包括JADX删cast的两种舍入链，没有借JADX结果当oracle。

results/legacy-regressions-root-v1/manifest.json SHA `dc8a09f2b9d34bd50f2e8b43f5fbcfda23e2f7bfdca58c9970cf81be962528eb`，全部22腿输入/完整生成源集保留。旧factory完整六类2/2，旧18矩阵仍16/18，BigDecimal两腿仍失败；旧direct完整六类仍0/2，五wrapper的boxedDirect已经Structured/Java，剩余numberGridDirect、collectionGridDirect、ownGridDirect拒绝。这里明确复核冻结原始流hash，不宣称新跑原程序。未删失败成员，不把局部恢复当整类成功。

Boolean派生控制直接iload Boolean→i2b，不经markInt整数返回；双腿真实SSA与NewRecord的0/3/5/6锚保留，完整正文Fallback/Mixed/NotJava并在5精确报告numeric source-category不符，primary consumer9。故意变体不执行。首次过强地要求fallback文本也映射未呈现new的0/3/6，实际拒绝只映射9/5，root-boolean-context-v1真实上下文保留；按结构与文本来源分层修正后root-boolean-v2通过。既有NewRecord/diagnostic的presented措辞债务另记results/boolean-body-boundary-audit-v1.md，不在本片改API。

handler controls-v1的[0,19)同覆盖Site及改start=8负例已过，整正文却有既有catch local作用域拒绝；v3的try外local又触发跨protected路径拒绝，原始源码/class/javap/CLI结果都保留。不扩局部scope实现。独立controls-v2用受保护的identity调用12消费new，完整四方法Structured/Java，源映射实际0/3/4/5/8/9/12/15/16/17/18；两真实JDK全部生成单类source空CP/SP编译成功，无main不声称runtime对照。root实际render与compile命令见root-handler-v4-*目录；standalone误配plain-jar的真实输入错误也保留后以准确jar snapshot重试。

reader实际census变为1063/4616/463/2711/8，新增8class/84bodies/4handlers，无新增branch/subroutine。旧pin失败记录完整保留，按实际reader更新后通过。普通verifier必要共享计费使P5六case合计IrItems+202/AnalysisSteps+1982，两per-method arms同增，其他七维/文本/outcomes不变，旧失败与recorder及新严格pins复验通过；不表述成性能提升。P5 corpus fingerprint新增21文件条目，regen/verify均通过。该段记载时workspace与Clippy最终门禁尚待完成；当前分区双seed结果及Clippy证据见末节。

## Workspace Disk Stop and Fresh Retry

root-workspace-seed1-v1在233.095秒时因空间低于20GiB停建线主动终止本次jarde Cargo及其编译子进程，实际exit -15；该中断不是测试成功，也未启动第二seed-v1。root-clean-after-disk-stop-v1实际cargo clean成功，可用空间由16294432768增至30378835968 bytes，冻结CLI与全部证据保留。其它项目的Cargo/target没有操作。

新run-root-gate-v2.py使用独立进程组，每2秒检查可用空间，低于20GiB仅终止自己的命令组；记录实际exit、停止原因、最小空间及raw streams。fresh seed-v2启用DEV/TEST strip=symbols并保持debug=0、默认debug assertions与opt-level，jobs/incremental/testthreads仍为1/0/1。没有改动Cargo.toml或产品编译配置，也没有覆盖v1证据。

root-workspace-seed1-v2真实exit101，469.601秒，完成135个test-result targets、1328passed/1failed。失败位于旧p3_heterogeneous_array_initializers direct控制：boxedDirect仍要求Byte等五wrapper PrimitiveConversion拒绝，而完整root候选回放已证明本片实际恢复。最低空间26595487744 bytes，未触发停建。第二seed-v2未启动；将该旧boxed正控按准确六Sites/source-map迁移，保留三个grid拒绝及完整direct整类0/2，不改原fixture或产品。历史全仓失败不覆盖，后续使用fresh v3。

迁移后root-fmt-v6及root-heterogeneous-control-v1均exit0，专项四条通过；root审读六Sites BCIs与冻结javap、完整legacy输出对应，cast/mark/constructor各一次，grid负控保持。随后主机可用空间再次低于20GiB，不能启动完整产物重建。root决定改为显式Cargo目标分批双seed，全程保留workspace/all-features/locked与默认assertions/优化级别，每组两seed后仅回收本项目由metadata命名的测试可执行文件。必须逐目标记录覆盖，不能把分批运行冒称两次单一全仓命令。

partitioned-workspace-gates-v1首次分批在第49条命令tests-021/seed1停止，实际exit101；此前223个metadata目标、48条命令双seed全部通过。此次失败为既有straight_finally_reuses_its_guard_verdict_with_a_tight_budget的固定AnalysisSteps52，实际61。冻结run()I九条指令均由新增普通census实际计费，方法无new，不进入构造verifier；region仍复用同一guard verdict，没有新增第二guard proof。仅迁移pin为61，full.produced和tight.text相等断言保持。root-fmt-v7及root-finally-count-pin-v1实际exit0，后者恰好61步完整恢复相同正文。

全部352目标的source SHA已由root在首次runner开跑前加入workspace_targets并记录，实际runner SHA4b9f0d1a66f6aac8d590430f6503f356648b70eab9e4bbbbaf24898f72574c02，不是Luna早期未含source hash的草稿版本。后续版本仅复用目标source/生产before身份未变且两seed实际成功的完整组；失败或不完整组整体两seed重跑。旧失败、raw files及manifest不得覆盖或裁剪。

## Final Production Adversarial Review

root复核results/final-production-adversarial-audit-v1.md及实际生产接口：新转换仅限参数SSA依赖与构造区间，既有handler loop逐指令及sole consumer比较ordinal；Stop由verify到census到report直接传播，局部pending/plan不对外发布。未发现本片放宽single-use或吞掉Stop。普通预算focused正例的pending map为空，非空pending后Stop的组合未单独实测，不将接口结构保证冒称该组合测试。

既有receiver-tail收尾helper尚未接入meter，合并/排序也非逐项计费；已独立记录results/receiver-tail-budget-debt-v1.md，后续另片处理。当前范围为修补普通constructor verifier入口漏计，不能宣称整个census每项工作都已计费。

## Final Dual-Seed Workspace Coverage and Clippy Record

分区续跑最终记录位于`results/partitioned-workspace-gates-v2/manifest.json`（SHA256 `03bf119c613c5c69f4c8c23e62e8bce329d6f28a67f679a0cf6315d5e1b65398`）；独立核验为`results/partitioned-workspace-root-verification-v2.json`（SHA256 `8dc50f950a5c1643f4587a7705aa7885f6780c679f0551d0cce23a52a80e0eb9`）。JSON确认metadata共352 targets、37 partitions、74 seed commands；两seed `5350648285461741569` 与 `5350648285461741570` 各有352条target result、3336 passed、0 failed、93 ignored。Independent verifier为`passed`，errors为空。对应闭集计数是prior 296 files与v2 262 files。分区复用/新跑计数也已按manifest重算：23个partition、213个unique targets在每个seed均复用此前已成功的两腿记录（46条复用command）；14个partition、139个unique targets在v2对每个seed新执行（28条fresh command）。所以213 + 139 = 352精确成立，且每目标仍有两个seed结果。

这些记录是有证据的分区续跑，不是两次fresh whole-workspace invocation；manifest的`scope_note`也明确排除该解释。不能把分区结果描述为两次全新的`cargo test --workspace --all-targets --all-features --locked`。原`partitioned-workspace-gates-v1`失败材料保留在`results/partitioned-workspace-gates-v1/manifest.json`及`tests-021/seed-5350648285461741569/`：原失败是旧tight-budget pin要求52步而已存在的single guard路径实际计61步，v1记录exit101；后续只把pin校准到61，`full.produced`与`tight.text == full.text`断言不变。v1原始失败及v2修复后双seed的成功记录都未覆盖。

精确Clippy v1结果保存在`results/root-ci-exact-clippy-v1/result.json`（exit101）及同目录`stderr`（SHA256 `c2c27aa29dc0e211836231709ce31edee504c467130e6f31a2a4523d5da84cab`）。它只报告`tests/p3_constructor_primitive_conversion_arguments.rs`两处`clippy::unnecessary_filter_map`：UTF8常量池descriptor筛选在原归档行479和653。两处均已用等价`.filter(|entry| matches!(...))`保留匹配的descriptor条目，再collect引用/计数；当前源码这两处为`filter`。该Clippy失败记录原样保留。root复验`root-ci-exact-clippy-v2/result.json`实际exit0，命令逐项取自当前CI既有allow-list且保持`-D warnings`，未新增allow。MSRV首次`root-msrv-v1`因PATH上的Cargo不是rustup shim，不识别`+1.88.0`而exit101，未进行编译；原失败保留。改用明确`rustup run 1.88.0 cargo check --workspace --all-targets --locked`的`root-msrv-v2/result.json`实际exit0。

## Final Local Gates

root-java-execution-comparison-v1实际exit0，P3显式ignored三项全部通过，argv明确使用真实OpenJDK23 bin路径；root-functional-constructor-execution-v1实际exit0，完整class双JDK对照一项通过。root-final-fmt-v1、root-openspec-strict-v2及root-diff-check-v2均exit0。15opcode含ignored完整原语料、P5严格pins、指纹verify、reader census的已冻结成功记录及全部raw stream hash再核对，见results/root-local-acceptance-inputs-v1.json；init/report/build/Cargo.lock和冻结CLI二进制身份仍一致。3.2已完成，3.3仍待当前代码提交推送与该SHA实际CI。当地未安装JDK25，不声称本地执行了JDK25 oracle；该步骤须由当前提交的CI验收。

## Raw Stream Whitespace Check

首次root-staged-diff-check-v1实际exit2，只报告Cargo原始stdout的终尾空行；此前git diff检查不覆盖当时未跟踪的日志。产品提交a738a894已本地创建但尚未推送。本片已有results/.gitattributes只匹配*.stdout/*.stderr，而root runner以stdout/stderr无后缀保存流；沿用相邻片的原始证据策略，只为这两个basename补充-whitespace，不改任何日志字节、hash或源码规则。完整相对前置f9c6f56d的diff检查须在补充后通过再推送；不把首次staged检查冒称成功。

补充后root-full-diff-check-v3实际exit0，命令为git diff f9c6f56d69d8dc3d37da4584003af8e3c83b3453 --check，覆盖当前全部已提交产品/测试/fixture/证据及尚未提交的属性修正。原始streams未变化，旧失败保留。

## Exact Committed CI Acceptance — 2026-10-10

已推送产品a738a8941及属性文档修正8f624ea64cd9d8e73810cc3a2384c773fa7f87bd。确切8f624的CI37968418985四job、48steps全部success，含两个独立seed全仓命令、实际Temurin25 oracle、显式P3/functional-constructor完整Java对照、MSRV1.88、Clippy/fmt、supply chain/fuzz、OpenSpec strict和tracked-tree检查。原始gh JSON为results/ci-code-sha-v1.json（SHA 2b4a064bdc238ee5b97b28c6fd0d0c6196236dfabd95b8f80adec2d7a5e5cc54）。root独立核对实际状态、48steps及该commit四个产品/构建blob与冻结CLI，见results/ci-code-sha-root-verification-v1.json（SHA 9dfb2eb952ed9faf23273e76beced788a9ce59b5708daa867ebe6559a30fa682）。本片7/7，不能借本次CI验收未来产品改动。

本项目cargo clean实际删除7681files/894.1MiB，root/fuzz/14辅助树当前无target；辅助树detached、干净且main祖先，没有遗留分支待合并。最新身份/清理见root-clean-after-local-acceptance-v1与worktree-audit-after-local-acceptance-v2.json。主机其它项目未操作。

BigDecimal后续独立问题见results/next-bigdecimal-fact-audit-v1.md。历史原/JADX源码、类、原始双流及入口错误/重验记录已归档52files/126063bytes，manifest SHA 850427a2b6cca0d99fde694f79ce0828a939ebae6bcea30c6ce61ad454e80658；root核验228checks/0errors（bigdecimal-historical-provenance-root-verification-v2.json）。这是历史字节保全，不是fresh Java/JADX执行；旧0/18与当前16/18、legacy18/22分母分开记录。

下一独立change recover-covariant-child-array-initializers规划4/4、前置及历史基线2/7；仅解除child ownership类型相等早截断，Builder继续用准确store/type证据。协变子数组、BigDecimal、member/statement与alias边界保留，本片成功不等于EM18整单元追平。

文档收尾staged diff v1实际exit2：新归档JAR MANIFEST.MF的标准CRLF/终止空行及一份新写审计Markdown的额外EOF空行。仅为两份原始manifest路径添加-whitespace保全字节/hash；新写Markdown收掉额外EOF空行，不改原始流或任何类。旧失败结果保留，修正后须以v2实际检查通过。

修正后root-acceptance-staged-diff-v2实际exit0，归档manifest原始hash未改变；OpenSpec strict收尾327passed/0failed。
