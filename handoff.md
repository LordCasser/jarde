# Jarde 接续入口

持续目标 active，在当前聊天继续。按本地 JADX **71 语法单元 / 612 测试文件**账本先完成大颗粒 MVP，再探索双方未覆盖的形态。局部通过不增加整体单元完成数。

## 2026-10-18 当前：TestNestedLoops5 最小实现已通过定向验收（5/8）

`recover-proved-loop-arm-returns`，main基点9b1a9e87e，仍在当前聊天推进。finThrow5/5、nestedFin16/16等旧产品已交付，不重复实现。当前工作区未提交；完整对照、广门、两fresh seed和产品自身CI尚未关闭，不能称交付。

原始NL5两profile完整outer+child三方法、A/B双javac/debug-none完整类的旧Jarde/JADX基线均已root独立接受。原始28命令/56raw及旧矩阵42命令/84raw；补充65命令/130raw及旧矩阵97命令/194raw。旧Jarde原始8个javac23及A/B32编译均缺return；JADX原始4个javac23和A/B16编译成功，隔离运行同各自真实原oracle。NL5 check依赖major55框架，Java8边界单列。五基线归档729成员的baseline-archive-review-root.json保持冻结，供最终API引用。

root已集成并审查Luna最小候选：只改loop_arm_join_source，精确natural∪同Frame已证明return叶、原canonical唯一入口/普通出口、准确ForHeader和物理gateway、原Stop传播。implicit_tail_latch_origin保持755fd原样。完整实际region树确认原child16尾If/gateway34、A child30与B child31尾Straight/gateway96，十输入均有ForHeader，因此已删除CompoundIf扩展，不需要新机制。

`/private/tmp/jarde-loop-arm-focused-root-v8-20261018`实际fmt、479 Java库（含7新Reader内部门）、3公开、4旧循环臂控制、181 Reader全通过，共667通过出现/0失败；root-review.json已独立复核raw/3134来源/CLI/default stack/exact clean。CLI63070624B/SHA09dda40069dc362bc59f141a7d82c8cdcaee8b64072d304ec1ba848bdc14da25，仅候选。公开门核完整方法、BLAKE3身份、全物理BCI/UTF8、SourceMap-only/all文本map及late-budget无source/map。内部变形是明确同BCI proof-context，不冒称有效完整JVM/全SSA；真实jsr clone作异域context，异常边界用新合法catch-return类并以预算确认到达canonical扫描。

共享夹具22class/6Java/3manifest/3README，独立物理增量38Code/2handlers/74targets/0jsr，Reader实测1280/5677/626/3921/12。标准指纹已经原入口再生：旧2356逐项不变，仅新增本项31，总2387，其他字段不变。v8后仅内部Frame初始化风格修正；最新来源需由后续广门重新绑定。

广门v1因新测试5处field_reassign_with_default失败，raw和exact clean保留于results/gates-failed-root-v1。v2通过CI原样Clippy后，Homebrew cargo不支持+1.88.0入口，尚未编译MSRV项目；源码不变/target清空。v3绝对rustup入口的MSRV1.88及479库已通过，P5第五目标因未开all-features为0tests被runner正确拒绝；v4启用all-features八P5实际104/0/5，CLI build成功且与v8逐字同，root重核全部当前3134来源、raw/守卫/exact clean后接受组合本地门（results/gates-final-root-v4）。当前11归档879成员复开审计见all-archives-root-review-v1.json，后续新增包不含于此。

最终API私有prep冻结v5：`/private/tmp/jarde-loop-arm-api-capture-prep-luna-v5-20261018`，24请求/56方法配置，root已读collector/reviewer/helper并纠正归档关闭生命周期；尚未执行。最终36源格/72重编collector冻结v2：`/private/tmp/jarde-testnestedloops5-final-rebuild-prep-luna-v2-20261018`，root已全面审读，Luna另准备独立reviewer，尚未执行。workspace checker prep v1保留旧19关键门并新增本项10，总29；root已读diff与原逻辑。root独占共享编辑/Git/Cargo/JDK/JADX/CLI/网络，Luna仅私有/只读，不改冻结目录。

下一固定TestSequentialLoops仅基线已接受：`/private/tmp/jarde-testsequentialloops-baseline-run-root-v1-20261018`及独立review-root-v1，24命令/48raw、四VM六有限case同原oracle、major55/CodeSource负控；旧八target fallback、map7/25，ctor3/3。重编matrix prep尚未完全审读/执行，不先新增机制或记完成。持续71/612和CF11整体计数不变。

## 2026-10-18 当前：TestNestedLoops3 已完成产品验收（7/7）

finThrow5/5、nestedFin16/16、父finally7/7、exception-only11/11、直接数组构造7/7、switch7/7均已交付，勿重复实现。持续推进固定71/612账本；本片不增加CF-11或整体单元数。

产品 `755fd2c84ecbe204d03f0ff608d3a711a1bea494` 已推送main。`recover-prefixed-exceptional-retry-loops` 当前7/7，关闭后apply all_done、strict368/0已实际确认。自身[CI39080067969](https://github.com/LordCasser/jarde/actions/runs/39080067969)已root独立接受：4jobs/52步骤成功，两seed各383target/3726通过/0失败/98忽略、逐target相同，10新关键门每seed唯一通过；52实际命令/104raw及官方逐步骤ZIP已归档，gh的UNKNOWN STEP显示限制与首次checker拒绝保留。root独占共享编辑/Git/工具链，Luna只在私有目录准备与审阅。

实现复用Loop/Try/Frame，不新增公共AST或pass。局部复合证明区分outer header、inner retry header、protected、normal join、outer exit，只接受direct或一个纯latch bridge；核真实物理/canonical row、SSA/effects、完整owner入口/出口/异常/clone闭合及精确transfer multiset。root已修正handler边界None契约、catch完整Sequence、封闭If residual，并补齐扫描/索引/checkpoint计费。真实外来边、错行/clone、effectful bridge、catch→outer及预算/取消/期限均有对抗门；内部metamorphism不冒称合法JVM输入。

最终CLI `/private/tmp/jarde-prefixed-retry-final-focused-root-v5-20261018/candidate-cli`：63070096B，SHA256 `25adeaf312a7644c7d4c73191ef1264af0378cf78139d35a890377e3c86371ce`。fmt、CI同款Clippy、472 Java库、3公开、181 reader、标准指纹及regenerator通过；仅新增31指纹，旧2325项不变。MSRV1.88 workspace/all-targets/all-features通过。

最终24公开请求核176结构化方法配置、全部物理BCI、4344同方法来源引用/UTF8；default/all与152非目标投影稳定。整类matrix214命令/428raw：Jarde48完整原样编译/96双JVM/864case同原oracle；JADX22编译/44运行404case同原，另两javac8 fullbundle major55工具边界无runtime。root独立checker通过，另重解析70实际重编class；全部方法保留、未手修生成Java。旧fallback、缺返回、首次collector参数错误、临时实现拒绝与reader普查计数失败均独立保留，不混算成功。

两fresh workspace已独立接受：每seed383targets/101批/3726通过/0失败/98忽略，19关键门每seed唯一通过；动态inventory与逐target计数相同，5051来源逐git blob/字节/mode绑定755fd产品。默认栈、5GiB余量/target1GiB/每秒守卫、finally exact clean通过，target不存在。旧workspace collector把artifact顺序误当运行顺序，root保留原表并按stderr Running重建，见独立债务，不混入产品修改。

证据索引为本change `results/README.md`；最终16归档2055成员已root独立逐字复开。关闭文档提交使用[skip ci]，不得把755fd产品CI说成后续文档HEAD自身CI。最终实际HEAD/远端/clean与无target收据将保存到 `/private/tmp/jarde-prefixed-retry-final-head-root-v1-20261018/execution.json`，接续时先核真实Git状态。

下一项固定审计 `TestNestedLoops5` 已实际取得完整基线：`/private/tmp/jarde-testnestedloops5-baseline-run-root-v3-20261018`，28命令/56raw、八Jarde格三方法完整；testFor全fallback/jre_region_arms_do_not_meet(block9)，全部29物理BCI在解释map中有来源；constructor/check结构化。独立review已接受。原core四双VM运行三case均7，check两JVM23实测通过；check的真实JadxAssertions依赖major55，Java8加载边界另列。完整矩阵在 `/private/tmp/jarde-testnestedloops5-rebuild-matrix-run-root-v1-20261018`，42命令/84raw、24原样编译：Jarde8javac23缺返回/8javac8框架边界；JADX4javac23成功，8coreVM24case+4check23case同原，4check8边界。root独立matrix reviewer已通过，无语义不匹配；不能混算编译失败为运行成功。

尚未为NL5新增机制或改共享Rust。Luna私有诊断patch经root逐块审读，修正未执行草稿的branch offset@18及block9/terminal11 gate后，准备于 `/private/tmp/jarde-testnestedloops5-diagnostic-root-v4-20261018`；仍未apply/编译/执行。下一步实际首拒probe，记录branch9→prefix14→child header16的Frame/SSA/现有continuation拒绝点，再决定最小OpenSpec及Luna实现。两参数化有限嵌套for补充源/独立driver在 `/private/tmp/jarde-testnestedloops5-parameterized-probes-prep-luna-v1-20261018`，六case计划覆盖真实早返回/正常路径/同实例尾部效果；尚未编译或采oracle。原样本的早返回在常量范围不可达，不能仅以原case返回7证明早返回语义。

## 2026-10-18 最新接续：switch 已完成产品验收（7/7）

用户要求的 **finThrow 5/5、nestedFin 16/16、父 finally 7/7** 已完成并验收；同一 nested 最终 CLI 的完整对照为 nested 432 + finThrow 144 = 576 rebuilt case。历史产品自身 CI、原始/JADX 失败及当前复核见下方 finally 节及 `openspec/evidence/java-syntax-2026-10-18/finally-leaves-recheck-root-v1`，不冒称这些旧矩阵在当前 head 重跑。后续 exception-only 11/11、直接数组构造参数 7/7 也已交付。

当前产品 **cf857abaabc4fcff82689812226f9906b8d932b2** 已提交并推送 main，`recover-switch-case-nonlocal-exits` 已 **7/7**。其自身 [CI39072531393](https://github.com/LordCasser/jarde/actions/runs/39072531393)由root独立核验：实际4jobs/52API步骤全成功，两seed各382targets/3716通过/0失败/98忽略，逐target计数相同、27关键门每seed在精确target下唯一通过。74命令/148raw、69个git-show/current来源pin及四job完整日志保存在该change `results/own-product-ci-root-v1`，160归档成员逐字重开。后续文档关闭head不冒称自身跑过该产品CI。不要重复实现finThrow/nestedFin或switch。

root 已审查集成 Luna 候选：switch case return 复用原返回叶证明，private LoopTarget 携带同次证书；非局部 break/continue 和 mixed switch 复用原 switch_loop_join、Frame、LoopBreak/Continue/SwitchBreak。单 candidate 遍历核完整 arm closure、真实入口及 clone/异常归属，join68 与 continue71 分离；没有新增公共 AST、solver 或 pass。root 另修正 Closed arm 的既有 frame 选择，覆盖 SN goto57/60 的来源，声明 gate 未放宽。

最终 CLI **62908496B / SHA256 10a73d1ee30a3820a018a2461cc3c3b3028b02bf658588bc74c4ec3ba2eb1620**，在 `/private/tmp/jarde-switch-final-focused-root-v5-20261018/candidate-cli`（与最终 broad gates 构建逐字相同）。已独立接受并归档在该 change `results/`：

- `final-focused-root-v5`：668 passing appearances；465 Java 库、181 reader、九返回叶/标签/停止、12 既有 switch 边界、一公开 request late-budget。真实外来入口、错/缺/歧义 return 证书、异常边和 jsr clone 绑定均有负例。reader census 1238/5495/606/3641/12 与独立 Code 计数一致。
- `final-api-root-v2`：16 实际公开请求、80 方法配置完整身份/顺序；六目标方法的 24 配置由 fallback 变 structured，700 个物理 BCI 全覆盖，2152 UTF8 map 区间/3024 来源引用合法；其余 56 方法投影同旧 605 CLI，SM 五控制形态不变，default/all 全文/map 相同。
- `final-matrix-root-v2`：225 命令/450 raw；32 次完整 Jarde 源码原样重编、64 次隔离 driver 运行共 1744 case、64 次 main 全等原 oracle。javac8 source/target8 与 javac23 --release8，双 JVM `-Xverify:all`、CodeSource 与 classpath 隔离。JADX SL 四次编译失败、SN/LD 各八次 driver 在 ledgerBreak 失败、LD 八次 main 仍为9而原为10，均原样保留，未执行 case 不计。
- `final-gates-root-v3`：fmt、CI 同款 Clippy、MSRV 1.88.0、reader、指纹再生/检查、P5、CLI 共 286/0/5；常规八 P5 104/0/5。仅指纹新增20项、原2305项不变，未改 P5 旧预算/计量表。exact clean，target absent。
- `final-strict-root-v2`：OpenSpec 1.11.0 strict 367/0；关闭后的 `closing-strict-root-v1`再次实际确认 apply all_done/7-of-7、strict367/0，六raw pins通过。原始 driver 另有 436 case，只属于原 class oracle，不混作 rebuilt 验收。

两轮 native fresh seed 均已 root 独立接受：每轮382 targets/100批/3716通过/0失败/98忽略，412 raw复核；5016来源、inventory及每target计数相同，九新门每seed各一次通过。最终CLI3063构建来源与workspace来源的5023文件并集按字节/mode绑定cf857产品，默认stack/5GiB/target1GiB/每秒守卫/finally exact clean均通过，target不存在。证据见该change `results/workspace-acceptance-root-v1`，434归档成员已逐字重开。无活动Cargo。文档关闭/推送后的最终实际HEAD审计保存在 `/private/tmp/jarde-switch-final-head-root-v1-20261018/execution.json`：接续先核该终态收据和当前Git，要求HEAD==origin/main且远端同SHA、clean、仅main、14辅助checkout detached/clean/main ancestor、target不存在、5023来源仍同产品。历史检查点不能替代未来HEAD检查。

早期 c6e CLI 的语义矩阵虽通过，后来发现 SN goto57/60 map 缺口，不能冒称最终来源验收。早期 format recorder、测试编译、census、jsr 整 loop 假设失败均完整保留，详情见 `results/README.md`。jsr fixture 在 NormalFlowView 未暴露自然 loop，因此当前只验证真实 clone completion edge 的同身份正例/异 clone 负例，不冒称整历史 loop 恢复；独立债务见 `separate-architecture-debt.md`。显式 finally return/throw、普通 self-new 匿名投影及旧 fallback StringBuilder map 问题继续单列，不混入本产品。

下一项先做固定账本 CF-11 的原始 TestNestedLoops3 完整基线；root静态审查已保存到 `openspec/evidence/java-syntax-2026-10-18/cf11-next-baseline-root-v1`。当前只确认尚缺该原始完整类的验收，不声称Jarde已失败；先原/JADX/Jarde整类重编、运行、全BCI/身份，再决定是否需要实现。TestNestedLoops5留作独立后续，不优先追finally显式完成或jsr自然loop探索。

持续 goal active；本 leaf 不增加 71/612 总体计数。

## 2026-10-18 当前：原始 TestFinally3 与直接数组构造参数

`recover-direct-array-constructor-arguments` 已7/7，产品81b0684d579a09e16f328f2840bcd2400338219c已推送并通过自身CI。首产品550b7ff922e32fa4ce5d4326d8c7d22bd8fa30a3的完整seed1曾发现P5重复遍历计量失败，root取消旧CI39060504198，原失败/取消日志均保留，不能当作成功。Luna补丁经root审查，仅init.rs复用已读取operands并将物理参数次数并入原有已计费循环，删除重复工作，未改P5旧表值或预算。原finThrow/nestedFin/exception-only验收与旧产品CI见下面历史区。本轮复用完整数组证明，精确匹配constructor物理final ValueId，替换String窄特判，闭合后置实参reader且不吸收sibling所有权；同owner调用按receiver SSA选取，无新solver/pass/carrier。

最终CLI62815152B / SHA605fdd1b2b0cc077bb9440a02b7f995ca33dded37b7527cc306f65a77bdf4c0c，`/private/tmp/jarde-direct-array-final-gates-root-v5-20261018/candidate-cli`。v5共470通过出现（461库、P5 bulk5、四公共门）/0失败/1忽略，fmt/CI同款Clippy/MSRV1.88通过、来源不变、exact clean。40实际请求重放核完整源码/非usage报告恒同已验收矩阵，仅ir_items/elapsed_millis变化；沿用相同源码的真实JDK编译运行证据，不冒称本次重新执行JDK矩阵。此前v4的181reader、指纹再生/检查门保留；指纹仅新增30项、原2275项不变，reader实测1227/5446/602/3502/10。

原始TestFinally3 debug/noDebug：100方法配置全BCI/物理身份/UTF8；20原样整类重编、40隔离运行320case等原oracle，88非目标配置不变；JADX6编译均两处inputStream未初始化错误、无runtime。独立完整数组家族88方法全BCI，8重编/16运行224case，JADX112case通过；sameowner8重编16case且class Complete。两种合法重排控制经真实双JVM验证，Jarde保持拒绝，无新增排序机制；其中JADX反序4编译成功但8运行全失败，trace21而原12，真实失败保留。

独立债务：ReverseArrayOrder自然形态三个方法均正确，但唯一普通self-new被误调匿名投影，class Partial/CLI4；不要混入数组实现。通用fallback StringBuilder head53 presented但map缺失，旧7287/新e01双输入与default/all实测均存在；public控制仅核断言前数组来源，完整正类仍全BCI。详见本change `results/README.md`、`separate-architecture-debt.md`及各capture/root-review。所有早期编译/测试/采集失败均保留，不冒称通过。

修正产品自身[CI39062097681](https://github.com/LordCasser/jarde/actions/runs/39062097681)已root接受：4jobs/52步骤全成功，两seed各381targets/3707通过/0失败/98忽略、逐target计数恒同、18关键门每seed各一次通过；4job raw/产品及26个class pins见本change results/own-product-ci-root-v1，后续文档head不冒称自身跑过该CI。八all-feature P5目标104/0/5，旧表不变。本地两fresh seed均381targets/100批/3707/0/98，412raw、逐target计数及8关键门通过，默认栈/守卫/exact clean、target不存在，构建3037与每轮4990来源绑定产品，见workspace-acceptance-root-v3。关闭文档/主线检查见final-delivery-root-v1，最终HEAD外置收据 `/private/tmp/jarde-direct-array-final-head-root-v1-20261018/execution.json`。持续goal active，71/612不变。

接续范围备忘见 `openspec/evidence/java-syntax-2026-10-18/explicit-finally-next-analysis-root-v1` 与 `explicit-finally-primary-test-audit-root-v1`。root独立核CF16的24主归属文件：22文件/24个Java finally体无显式return/throw，Luna草稿28计数已更正；该结论不外推612或JADX产品能力。原七方法中的finallyReturns/finallyThrows仍为单列债务，先按既有JADX用例完成差距，再为确有必要的探索取真实SSA/首拒和原JVM覆盖优先级证据，尚无新机制或实现任务。

## 2026-10-18 接续：仅异常完成 finally 已完成产品验收

`recover-exception-only-finally-completion` 已11/11，实现和对照已验收，产品 **3fa506df0ccd7da3d38ab769bfd65235817508fe** 已推送。其自身[CI39051492999](https://github.com/LordCasser/jarde/actions/runs/39051492999)实际4jobs/52API步骤全通过：两seed各378targets、3700通过/0失败/98忽略，逐target计数相同，9新关键测试各seed一次通过。完整API/四job日志、collector与独立checker见本change的results/own-product-ci-root-v1；后续文档head不冒称自身运行过该产品CI。

root审查集成Luna的最小实现：复用FinallyCompletion::Throws、Shape::Finally.cleanup和既有Try AST；完整checkpoint覆盖declare/body/cleanup/push及undeclared拒绝；只放行已消去、unused且每输入同一Entry参数的冗余handler Local phi，不增solver。真实SSA首拒、旧phi基数拒绝及Builder测试组装失败均保留，不降断言凑绿。

最终CLI SHA **7287e856c5ce256c127fc6b33bcc3c663b058e913bf50de5d82ed9c9373b95c8**，62788432B，构建收据 `/private/tmp/jarde-exception-only-final-focused-root-v3-20261018/execution.json`。3002构建来源和4955native来源逐文件绑定产品。本地两个fresh seed均378targets/99批/3700通过/0失败/98忽略，来源/inventory/逐target计数同，9关键各一次，408raw复核，默认栈/5GiB/1GiB/每秒守卫/最终exact clean通过，target不存在。最终focused649 passing appearances（458库、181reader、2公共正例、2停止、1指纹再生、5指纹检查），0失败/1忽略；CI同款fmt/Clippy和MSRV1.88全workspace/all-targets check通过。标准指纹仅新增19项，原2256项不变，reader实测1201/5368/598/3496/10。

final-matrix-root-v2：75命令/150raw、44方法配置核全物理身份/BCI/UTF8，8次完整独立类原样重编、16运行144case等4原oracle36case；对象identity、try/cleanup异常覆盖和78一次通过。入口参数另8运行24case等4原oracle12case，null传播NPE、非null保持同一对象。完整七方法仅tryThrows改善，finallyReturns/finallyThrows仍有两处missing return，八次sealed整类编译红项保留，无sealed runtime。JADX1.5.6主矩阵四编译成功、八运行首case漏mark7得8；参数矩阵两编译成功、四运行首case同样失败，未冒称后续case已执行。

neighbor-public-root-v2七完整拒绝类184方法配置，以及existing-finally-public-root-v2旧15class236方法配置，正文/map/quality/fallback同旧4811CLI。这是完整来源对照，旧产品的576runtime仍只属于旧产品。归档及实际范围见results/README.md与各root-review；最终两版strict各366/0、产品来源恒同，交付收据见results/final-delivery-root-v1。

持续goal active，本leaf不增加71/612。下一项按固定JADX账本核TestFinally3原始debug/noDebug profile；当前Tf3是转写样本，JADX build中的原nested产物缺源码/编译参数收据，须新冻结原输入后再取Facts/首拒、决定是否需要实现。Smali单独记账；finally自身显式return/throw继续作为独立欠账。原finThrow5/5、nestedFin16/16、父finally7/7已验收，历史状态见下节。

## 2026-10-17 当前推进：finThrow 与 nestedFin 已通过产品验收

先从 GitHub 快进64个提交到 e5757b5a9920da5d2b86dc44d8c729c98caf1253，复核新交接与父 finally 真实4/7状态；旧本地 facade 草稿仅备份/stash，未应用。

当前 `recover-branch-throw-return-finally` 为5/5：finThrow 的完整证明原本已有，缺口是纯比较入口无异常边，直到 throw 子分支才调用 guard。最小修复在单 catch-all row 起点、范围内 comparison、双正常 successor 入口提前调用既有 examine；不新增 proof/公开表示/Builder 机制。root已接受实际红基线、450 库单测/2 API测试、全 reader census、指纹与CI同款Clippy，以及最终 CLI 的4 profile/8次完整源码重编/16次隔离运行/144case 对照。JADX1.5.6 对同输入虽可编译，但清理抛异常会打印两次ft，真实失败原样保留。

产品已提交推送6b32c708d1adea06558478fd0f339cc23885c581；strict365/0、primitive finally旧预期更新后的P5 19/0/1、双fresh seed各375targets/98batches/3686通过/0失败/98忽略均已独立接受。该产品自身[CI39037102727](https://github.com/LordCasser/jarde/actions/runs/39037102727)已由root独立接受：4jobs/52steps，两seed各375targets/3686通过/0失败/98忽略，9项关键门均通过；关闭文档后另核实际clean主线，文档head不冒称自身跑过产品CI。证据均在该change的results及root-review.json；该finThrow单独交付阶段父 `recover-return-through-effectful-finally` 为4/7；最终组合验收状态见下文，71/612不增加。

`recover-return-through-nested-finally-cleanup` 的实现产品 **4c6852ac8c4ab9ce88f0716f4d2bd702e15684c5** 已提交推送。root按OpenSpec取实际六块/四行Facts及全部generic首拒后派发Luna，保留公开Finally/SavedReturn与Try AST，私有Plan只存两个呈现区间并一次拥有六块；root修正SSA local/stack身份链、核四份out副本和全部异常归属、Builder完整回滚。15真实canonical邻变体拒绝并通过双JVM30次验证，late预算/取消/期限不发布部分正文/映射，453库/2API与当前CI Clippy通过。原编译与首拒失败raw保留。

同一冻结CLI **4811eea2d7f52d4c1b818c86acc1d961c91e3d7fe9d5adaa2c53131f84140f8e** 完成nested432及finThrow144个rebuilt case，共576；完整封存EX六方法保留、4次原样整类重编和8次nested driver运行通过，20个方法配置全部BCI/映射/身份接受，非nested文本同旧基线，finReturn仍fallback。FY/FX/FO父矩阵6完整类32方法同基线，6原样编译/12 rebuilt-only main等固定原oracle。FinallyCompletion完整七方法四配置保持三fallback，8次原样编译仍各三处missing return，无runtime；没有删方法凑绿。

产品自身[CI39042611463](https://github.com/LordCasser/jarde/actions/runs/39042611463)已独立接受：4jobs/52steps，两seed各376targets、3691通过/0失败/98忽略，14关键门各一次通过；两个native seed也各376targets/98批/3691/0/98，4933来源/inventory/每target计数相同，404raw接受，默认栈/5GiB/1GiB/每秒守卫及最终exact clean，target不存在。父29本地门131 passing appearances/127不同FQN、双strict365/0、本地MSRV1.88通过，两轮实际运行及最初MSRV启动失败分别保留。证据全部在该nested change的results索引与root-review，产品CI不冒称后续文档head自身CI。叶片16/16、父7/7，更新计划后OpenSpec1.11.0/1.14.0 strict均365/0。实现验收完成，尚未同步/移动归档；后续提交与clean-main收据见nested结果的final-delivery-root-v1。

JADX1.5.6的nested两次cleanup导致IOIO真实失败、父FX两次undefined-th编译失败均保留，不把它当正确oracle。下一独立欠账是 **FinallyCompletion.tryThrowsFinallyRuns()I**：没有正常return，不能冒用Void；只有exceptional cleanup而无normal copy，需先做实际Facts/SSA/effects/guard首拒probe后再定最小机制。普通cleanup调用抛错必须覆盖pending primary，是正向语义；finally自身显式return/throw继续范围外。静态备忘在父results/exception-only-next-analysis-root-v1，未运行probe、未实现、不计完成。

以下10月15日内容为历史记录，当前状态优先看本节与对应change实际任务、raw、源码/工具pin、自身CI。

## 2026-10-15 class-view 歧义修复已交付并归档

产品e8c749252a71a857838d0cb854f5a6d80a5c2704已进入main，自身[CI38679565191](https://github.com/LordCasser/jarde/actions/runs/38679565191)由根802项/独立1462项核查接受：4jobs52steps，两seed各365suites、3589/0/98，79具名门及既有额外JDK/examples/层级/fuzz门通过。修复保留foreign-owner优先级，重复Name及listed identity都在body准入前按完整候选返回歧义。既有76/0/2本地门、九组27库请求及三CLI请求保持原范围。

[自身CI与交付证据](openspec/changes/archive/2026-10-15-reject-ambiguous-class-view-body-selection/results/root-own-ci-v1/README.md)逐成员复开；4900测试来源同primary字节。4/4完成，analysis-contracts新增四scenario并归档。产品CI头与后续文档头区分；不增加CF11/CF18/71/612总数。

接口停止修复在独立AUX已有cc305fe6a本地检查点，813/0/1及八组完整源对照通过，尚待最终组合产品workspace/自身CI；snapshot/double-brace保持独立规划，当前冻结接口CLI已复现原H1 ClassHeaders8/15仍Complete，未冒称修复通过。

## 2026-10-15 四工程联合产品已接受

产品 **d3e2b2457338b61660b842f5705b225bba08a73a** 包含共享facts容量、no-Code重复方法选择、canonical真实保护范围及emitter停止分类四项修复，四项均4/4；已同步四份主规格并归档。自身[CI38673548915](https://github.com/LordCasser/jarde/actions/runs/38673548915)4jobs/52steps成功，两seed各3587通过/0失败/98忽略，77具名门一次通过；native Rust1.99/Temurin25.0.4+7，strict367/0。另实际九额外编译执行测试、两examples及12依赖层配置通过。 同步归档后的两版本strict均363/0；旧reuse-selected-class-read已补齐新增容量场景，首次362/1失败raw保留。

[联合证据](openspec/changes/archive/2026-10-15-preserve-emitter-output-stop-reasons/results/root-joint-integration-v1/README.md)保存1771 payload/12721464B/SHA f723dfa3e3e8bf3f91ed194453401464bfef295d474e20983abc0a5f91330073，1772成员由根逐字复开；focused-v3为462/0/2，fresh workspace-v3为7174/0/196、730suites、195命令/390raw。完整matrix-v2的16 Jarde单元全部重新编译运行，45/33行oracle同原，JADX八baseline编译失败/no-runtime、八admission成功。d3重建CLI与实际matrix所用c814 CLI同59602720B及SHA cdf06694fafcb4e8cef5a4b9452676489e8cf02495c0254ce38a21e44ee4c7d5，按字节恒同复用，不冒称d3重跑。早期canonical/emitter/c814 CI、workspace-v1/v2失败与真实计费修正全部保留。

历史运行来源闭包为4887项；额外13个fuzz遍历输入只按audit-time sidecar记录，未倒填历史运行开始来源。实际P5测试和标准再生成均核全2219corpus项。最终文档/main HEAD须与产品来源恒同、clean和target absent另核，不将产品CI说成文档提交自身CI。持续goal active，不增加CF11/CF18/71/612整体完成数。

7d0检查点的下一批三个计划当时仍 **0/27**：`reject-ambiguous-class-view-body-selection` 0/4、`preserve-interface-super-proof-stops` 0/12、`preserve-class-source-dependency-proof-stops` 0/11。先仅复制各manifest覆盖的change子树，再逐项取status/apply上下文，旧生产实际复现后才修复；私有静态候选未编译、未运行，不能计作实现完成。根串行独占Git/Cargo/rustfmt/JDK/JADX/CLI，子agent只在隔离cache准备与审阅，不操作其他聊天checkout。

## 2026-10-15 普通 try 交付与四工程早期状态（历史）

普通 try List 泛型及整数carry的修复产品 **644882d6db69f7d5b0bc1165c035a12b4f7cd8aa** 已提交推送main，自身CI **38665083468**由根实际742项复核接受：4jobs/52steps成功，两seed各365suites、3565/0/98，native Rust1.99.0/Temurin25.0.4+7和strict367/0。旧f014可编译错码反例保留；新实现先规划整数保存点再处理receiver依赖，保存失败恢复原primary、整方法闭合及原Stop。[自身CI与clean-primary证据](openspec/changes/recover-ordinary-try-list-generic-bodies/results/own-ci-644882-and-clean-main-root-v1/README.md)329payload已逐一复开。primary源/remote/target检查通过；关闭文档后的最终实际HEAD审计将保存于 `/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/primary-doc-closure-final-head-v1.json`。持续goal active。

最新本地证据已root接受并逐成员复开：[final-local-delivery-root-v4](openspec/changes/recover-ordinary-try-list-generic-bodies/results/final-local-delivery-root-v4/README.md)，3536 payload/23673727B/SHA c5ba3c8f273493df9cdcce092fd7de49a83df77b37b47678797552c022c8d95f。十个final-source逐字绑定focused/workspace编译来源及当前文件，4886完整来源相等；补充多维数组原Java/Runner另由supplemental pins覆盖。最终CLI 59590432B/SHA cd5e52dd45234b45e026af0117dae5f77bf3ef2a1316181bbbaf8c672c95679d，v17/v18/v19逐字同。

focused-v19 642/0/4，421Java库/178reader/原List及ICN控制、三个保存失败出口及真实prepare_deferred_bindings入口预算/ledger取消均通过。工厂/this/parameter完整十方法正例四cells/40maps/六行oracle同原；两方法receiver赋值负例四cells保留原primary/ExplanationOnly与全部13物理BCI，正确原Runner和helper classpath下四次javac仅因jarde_refused_body失败/no-runtime。早期collector未使用Runner副本错误单独保留，实际compile-v1未传该文件，v2显式使用正确Runner。

原baseline十七方法45行、admission十三方法33行的matrix-v4完整16Jarde cells全通过；JADX真实八baseline失败/八admission成功保留。forward-order-v4完整16方法四cells/64maps/12行及BI/ICN二十render旧产品全文/质量/map/身份不变。下游多维数组猜测经完整七方法四cells/28maps/两行运行证伪，已有NewArray支持，未改solver。

fresh workspace-v10已完成并独立root acceptance：195命令/390raw，两个seed各365targets/96batches，共7130/0/196、730 suites；required named gates逐一关联。ci-extra-forward-v3九测试及两examples、ci-tree-forward-v2依赖边界通过，全部exact finally clean/target absent。原workspace-v9第65批P5 IR固定费+6失败保留；原recorder六case与187请求direct/shared实际重测后只更新三exactpins+注释，旧所有shape/text/outcome/其它维度断言保留。focused19相对18仅该测试源变，CLI逐字同。早期v15/v16、audit环境/content错误及v9缺脚本launcher失败均分开保留。

forward change已10/10，ordinary已6/6，strict-artifact-hygiene已4/4。局部交付完成，不增加CF11/CF18/71/612总体计数；历史失败及冻结成功各按原scope保留。后续规格同步/归档随共同集成处理，保持当前collector与历史证据可用。

四工程修复已在本聊天附着的 `/home/lordcasser/.codex/worktrees/core-integrity/jarde` 独立实施提交，当前集成代码 **ad8eb3d397529775722056cf505da2a63e7e6dc1**：cache `8a56a863a` 本地243/0/1；no-Code method `de386896b` 49/0/0；canonical `d6fba33aa` 接受144功能+5计费的组合scope（旧P5失败保留、真实费用+68IR/+335Steps）；emitter `ad8eb3d39` 旧生产根对照0/3、修后440/0/0。四项各自CI运行中，主线尚未包含；最终CLI继续双JDK完整矩阵及共同workspace。aux为4887来源、reader181/Java424，严格clean；持久证据位于 `core-integrity-20261015`，topic focused不代替main交付。

review私有目录另外交付两个独立OpenSpec计划与真实公共反例：class-view PhysicalMethodId重复成员的无Code早返，以及interface-super proof在DependencyDepth=0时吞真实Stop并报Complete。后续按manifest仅复制change子树，独立实施；不复制其组合生产源码或未修复负例。可选54行prepass-stop仅测试增强不阻当前交付，优先已证缺陷。root独占共享Git/Cargo/rustfmt/JDK/CLI，保留其他聊天worktree。

## 上一批主线（历史）

2026-10-13：产品 **597010a8cb9992eead8276b2f2a67186ada393d9** 已提交推送 main；[自身 CI 38321041720](https://github.com/LordCasser/jarde/actions/runs/38321041720) 已由 root 独立接受，4 jobs / 52 steps 全成功。两 seed 各 365 targets / 96 batches / 3547 passed / 0 failed / 98 ignored，4860 来源与本地及产品一致。[CI 与主线检查点归档](openspec/changes/recover-debug-reference-lifetimes/results/own-product-ci-597010-and-clean-checkpoint-root-v1/README.md)的30成员已由root逐成员复开，18捕获raw/gzip流恒同；检查点43命令只有bytes/SHA记录，不声称保留raw。

已接受 clean-main 检查点 **bcf426e5ef4d2c49c4f1d519c5ef974393b6bf98**：仅 main，15 worktrees，其中14辅助checkout detached、clean、main ancestor，无target，4860来源字节及mode恒同。关闭文档提交后root再审计实际最终HEAD，结果固定外置 `/private/tmp/jarde-reference-prefix-clean-main-final-root-v1/execution.json`；接续必须核该结果状态及head，不以检查点证明未来提交。

## 本批已验收

[引用生命周期](openspec/changes/recover-debug-reference-lifetimes/tasks.md)与[普通try前缀循环](openspec/changes/recover-protected-prefix-loop-sequences/tasks.md)均 **6/6**。前者把LVT范围绑定实际引用身份，保留alias/CFG门；后者复用精确named row贯通prefix与foreach异常归属，完整Sequence留在Try.body，只有已证range-end纯bridge成为兄弟continuation。未新增solver/pass或公共carrier。

- [联合整类证据](openspec/changes/recover-debug-reference-lifetimes/results/joint-fullclass-candidate-root-v1/README.md)：双JDK/debug-none/default-all，8条Jarde整类运行各17场景同原；另8条运行各10异常边界场景同原，390成员/164raw已复开。JADX16条整类编译失败如实记no-runtime。
- [最终focused-v11](openspec/changes/recover-debug-reference-lifetimes/results/final-focused-root-v11/README.md)：680/0/3，14具名门，fmt/当前CI Clippy/strict360/fresh CLI/finally exact clean均通过，4860来源不变。
- [fresh完整工作区](openspec/changes/recover-debug-reference-lifetimes/results/full-workspace-accepted-root-v3/README.md)：195命令/390raw，两seed合计7094/0/196，默认stack及资源守卫通过，finally exact clean、target absent。

冻结产品CLI `/private/tmp/jarde-reference-prefix-final-focused-run-root-v11/candidate-cli`，62562992B / 0555 / SHA `fe00fa7f0edc587a4e85bcc1136bd4397738c2a9df632432a2e79a8221c8bdd6`。

旧workspace-v1因过时同名LVT失败要求终止，v2因4个新夹具未登记指纹终止；原失败分别保留在[同名对照与失败归档](openspec/changes/recover-debug-reference-lifetimes/results/same-name-fullclass-and-workspace-failure-root-v1/README.md)和[指纹失败归档](openspec/changes/recover-debug-reference-lifetimes/results/workspace-v2-fingerprint-failure-root-v1/README.md)。这些失败不拼接为成功。上一批循环catch8/8、字段拼接6/6、Structured List9/9的历史交付见[完成审查](openspec/completion-review.md)，旧证据不替代新产品验收。CF11 / CF18 / 71 / 612 计数保持账本原值。

## 上一批规划（历史）：普通 try 内 List 泛型

[recover-ordinary-try-list-generic-bodies](openspec/changes/recover-ordinary-try-list-generic-bodies/tasks.md) **1/6**，spec-driven ready，root已读全部apply上下文。复用同次Try/canonical/code的精确row与owner闭合，打开现有完整void用途扫描的handler交集，逐指令核Code/operands/SSA/effects/Operations，并包含catch用途。统一处理现有精确E，至少String/Integer分支与null、remove(Object)前缀、nestedforeach；unknowncall/rawalias/wildcard/重赋值/indexOverload继续按既有证明拒绝。不要只删handler早拒或加通用solver。

[44行完整基线](openspec/changes/recover-ordinary-try-list-generic-bodies/results/baseline-product-597010-root-v1/README.md)已接受：17物理方法、双JDK/debug-none各27命令/54raw，8条Jarde整类运行同原，8条JADX编译失败no-runtime。[45行补充基线](openspec/changes/recover-ordinary-try-list-generic-bodies/results/baseline-45-supplement-product-597010-root-v1/README.md)已root执行auditor-v4接受并逐成员复开251成员/118raw绑定；旧44行逐字作为前缀，新增可变output的handled正常完成，effects=2。五目标方法当前仍raw泛型，不能把基线通过算作新实现完成。

Luna正在私有目录准备report/region候选与永久正反测试，另一Luna准备公开integration/改名/catch用途fixture。root审读后才应用共享源码并串行编译。实际候选须重跑至少45行完整类、异常注入、default/all、Stop和最终交付门。[JADX算法与架构分析](openspec/evidence/java-syntax-2026-10-13/try-list-signature-next-mvp-analysis-root-v1/README.md)供参考；按原class契约验收，不照搬JADX rawalias错码。

## 边界与资源

root独占共享Git/Cargo/rustfmt/JDK/JADX/CLI；Luna私有准备或只读，不创建分支/worktree。用户授权提交推送、主线清理和持续推进。

构建前重新核可用 **5 GiB**、target上限 **1 GiB**、**1秒进程组守卫**、默认Rust stack，finally exact cargo clean。不得并行root工具链或清活动target。ScriptOS此前已结束产物累计释放2.35GiB，独有验收材料保留；该数字属于上一批主机；当前每次持久receipt记录实际free_bytes，不能沿用历史磁盘余量。其他任务仍使用磁盘。

[引用definition-order债务](openspec/evidence/java-syntax-2026-10-13/reference-definition-order-audit.md)仅静态推断，尚无实际错码反例；匿名类super前capture编译差距、DF.loopFinally及纯goto文本映射缺口独立保留，不混入当前List改动。证据质量须区分source、物理身份、结构恢复、声明投影与完整运行；禁止删方法、手修生成Java或降低断言。
