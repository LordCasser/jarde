# switch 非局部出口实际证据

当前 **7/7，产品cf857abaabc4fcff82689812226f9906b8d932b2已推送并通过自身CI39072531393**；最终矩阵、本地双fresh seed、CI原始日志与关闭strict均已root独立接受，详见本文件末尾的“最终产品验收：7/7”。最终Git主线检查见handoff所列外置终态收据。

下文按执行顺序保留历史检查点；2/7、6/7及pending描述仅属于对应阶段，不覆盖最终验收状态。失败raw与旧候选来源限制继续保留。

## 2026-10-18 旧产品红基线（历史）

`baseline-root-v1` 使用已验收产品81b0684d5的605 CLI，新编译原封不动的SN/SM/SL/LD：四完整源×javac8/23，八class均major52；双JVM共16原始oracle。137命令/274原始流逐份核字节与SHA，工具/输入/源码前后不变，target不存在。完整源码从Jarde JSON或JADX输出原样重编，未删除或修补任何方法。

- Jarde共16配置；SM八次完整类重编、16次隔离运行匹配。SN/SL/LD共24次完整重编失败，无rebuilt运行。执行complete/CLI0表示请求完成，不是源码质量通过；所有六个目标方法均有明确fallback/explanation-only标记。
- SN两个case-return的报告原因是uncovered blocks；SN.labBreak、SL.labeledBreak与LD.lab是switch shape/loop shape/uncovered；SL.mixed是switch arms overlap/loop shape/uncovered。同一local1诊断不能证明相同首拒。
- default/all的八对完整类正文、方法正文/quality/fallback/source map相同；all额外展开evidence/region/rule，非usage整报告不要求相等。40个default方法配置为28 structured/12 fallback；all对应相同，共80方法配置。独立检查了668个default map区间、1120个同类origin BCI引用；这里只证明引用合法及范围合法，不宣称拒绝方法覆盖全部语义指令。
- JADX1.5.6的SN/SM共八次完整重编、16次运行匹配；SL四次编译均因不可达break失败，无runtime；LD四次编译成功，八次运行均输出9，原始双编译器/双JVM均为10。普通SN main未暴露此错，LD判别输入不可省略。

root接受的是当前红基线，不是实现完成。采集器、独立接受器、两份复核结果及所有实际输出均在归档。

## 同次IR首拒已确认

`first-refusal-root-v1`实际应用Luna私有诊断补丁，守卫构建39.34s/峰值372076204B、exact clean；八all请求的整个非usage JSON同605基线。40方法配置按完整stdout方法顺序和canonical Code区间配对；26raw pin复核，源码SHA精确恢复，完整产品来源同基线，target不存在。SN返回44未入loop scope、label break无局部join准入、SL重复尾71与后置声明拒绝已独立复核；未打印的SL forward-join内部条件不推断，见instrumentation.md及归档独立memo。

design已按这些事实补齐并派发两个Luna私有实现：返回叶片及switch join/出口消费。当前2/7，仅取证关闭，产品实现和对照未验收，不按规划或私有补丁计完成。已有LoopBreak/Continue/SwitchBreak和Return足以表示，暂不新增公共表示或solver。

## 原始语义 oracle 与 case-return MVP 阶段

`original-driver-root-v1`：root 用完整四份原始源码重新编译八个 class，字节与红基线完全一致。独立反射 driver 先检查目标 CodeSource 和恰好两项的隔离 classpath，再执行 SN 37、SM 42、SL 20、LD 10 个输入；16 次双 JVM 运行共 436 个用例调用，完整 stdout 与固定 oracle 相同。25 命令/50 raw，输入与源码在采集期间不变。之后 root 接入 return MVP 导致当前 checkout 与这个历史快照不同，不能误称采集期间漂移。本阶段没有 Jarde/JADX rebuilt 验证。

`return-fmt-root-v1`、`return-focused-root-v1`：root 实际格式化指定两份 Rust 源码，随后守卫执行双编译器 SN case-return 正例测试（1 项）、旧 CF07 回归（5 项）及 CLI 构建。6 个测试通过，默认栈、源码在构建期间不变、最终 exact clean，target 不存在。候选 CLI SHA256 为 `5c3e9a1980a4b102edbda8b3e1f53239f19928aceea6aa6f32c005aa077ae082`；归档保留实际构建源码快照和 CLI pin，不包含可执行文件。

`return-api-root-v1`：该 CLI 对八个完整原 class 实际执行 16 个 default/all 请求，32 raw、22 个断言通过。80 个方法配置的声明顺序、flags、完整 owner 身份已核；1640 个 UTF-8 map 区间及 2520 个同类 origin BCI 引用合法。只有 SN 两个目标 return 方法的 8 个配置改变 text/report，其余 72 个方法配置的稳定投影与 605 基线一致。default/all 完整正文与 map 一致。这里未重编完整生成源码，也未完成负边界、晚停止或 break/mixed 组合验收，任务仍为 2/7。

四份 capture 均由 root 重开全部 tar 成员并核对字节；对应 `capture-review.json` 记录 SHA 和成员数。后续组合源码或结果不得覆盖这些 MVP 阶段凭据。

## 首次组合实现：完整源码矩阵已接受，最终产品门待完成

`combined-focused-root-v1`：root集成Luna join v4，保留单candidate遍历，复用已有return证书及loop transfer表示，修正入边检查的停止位置为终端switch BCI。461库、7返回叶、8fallthrough、2boundary、2scope共480测试通过，CLI构建成功；默认栈、3051来源不变、峰值495170620B、最终exact clean。CLI 62908496B / SHA `c6ea7b9159be9a358de2e3d8c64506781a1a8d112103de0199e459e4413a6869`，实际构建源码保留在归档。

`combined-api-root-v1`：该CLI实际16请求，六目标方法的24配置全部由fallback变structured；其余56稳定投影与605红基线一致，含SM全部控制。80方法的完整身份、flags、声明顺序、2144 UTF-8 map区间及3016同类BCI来源通过root独立核验，default/all全文与map一致。

`combined-matrix-root-v1`：root实际225命令/450raw，24份完整Jarde/JADX源码字节不改，双javac共48次目标重编，外部driver单独编译一次。Jarde32次整类重编、64次driver运行（1744个实际用例调用）、64次原main运行全匹配；每次driver启动均先验证目标CodeSource及恰好两个隔离classpath根，所有运行有`-Xverify:all`。JADX SL四次因不可达break编译失败，SN与LD各八次driver在ledgerBreak判别输入处抛AssertionError，未执行的后续case不计运行；LD八次main仍是9而原为10，SM八次driver匹配。JADX失败均保留，普通SN main通过不能代替其labBreak判别输入。全体driver stdout共2408行不是成功用例数；不能将失败前缀当完整运行。

root重核全部raw pins、重编源码与原API/JADX文本相等、class/driver在运行前后字节恒同、采集期间源码及输入恒同，target不存在。矩阵归档541成员逐字重开，SHA `2feaa84b24a8bc780c0de1115e793aced5e71e64a01ca2bf59c13087c35b8495`。其后新增的负边界及公开停止测试尚待编译验收，不能倒填到该矩阵的来源快照。最终Clippy/MSRV/reader/P5、两fresh workspace与精确产品自身CI仍未完成，任务仍2/7。

## 对抗性验收与来源补全，尚未关闭产品任务

`final-focused-root-v4`实际通过九个返回叶/标签break/方法停止测试及一个公开ClassSource请求级预算边界测试；新CLI与c6e矩阵CLI逐字节相同。此前`negative-focused-root-v3`的463个库测试（含证书缺失、错误/歧义loop target、真实外来predecessor owner、proof内正计费量停止及分析后的取消/期限入口）通过，随后独立integration测试的借用错误已单独保留并修正。取消/期限测试不冒称确定性mid-proof interrupt。编译错误、后继顺序错误及格式收据scope错误均各自保留失败capture。

实际全BCI差集又发现c6e的SN两return方法遗漏goto57/60，共8个default/all配置；其余16目标配置全覆盖。原source-map审查只证明引用合法，不曾证明指令全集覆盖，详见`implicit-continue-map-gap-root-v1/review.json`。root已补设计：已证无局部join的case仍须走既有switch-arm frame，携带精确continue并由LoopContinue保留来源；Luna最小补丁已集成，最终源码重编与全BCI验收待重跑。原c6e语义矩阵不能冒称新源码最终验收。

现有输入确实缺少loop-switch异常边及jsr clone对照，root补充并实际验证了三完整class：`exception-boundary-original-root-v1`双编译器/双JVM，6命令/12raw、4运行均匹配五行oracle；`jsr-boundary-original-root-v1`标准classfile生成的v49输入，4命令/8raw、两JVM均通过完整校验及隔离CodeSource检查。生成器草稿的tableswitch opcode配lookupswitch负载已在实际生成前由root静态审查纠正；不是生产修改，也不存在一次已执行的错误JVM运行。原始边界输入验证不冒称Jarde/JADX重编。

异常边的首个真实Reader测试已通过。jsr的整loop测试因NormalFlowView没有相应自然循环而失败，原464/1结果保留在`clone-test-precondition-failure-root-v1`；新测试仅核真实clone边身份，不伪造Frame，不固定缺失能力，独立问题见`separate-architecture-debt.md`。第二轮gates的Clippy/MSRV通过，reader180/1仅夹具census需刷新，见`reader-census-before-refresh-root-v2`。新11个class的独立原始Code增量为49 bodies/4 handlers/139 targets/2 jsr，预期总量1238/5495/606/3641/12；已更新人口快照，尚待最终Rust walk复核。P5旧性能表和预算未改动。

## 实现与三方矩阵检查点（历史6/7）

`final-focused-root-v5`：修正Closed frame后，root实际通过465库、181reader、九返回叶/标签/停止、12旧switch控制、一公开request停止，共668项。reader完整walk已确认1238/5495/606/3641/12，与独立Code增量相同。两编译器异常边均直接匹配canonical Exception端点，clone同/异上下文边证书正负测试通过。默认栈、来源不变、exact clean/target absent；CLI62908496B，SHA `10a73d1ee30a3820a018a2461cc3c3b3028b02bf658588bc74c4ec3ba2eb1620`。实际源码快照及全部raw在归档，30成员已重开。

`final-api-root-v2`：同CLI实际16请求/32raw，80方法身份/flags/顺序、2152 UTF-8区间/3024 BCI来源合法。六目标24配置的700个物理BCI全部有来源，缺失0；SN57/60已恢复。56非目标稳定投影与605基线完全相同，SM五形未变，default/all正文与map一致。

`final-matrix-root-v2`：最终CLI文本完整原样重编，root225命令/450raw独立接受。Jarde32次重编、64 driver运行1744case、64 main全匹配原oracle；javac8使用source/target8，javac23实际使用`--release 8`，双JVM完整验证及隔离CodeSource通过。JADX原文也重新编译：SL四失败，SN/LD各八driver在ledgerBreak处失败，LD八main仍9而原10，SM八driver匹配；后续未执行case不计通过。来源/工具/生成文本/class/driver均在采集期间不变、target absent；541成员逐字重开，archive SHA `b725f3328641f8f3b81f0ce1da2bb937cfeee50dd2152df33b9c91bf9f61d00c`。

据此只关闭实现与矩阵任务，当前6/7。最终生产源码的Clippy/MSRV/reader/P5/指纹门正在重新运行（gates-root-v3），之后仍需两fresh workspace、strict、精确产品提交/推送及其自身CI、clean-main。不得把旧gates的成功段拼接冒称最终整门通过；不得增加71/612总数。

`final-gates-root-v3`现已实际整门通过：fmt/当前CI同款Clippy、MSRV1.88全workspace/all-targets check、181reader、标准指纹再生/检查及七P5目标、CLI构建。共286 passing appearances/0失败/5忽略，其中常规八个P5目标为104/0/5；P5旧表与预算未改。指纹2305旧项完全不变，仅增20项至2325。源码仅该指纹JSON按预期改变，CLI逐字同10a73矩阵，默认栈/守卫/finally exact clean、target absent，35归档成员重开通过。

`final-strict-root-v2`已接受OpenSpec1.11.0严格367/0及4raw pins。两个fresh workspace与精确产品自身CI仍待验收；当前启动seed1于`/private/tmp/jarde-switch-workspace-seed1-root-v1-20261018`，不能提前当作通过。交付任务保持未勾选。

## 产品与本地双 seed 已接受，自身 CI 收尾中

产品 `cf857abaabc4fcff82689812226f9906b8d932b2` 已推送 main。`workspace-acceptance-root-v1` 已由 root 独立接受：两 fresh seed 各382 targets/100批/3716通过/0失败/98忽略，412 raw 全复核；逐target计数及5016来源完全相同，九个新增关键门每seed在精确target下唯一通过。默认stack、5GiB余量/target1GiB每秒守卫、最终exact clean与target absent通过。最终CLI3063构建来源和5016 workspace来源的5023文件并集逐字/按mode绑定产品Git tree。

归档10240458B / SHA `1e8a89f2f50daedbe7aa619dcb0fee0a08f9b4edb829f2146fafb75f1a652674`，434成员逐字重开；源码绑定只证明来源，不替代实际测试成功。产品自身CI39072531393仍待完成原始日志核验；3.2保持未勾选。

## 最终产品验收：7/7

产品 `cf857abaabc4fcff82689812226f9906b8d932b2` 自身[CI39072531393](https://github.com/LordCasser/jarde/actions/runs/39072531393)已由root独立接受。实际4jobs/52API步骤全部成功，两seed各382targets/3716通过/0失败/98忽略，完整逐target计数一致，27关键门在精确target下每seed唯一ok。四原始job日志、run/jobs/steps JSON、74命令/148raw pins、69个git-show/current来源绑定及实际collector/verifier均保存在`own-product-ci-root-v1`；160成员逐字重开，archive1758013B / SHA `de9a05a85634cae7998e4f68d7cc93c2059dbb958eb9b02cb97a55564596166e`。

本地两fresh seed已接受，见前述`workspace-acceptance-root-v1`；该捕获中的“CI pending”只是当时检查点，不覆盖本节新的CI实际验收。本change7/7，不增加71/612整体完成数。原始红基线、早期测试/采集失败和c6e来源缺口全部保留。

后续关闭提交只含文档与验收证据，不能冒称它自身跑过产品CI。最终实际HEAD/remote一致、clean、仅main/辅助worktree detached且clean/ancestor、target absent及5023来源恒同检查，将保存在`/private/tmp/jarde-switch-final-head-root-v1-20261018/execution.json`，必须以该终态收据和当前Git状态为准。

关闭后的`closing-strict-root-v1`实际OpenSpec1.11.0 apply为all_done/7-of-7，strict367通过/0失败，6raw pins、target absent；9归档成员已逐字重开（9546B / SHA `2eaa9d1e8f98cb9a1045c99752969a784706c8dbcee678b1abd27f3f8bfc125b`）。
