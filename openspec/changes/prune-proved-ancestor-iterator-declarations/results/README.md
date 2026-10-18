# 验收记录

当前任务4/7：最小实现与永久边界已通过，最终完整Java及产品交付验收仍在进行。

- `baseline-revalidation-root-v1/execution.json`：root 实际重核已交付 5154 来源 bytes/Git blob/mode、CLI SHA 和 debug/none 原始 raw/类投影空声明；新 JDK/CLI 执行数为零，旧对照明确沿用。
- `planning-review-luna-v1/`：独立只读规划审查与输入指纹。物理完整方法访问门、实际 AST 祖先、来源和 checkpoint 必须由实现与测试证实。

审查 memo 中“所有 pending 先全部成功再修改”并非本设计要求：每条关系独立完整核验后可清理，无效关系保留；未发布 Builder 后续遇 Stop 返回 Err，因此无需全树克隆或新事务。更正原文另存，原 memo 保留。

## 实际基线与定向实现（3/7，尚未交付）

`baseline-classification-root-v1` 是 root 独立复核：四原始编译配置、8 VM/152 case 逐字共同 oracle；12 API、4 JADX 完整闭包。`controls-baseline-bounded-root-v2` 保存实际完整矩阵：24 Jarde 整类编译均因既有拒绝控制失败，故 Jarde VM 声称为零；8 JADX 编译成功、16 VM/304 case，每轮只有 protected-catch-read-write 与原始结果不同，有限事件 guard 使该错误表现为 AssertionError。原始4与生成8 wrong-root 均拒绝。protectedPositive 的12 API为structured foreach且无空声明；protectedAfterWrite等没有进入foreach，不计新清理门覆盖。scope限定：旧whole-class fallback及安装版JADX1.5.6错误try移动独立记录，不混修当前叶子。

`controls-baseline-fallback-and-timeout-root-v1` 是早一版源形态的实际矩阵，24 Jarde 编译失败、16 JADX VM 真实timeout124原样保留。两原收据采集parser都误拒camelcase用例名，root从完整原始stdout重新解析8份19行；不重写原收据或拿其rows/matches当真。

`focused-compile-failure-root-v1` 保留OriginSet API误用和测试Builder漏新字段的真实编译失败；root修正为直接检查derived本地来源并补字段。`focused-budget-test-failure-root-v2` 保留late-IrItems测试误用全零Limits导致更早AnalysisSteps停止的真实失败；root仅修测试所需维度。`focused-root-v3` 完整5测试通过、fmt/CLI构建成功，root逐source-pin复核，default stack/guard/exactclean完成；候选CLI SHA 60e1b07691100a238c7462a400b381a2be923b366278349f1562f69d3d890ba5。此为定向候选，不是最终产品。

20个同argv API已用该候选实际采集（未重跑JDK），raw、类与方法投影预期只删受证空行。精确checker正在核对派生text_bytes/FNV128摘要及唯一jre_recovery_produced计数，尚未接受其全部差分；完整最终Java、广门/native、产品自身CI与交付均待完成。

## 最终广门与精确 API 差分（4/7）

`broad-old-assertion-failure-root-v1` 保留旧公开测试仍要求冗余空声明而失败的实际收据；新行为已由OpenSpec要求，测试只更新这一预期及对应raw行。`broad-root-v2` 保存全门重跑：810通过/0失败/6既有忽略、18命令36raw；fmt/CI Clippy/MSRV1.88/Java502/Reader181及公开/P5通过。root独立核3194构建来源与CLI，默认stack/5GiB/target1GiB守卫、exact clean/target absent通过。最终CLI63623984B/SHA60e1b07691100a238c7462a400b381a2be923b366278349f1562f69d3d890ba5，和focused v3同字节；最终源码包含后补边界及更新断言。

`api-smoke20-root-v1` 是实际同argv旧新20pair，未跑JDK。root运行v4 checker接受：原始8格与sameList控制12格仅精确删除祖先空声明，保留全部BCI/来源并移位map；binding text_bytes/FNV128和唯一produced诊断计数按删除量重算，usage之外其余JSON严格相同。原始local5删除31B；控制debug inner30B/none local3 31B。不可单凭此称完整Java或产品交付通过。

最终原始与生成矩阵已实际重跑并独立接受，控制矩阵尚在执行；下一步完整绑定、两fresh native、自身CI和文档交付。上文“20差分尚未接受/广门未跑”是历史检查点。

## 最终实现和完整对照接受（5/7，产品 CI/native 待验）

末轮静态对抗审查发现本次新增walk的空arm/catch容器缺独立计费，`empty-container-meter-fix-luna-v8` 保存Luna最小补丁及内部AST预算负例；root应用、格式化并全跑 `broad-root-v3`，810/0/6与9精确关键门通过。最终CLI63623984B/SHA **26eb321d0810a54328f7880483c6c5a5c2033c2ad747f7b177a51e723e96af52**。3194构建来源不变，守卫/默认stack/exactclean与target absent均接受。无新IR/pass/公开API或新遍历机制。

`original-final-root-v1` 和 `generated-final-root-v1` 是60e1 CLI实际完整采集/独立验收：原20命令4VM32case/8API，生成75命令20编译18成功、36VM288case均同原、18wrong-root；2JADX outer框架JDK8编译边界不计运行。`controls-pre-meter-root-v1` 是89命令完整19case控制，root0 integrity error且原8VM152case共同oracle；24Jarde整类编译仍旧拒绝失败/零JardeVM，JADX8编译/16VM304case只protected-catch-read-write每轮不匹配，12wrong-root均拒绝，不能当运行通过。

root实际 `pre-meter-final-bind-root` 核上述完整采集8+12 API与已接受smoke全部非usage JSON相同及物理class bytes相同；输入路径不同已明确记录，非同argv运行。`final-cli-replay20-root-v1` 又用最终26eb实际重放20API+clean，root-v4独立核21实际命令/40APIraw、guard、3194来源与broad-v3恒同、physical bytes及全部非usage JSON逐字等60e1。原v4 delta checker还直接核c688→26eb受证唯一行/map坐标/binding FNV和produced计数变换，其余JSON精确不变。**26eb未重跑JDK**；因完整生成Java字节恒同，明确复用刚实际完成的原始/生成/控制Java矩阵，而非伪称新JVM运行。

`pre-meter-preparations-root-v1` 和 `final-replay-preparations-root-v1` 保存私有实现/审查/checker版本和最终三源码副本。replay reviewer v1虚构schema、v2漏minimal cleanup row被root静态拒绝，v3实际拒绝成功cargo clean的标准stderr；v4要求精确Removed 0 files行、成功exit和target absent，其他判据保留。工具准备失败不是产品失败。主规范已同步；任务3.1按相同Java字节与实际矩阵完整绑定接受，3.2/3.3仍待。

下一固定候选见 `next-fixed-ledger-analysis-luna-v1`；root最终26eb真实输出仍有equals/remove两个Object包装，add(E)已无包装，仅列下一分析，不扩本片或增加CF11/71/612完成数。

## 完整产品验收（6/7，文档主线关闭待核）

实际产品 **ea2aaf042ba91572727abb48c4b6ae8db3ca93a8** 已推main。`workspace-seed1-root-v1`、`workspace-seed2-root-v1` 为两fresh实际终态：各385目标/101批/**3768通过/0失败/98既有忽略**；默认stack、5GiB/target1GiB每秒guard、5147来源不变及exactclean/target absent。`workspace-acceptance-root-v1` 保存root实际三条独立命令：18精确门每轮各一次、全3866具名(name,status)逐目标恒同；构建3194/native各5147并集5154逐Git blob/bytes/mode绑定ea2a，最终26eb CLI不变。collector旧artifact顺序表的错配原样保留，依据实际串行Running/executable与harness绑定，不拿旧表充当逐target证明。

`own-product-ci-root-v1` 是自身 **CI39130065483** 完整接受包：4jobs、**52全部官方API步骤成功**，官方ZIP全部52独立step成员齐全、零aggregate重建；CI双seed各385目标/3768-0-98/3866具名、**63精确门**各通过一次。实际24原始采集+1官方ZIP=25服务命令50raw，root verifier调用另存。13源码/5fixture绑定实际ea2a；后续文档HEAD不能冒称另跑此产品CI。

主规范已同步，当前source/code不再修改。任务3.2已勾；还须文档提交推送后实核clean main/远端/14辅助checkout并关闭3.3。下一固定Object参数包装分析持久在 `openspec/evidence/java-syntax-2026-10-18/cf11-object-argument-analysis-root-v1`，34成员，包含root架构约束、JADX六源码及Luna原/更正版本；未新建下一OpenSpec、未实施或编译新控制。Generic可选清理缺证不得降级既有typed能力、Integer拆箱错误反例已撤回、控制v1误用不同入参/receiver cast由v2纠正，下一root仍须实际JDK验证。

预关闭实际审计：strict375/0，apply6/7 ready，所有当前本片及下一静态分析23归档/1954成员已按相邻SHA和内置manifest逐字复开（`all-archives-root-review-v1.json`）。此报告不预先包含尚未产生的clean-main/最终关闭收据；先文档checkpoint推送实核后再关闭最后任务。
