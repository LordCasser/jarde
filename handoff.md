## 2026-10-19 当前检查点：indexed List 5/6，修正旧 Iterable 预期

本节优先。finThrow5/5、nestedFin16/16已交付勿重做。当前产品b7d53e50的第一次fresh seed1实际在batch40失败：listOwner(List<String>)已合法恢复String foreach，旧测试要求Object。root审查Luna测试补丁，只更新该正向预期；Collection/extra next/escape拒绝不变，新增非空/空/null元素原始及恢复整类执行对照。实际Iterable5/0、语料再生成1及校验5/0/1、CI Clippy/最终CLI构建通过，target absent。失败native和未接受b7 CI快照保留，不能当完整通过。

最终构建 `/private/tmp/jarde-indexed-list-signatures-listowner-final-build-root-v1-20261019/execution.json`，CLI90b30ad1132bdc904c20f1b4624347b84d37637c78f1f3e291ce2f0dea91f64c/63809216B，与既有完整64VM832case矩阵CLI逐字相同。仅test/driver/manifest变化，生产实现不变。归档/详情见当前change/results/README.md。下一产品提交后需其自身官方全部CI日志/两seed和新fresh完整workspace验收，再关闭3.2；不得重用b7验收。

root独占共享/Git/网络/工具链；Luna仅私有。Cargo期间禁止source/Git mutation/network/manualJDK或CLI。5GiB/target1GiB/defaultstack/每秒guard/exactclean。新fresh目录v2，seed5350648285461741569。新CI/native准备v5审查后用，原v4 gate重复有误不得执行。持续goal active，71/612和CF11整项计数不增加。

## 2026-10-19 当前推进：NL2 indexed List 声明，5/6，完整对照接受

本节优先。finThrow5/5、nestedFin16/16及前片已交付，不重做。当前新片 `recover-proved-indexed-list-parameter-signatures` 5/6：完整实际基线、Luna私有实现与root架构审查、focused/适用广门和最终完整源码对照已接受；产品提交、自身CI及fresh workspace尚待。旧main3f888afb1，当前全部新代码/fixture/OpenSpec/evidence未提交。

最终CLI `/private/tmp/jarde-indexed-list-signatures-broad-root-v2-20261019/candidate-cli`，63809216B/SHA90b30ad1132bdc904c20f1b4624347b84d37637c78f1f3e291ce2f0dea91f64c。focused40/0/1，广门578/0、fmt/CI Clippy/MSRV1.88通过，3198来源恒同，target absent。旧清理器不支持multi-target的真实harness失败保留，之后仅续跑未执行目标，不重跑已过门。

完整最终采集184命令/368raw，32Jarde整类编译/64VM832case全同原；完整上游/控制原始各4VM40/64case一致。JADX no-debug双VM各4case重复get事件、2旧框架major55及4非法String→Integer编译失败保留，失败不运行。40 positive方法cells只改ListString头与确切成功marker，16 negative cells仅确切拒绝说明改变，其余代码/raw outcome/map逐字保留。原始Java/class/driver/JDK/oracle已独立重绑。结果入口本change/results/README.md，五归档共1378成员root全部逐字复开。

复用CompleteBody census、精确primitive/Signature/descriptor、唯一get→c0/同SSA/AST Cast(Call)，保留真实cast。Luna v3修旧Stack0 receiver假设（size实际Stack1），root补计费/poll；不新增IR/pass/solver/重复CP事实。两个既有预算债务及显式void-throw quote单列，不混入；nested segment.last丢outer假审查已撤回。主规范一requirement/五scenario已同步，18/18通过；CF11及71/612仍不增整项完成数。

接续：先核Git与target/进程。root独占共享/Git/网络/工具链，Luna私有。产品提交推送后可让远端CI与本地fresh seed1并行，但本地Cargo期间禁止source/Gitmutation/network/manualJDK或CLI。fresh runner为现有workspace_runner.py，seed5350648285461741569；准备好的native/CI/binder在 `/private/tmp/jarde-indexed-list-signatures-final-acceptance-prep-luna-v3-20261019`，保留全部旧门+七新门、14source/9fixture、官方ZIP每step独立日志。默认stack、5GiB、target1GiB每秒guard/exactclean。最后root验收native/自身CI与Git源码/CLI绑定、strict/apply/账本及干净远端main后再勾3.2。持续goal active。

## 2026-10-18 最新交付：Object 参数包装 6/6，自身 CI 完整接受

本节优先。finThrow5/5、nestedFin16/16、父finally7/7及前片均已交付，勿重做。当前 `remove-proved-list-object-argument-wrappers` 6/6；持续 goal active，CF11整体及71单元/612测试不因本片增加完成数。

产品 **096a764dce8bb29b2d7d79e86bef6bea880bc848** 已推，实际自身 **CI39138950945** root完整接受4jobs/52官方API步骤；官方ZIP全部52独立step日志齐全，无aggregate重建。两seed各385目标/3770通过/0失败/98旧忽略、65精确门各一次。采集24命令+1官方ZIP，13源码/5fixture绑定产品；root verifier完整stdout/stderr及argv另存。文档后继不冒称另跑产品CI。

最终CLI `/private/tmp/jarde-object-argument-broad-root-v2-20261018/candidate-cli`，63624704B/SHA56a589ef9a170a1bb2911ed5597c20b2b4e06abf78da845f8fd3946099b2fa51。最终广门812/0/6、fmt/CI Clippy/MSRV1.88通过；fresh本地workspace385目标/101批/3770-0-98、20精确门各一次，3194构建与5147native并集5154来源按Git blob/bytes/mode绑定096a。target absent，无待续Cargo/JDK/CLI。

复用既有元素SSA/cast/store/load/唯一局部证明与opaque AST事务；精确String.equals(Object)/List.remove(Object)只清除冗余Object展示包装，remove不推断E，Integer引用仍选Object重载，缺可选证明保留可靠泛型/typed，Stop传播。v3按唯一equals key分别证明，错key不阻塞其它可靠清理，重复key不授权；不加IR/pass/solver/公开API。

最终String原始4VM32case；完整生成20编译18成功/36VM288case/18wrong-root接受，2次JADXouter/JDK8保留major55框架边界。Integer e22实际10完整编译/20VM120case；最终56实际26API把Integer完整Java绑定e22、24控制非usage JSON绑定旧26eb，未冒称56重跑这些VM。控制旧32整类编译失败不运行，成功40树/80VM224重建targetcase实际同原。来源/raw/map与所有真实失败见本change/results。

下一固定SequentialLoops已有最终56完整验收：8API结构化无fallback/全部25BCI；70命令、24完整编译22成功、44VM264case同原、1wrong-root；Jarde16编译全成功，2JADXbundle/JDK8旧major55边界不运行。既有statementful header足够，无新机制。

CF11其余固定输入最终56实际32API+clean，16对default/all文本/raw map一致，全部方法structured且物理BCI来源闭合；NL3/NL5旧baseline fallback是已交付历史，勿重做。NL2/NL4使用同SHA归档leaf目标class迁移旧临时路径，其余参数保留；这不冒称本轮重跑五组JDK或CF11整项验收。

**下一明确缺口：NestedLoops2泛型Signature**。当前结构化循环仍输出raw List与真实String cast。只读架构memo已定位report/ordinary Signature只接受VoidBody、List合同缺get(I)Object两道门；现有result_value_fact可复用，不需新IR。后续先证明primitive/void返回与参数用途census分离，get结果唯一通往物理cast(E)/retained AST，保留真实cast，不凭Signature放开正文或推断E，再写新OpenSpec派Luna实现。尚无下一Spec/生产改动，勿混入已交付片。

主java8-recovery规范已同步，最终strict376/0、apply all_done6/6，24归档/2025成员已root逐字复开（all-archives-root-review-v2.json）。最终关闭文档提交后的实际clean主线/远端HEAD以外置 `/private/tmp/jarde-object-argument-final-clean-main-root-v1-20261018/execution.json` 为准；接续先核Git再推进。root独占共享/Git/工具链/网络，Luna仅私有/只读；本地串行、defaultstack/5GiB/target1GiB/每秒守卫/exactclean，Cargo期间不改源/Git、不跑JDK/CLI/network。

## 2026-10-18 最新交付：祖先 Iterator 声明清理 7/7

本节优先。finThrow5/5、nestedFin16/16、父finally7/7及typed/原名foreach前片均已交付，勿重做。当前 `prune-proved-ancestor-iterator-declarations` **7/7**；持续goal active，仍按JADX71单元/612测试账本推进，不因叶子关闭全局目标或增加CF11完成数。

实际产品 **ea2aaf042ba91572727abb48c4b6ae8db3ca93a8** 已推，自身 **CI39130065483** root完整接受4jobs/52全部官方API step、两seed各385目标/3768通过/0失败/98旧忽略、63精确门各一次；官方ZIP全部52独立step成员齐全，无aggregate重建。后续提交只记录文档，不冒称另跑产品CI。

最终CLI `/private/tmp/jarde-ancestor-iterator-broad-root-v3-20261018/candidate-cli`，63623984B/SHA **26eb321d0810a54328f7880483c6c5a5c2033c2ad747f7b177a51e723e96af52**。最终广门810/0/6、fmt/CI Clippy/MSRV1.88通过；两fresh native实际各385目标/101批/3768-0-98，18门各一次、全部3866具名结果逐target同。构建3194/native5147并集5154按Git blob/bytes/mode绑定ea2a。目标target已exactclean，无Cargo/JDK/CLI待续。

root/Luna仅复用既有foreach/slot/SSA和checkpoint，增加成功消费者窄关联与最终真实祖先声明唯一核验；同变量其它SSA值、额外读写、缺证/重复/兄弟/错来源均保留原声明和可靠循环，不建新IR/pass/solver。末审空arm/catch计费缺口已v8补齐并全跑，Stop不发布半完成方法。

完整原始Java4VM32case、生成20编译18成功/2旧JADX框架JDK8边界、36VM288case/18wrong-root均实际接受。19case控制仍有24Jarde旧拒绝整类编译失败/零JardeVM、JADX16VM304case仅catch控制不匹配，不能当runtime通过。最终26eb实际20同argv API全部非usage JSON/whole/projected/raw/map等60e1，physical class bytes绑定完整矩阵；明确复用相同Java字节，26eb未重跑JDK。主规范同步、strict375/0和apply all_done最终结果见results关闭记录；所有真实失败与被替代checker均保留。

最终strict375/0、apply7/7 all_done；本片及下一静态分析25归档/2039成员已root全部逐字复开（results/all-archives-root-review-v2.json）。

检查点 **1665f548fd6c16c1346173ffe5f7cb2e13418048** 已实际核远端==本地干净main，仅main分支，14aux detached-clean-main祖先/无target，5154来源及CLI恒同。最后关闭文档推送后，以外置 `/private/tmp/jarde-ancestor-iterator-final-clean-main-root-v1-20261018/execution.json` 的实际HEAD/终态为准；接续先重核Git。结果总入口 `openspec/changes/prune-proved-ancestor-iterator-declarations/results/README.md`。

**下一固定TestNestedLoops equals/remove Object参数包装**：最终真实26eb已确认仍在，add(E)无包装，不重做。仅静态分析及3个私有控制类准备，无新OpenSpec/实施/新JDK运行。持久 `openspec/evidence/java-syntax-2026-10-18/cf11-object-argument-analysis-root-v1`（34成员/de450ebc…）含JADX六源码、root架构约束与Luna原/更正版本：先证明确切调用选择，再用既有opaque AST exact wrapper/removals事务；现有add物理cast→store→load证明可提取复用，remove参数不能推断List<E>，可选清理缺证不能让可靠泛型/typed能力降级，不在descriptor-only Builder入口做通用Object擦除。Integer拆箱反例已撤回；primitive同值/自定义equals重载/真实参数CHECKCAST控制准备v2尚未编译，须root先核全源/driver和javac8/23实际descriptor/oracle，再写Spec派Luna实现、root完整对照验收。

root独占共享/Git/工具链/网络，Luna只私有/只读。资源守卫仍defaultstack/至少5GiB/target1GiB每秒检查/exactclean；本地工具串行，Cargo运行时禁源码/Gitmutation/JDK/CLI/network。

## 2026-10-18 当前验收：祖先 Iterator 产品及自身CI通过，6/7待主线关闭

本节优先。finThrow5/5、nestedFin16/16、父finally7/7及前片foreach均已交付，勿重做。当前 `prune-proved-ancestor-iterator-declarations` 6/7：实际产品 **ea2aaf042ba91572727abb48c4b6ae8db3ca93a8** 已推；自身 **CI39130065483** 已root完整接受4jobs/52官方step/63精确门，全部52独立ZIP step成员齐全，无aggregate补造。

两fresh native实际各385目标/101批/3768通过/0失败/98旧忽略，18门每轮各一次，全3866具名结果逐target同；3194构建/5147native并集5154按Git blob/bytes/mode绑定ea2a。actual复核三命令 `/private/tmp/jarde-ancestor-iterator-workspace-acceptance-root-v1-20261018/execution.json`；target absent，无Cargo待续。最终CLI26eb321d0810a54328f7880483c6c5a5c2033c2ad747f7b177a51e723e96af52/63623984B，最终广门810/0/6，fmt/CI Clippy/MSRV1.88通过。新walk空容器计费缺口已Luna v8+root修正并全跑，无新IR/pass/公共API。

完整Java与最终20同argv非usage精确重绑见results/README，明确26eb只重跑API，复用完整Java字节相同的60e1真实JDK矩阵，不伪称新JDK运行。原始4VM32case；生成20编译18成功/2JADX旧框架JDK8边界、36VM288case/18wrong-root；控制19case原8VM152case，24Jarde旧整类拒绝编译失败/零JardeVM，JADX16VM304case仅catch控制不匹配，失败边界不混称通过。

剩余：strict/apply与全部归档复开、文档checkpoint提交推送/实际clean main和14aux detached-clean-ancestor核验后勾3.3，再关闭文档push及最终外置clean-head收据。当前文档HEAD是产品后继，不能称其跑过ea2a产品CI。持续goal active，不因叶子完成关闭。

下一固定equals/remove Object包装仅静态分析及3个私有控制类准备，持久 `openspec/evidence/java-syntax-2026-10-18/cf11-object-argument-analysis-root-v1`（34成员/de450ebc…）。root定窄证明后才新OpenSpec派Luna实施：复用opaque AST exact wrapper/removals事务和已存在SSA元素cast→store→load证据；不能通用删Object cast，不能让可选清理缺证撤回可靠泛型/typed foreach；remove参数不推断List<E>。JADX重载显式保护算法可参考，Integer拆箱假反例已更正，primitive同入参/真实参数checkcast等控制未编译，需root实际javac8/23确定。add(E)已无包装，不重做；CF11/71/612计数不增加。

## 2026-10-18 当前运行：fresh native seed1通过，seed2接续

本节优先。实际产品ea2aaf042已推，自身CI39130065483尚未完整接受。seed1-root-v1真实终态exit0：385目标/101批/3768通过/0失败/98旧忽略、5147来源不变、exactclean/target absent。双轮独立具名验收仍待，不把collector旧顺序表当实际逐target计数。

root即将/正在fresh seed2 `/private/tmp/jarde-ancestor-iterator-workspace-seed2-root-v1-20261018`，5350648285461741570。先核真实receipt/进程再接续，不覆盖重启。Cargo期间禁源码/Gitmutation/JDK/CLI/network；现有workspace_runner、defaultstack/5GiB/target1GiB每秒guard/exactclean。两轮后verify-native18门、全部具名比较、product binder绑定ea2a，实际CI官方ZIP/verify-own-ci63门，再strict/全归档/账本/handoff/干净main。任务仍5/7。

下一Object包装仅静态分析和私有控制准备。root已否定通用String→Object assignability删cast，以及把kind1(add)直接扩kind2(remove)会让可选清理缺证撤回可靠泛型的做法；优先复用opaque AST removals和窄物理producer证据、失败保持原包装。Luna的Integer拆箱反例错误和控制v1非同实参/非argument-cast问题均已更正保留版本，尚无下一Spec/实现/新JDK运行。

## 2026-10-18 当前运行：ea2aaf042 已推，fresh native seed1

本节优先。祖先Iterator清理5/7，实际产品 **ea2aaf042ba91572727abb48c4b6ae8db3ca93a8** 已推main，自身CI **39130065483** 已in_progress，未接受。最终CLI26eb321d…和810/0/6广门、完整Java20API字节重绑已接受，详下节与results。

root即将/正在运行seed1 `/private/tmp/jarde-ancestor-iterator-workspace-seed1-root-v1-20261018`、5350648285461741569，随后fresh seed2-root-v1 /5350648285461741570。先核真实进程/receipt，不重启覆盖。Cargo期间不源码/Gitmutation/JDK/CLI/network，默认stack/5GiB/target1GiB每秒guard及finally exactclean。两轮终态后用ci-prep-luna-v1 verify-native18门、review_named全具名、product binder绑定实际ea2a，再实际收CI官方ZIP/verify-own-ci63门/全部jobstep，两项任务未完成。

root已复开此前17归档1402成员，native/CI未来包尚未验证。finThrow/nestedFin及已交付前片勿重做，持续goal active；下一两个Object参数包装仅固定黄金分析，不扩本片。

## 2026-10-18 当前推进：祖先 Iterator 空声明，5/7，产品待提交

本节优先。finThrow5/5、nestedFin16/16、父finally7/7及前片foreach原名绑定7/7已交付，勿重做；当前 `prune-proved-ancestor-iterator-declarations` 5/7，持续goal active。生产/永久回归/完整对照已接受，实际产品提交、自身CI和两fresh native尚待。

最终v8仅补新增AST遍历的空switch arm/catch容器逐项计费，root接受并重跑最终broad-v3：810通过/0失败/6旧忽略、fmt/CI Clippy/MSRV1.88、Java502/Reader181/公开/P5通过。CLI `/private/tmp/jarde-ancestor-iterator-broad-root-v3-20261018/candidate-cli`，63623984B/SHA26eb321d0810a54328f7880483c6c5a5c2033c2ad747f7b177a51e723e96af52，3194构建来源不变、target absent。

最终21命令20API重放已root独立接受，完整非usage JSON/whole/projected/raw/map等先前60e1 CLI相同，同时直接对c688基线精确核只删受证空声明及必要坐标/摘要/计数。physical class字节绑定刚实际完成的完整Java矩阵：原4VM32case，生成20编译18成功/2旧JADX框架JDK8边界、36VM288case/18wrong-root；19case控制8原VM152case，24Jarde整类旧拒绝编译失败/零JardeVM，JADX8编译/16VM304case均仅catch控制不匹配，12wrong-root。最终26eb只重跑API，未冒称重跑JDK；依赖相同完整生成Java字节复用实际运行，所有失败边界保留。主规范已同步并18/18 strict通过，全库strict375/0；任务3.3尚待最后交付验证。

下一步先提交推送本产品并取自身CI run id，再使用既有workspace_runner串行fresh seeds5350648285461741569/1570；root独占共享/Git/工具链/网络，Cargo期间禁源码/Gitmutation/JDK/CLI/network、5GiB/target1GiB/defaultstack/exactclean。然后18 native门/63CI门完整具名验收、5154来源绑定、全部归档复开/固定账本/handoff/clean main。下一固定TestNestedLoops equals/remove Object包装已在最终真实26eb API确认仍在，仅分析无新Spec/实现；add(E)无包装，不重做。CF11/71/612计数不增加。

## 2026-10-18 当前推进：祖先 Iterator 空声明，4/7，最终 Java 验收中

本节优先。finThrow5/5、nestedFin16/16、父finally7/7和foreach原名绑定7/7已交付，勿重做。main已交付HEAD仍b45167b72，新change `prune-proved-ancestor-iterator-declarations` 及三产品文件未提交；持续goal active。

最终广门v2实际810通过/0失败/6既有忽略，fmt/CI Clippy/MSRV1.88/Java502/Reader181/公开与P5门通过，root独立复核3194构建来源未变及CLI SHA。最终CLI `/private/tmp/jarde-ancestor-iterator-broad-root-v2-20261018/candidate-cli`，63623984B/SHA60e1b07691100a238c7462a400b381a2be923b366278349f1562f69d3d890ba5；target absent。v1旧测试要求保留空声明的真实失败另存，仅按新规格改该断言再全跑。永久边界已过，任务2.2勾选。

20同argv API精确差分已root接受：仅删除受证空行及相应map坐标、binding字节/FNV摘要和produced计数，其余全部非usage JSON相同。最终原始20命令/4VM32case/8API已独立接受；完整生成75命令/20编译18成功/2旧JADX outer框架JDK8边界、36VM288case/18wrong-root亦已独立接受。最终19case控制正在 `/private/tmp/jarde-ancestor-iterator-controls-final-root-v1-20261018` 采集，exec session27192；仍待独立审查和最终20API输入字节绑定，不把旧整类拒绝或JADX catch错误算runtime通过。两fresh native、产品提交/自身CI、主规范/strict及干净main尚未完成。root独占共享/Git/工具链/网络，本地工具串行；Cargo期间不写源/跑网络/JDK/CLI，5GiB/target1GiB/defaultstack/exactclean。

## 2026-10-18 当前推进：祖先 Iterator 空声明，3/7，未交付

本节优先。finThrow5/5、nestedFin16/16、父finally7/7及foreach原名绑定7/7已交付，勿重做。当前main已交付HEAD仍b45167b72；新change `prune-proved-ancestor-iterator-declarations` 与两产品文件未提交，持续goal active。

root已完成全变量物理访问/成功消费者/最终真实AST设计，Luna私有实现并root修正，既有checkpoint以append长度截断恢复。guarded focused v3实际5测试/fmt/CLI通过，CLI在 `/private/tmp/jarde-ancestor-iterator-focused-root-v3-20261018/candidate-cli`、SHA60e1b07691100a238c7462a400b381a2be923b366278349f1562f69d3d890ba5；target absent，无待续Cargo。v1真实编译失败/v2测试错误均归档，勿当通过。

任务1.1/1.2/2.1完成，2.2仍待Luna v6边界测试及root运行。19case控制原始四编译配置8VM152case共同oracle；24 Jarde整类compile失败为旧refusal，零Jarde VM，不算运行通过。安装版JADX1.5.6的catch控制有错误重试，早版16VM真实timeout，有限128事件版16VM完整304case均有同一catch用例不匹配；独立债务，不扩本片。protectedPositive12API可foreach，其它protected拒绝不计clean门覆盖。原collector大小写parser错误已从raw独立纠正，收据未改。

20同argv API已实际采集，候选原始raw/map精确差分checker还需验证目标text_bytes/FNV128和produced诊断的确定计数变化，不能称已接受最终差分。完整最终JDK/JADX/Jarde对照、广门、两fresh native、自身CI/主spec/strict/干净main均未完成。结果入口新change/results/README.md。root独占共享/Git/工具链/网络，Luna仅私有；本地工具串行、Cargo时禁源码/Gitmutation/JDK/CLI/network、5GiB/target1GiB/defaultstack/exactclean。下一步先核实际进程状态，再审Luna v6 test-only patch，不重复新建工作树或重做已交付语法。

## 2026-10-18 最新交付：foreach 原名绑定 7/7，自身CI完整接受

本节优先。finThrow5/5、nestedFin16/16、父finally7/7、typed7/7/bulk4/4早已交付，勿重做；本片 `fold-proved-foreach-local-bindings` 现7/7。持续goal active，按固定JADX71单元/612测试推进；CF11整体及71/612计数不增。

实际产品 **0a7e3c98f63b128a13a658a475b9e69a9e60d048** 已推。自身CI **39120933555** 已root独立接受4jobs/52全部step成功，官方ZIP全部52独立step成员齐全、零aggregate重建；双seed各385目标/3766通过/0失败/98旧忽略、61精确门各一次。25原始采集命令50raw、13源码/5fixture绑定0a7e；verifier调用另存。文档后继不冒称另跑产品CI。 最终strict374/0、apply all_done；本片及下一静态分析22归档1832成员已root逐字复开，见results/all-archives-root-review-v4.json。持久自身CI包75成员/915e61b6…，本片results索引汇总全部证据与真实旧失败。

最终CLI `/private/tmp/jarde-foreach-fold-broad-root-v1-20261018/candidate-cli`，63586736B/SHA **c688920cac70ee9e46681672ebaaba5df2948c5ef5b5f262d006548ca1f83ebe**。广门808/0/6旧忽略；fmt/CI Clippy/MSRV1.88通过。Reader1303/5755/630/4066/14、fingerprint2432，全部旧2429条目不变。复用typed事务/slot/canonical SSA/真实词法范围窄证明，闭合才提升已有名、移除首copy/无用空声明，缺证保留typed，无新IR/solver/pass。实际SSA None use是phi结构边，全部Some/物理/AST读取仍一一闭合，真实Phi读取仍拒绝。

完整三方Java本CLI已实际重跑：原4VM32case；生成20编译18成功/2JADX outer框架边界、36VM288case/18wrong-root；19case控制24Jarde编译/48VM912case同原/24wrong-root，8JADX String→Integer整类编译失败保留。精确20API差分仅受证头/copy/空声明变化，其他projected/raw text/map/quality/fallback不变。没有手修/裁源或用旧JVM收据冒充新跑。

两fresh native实际各385targets/101批/3766-0-98、16门各一次，全部3864具名(name,status)逐target相等；5147 native/3194 build并集5154逐Git blob/bytes/mode绑定0a7e。actual三命令接受 `/private/tmp/jarde-foreach-fold-workspace-acceptance-root-v2-20261018`；默认stack/5GiB/target1GiB每秒guard/exactclean，target absent无待续Cargo。原v1验收旧checker把root jarde lib误当test的失败保留；v3只改kind这一行，保留全部判据。CI verifier仍v2，不能混淆各版本。

检查点 **716fefc658958bfc93a4092186913ee3306b8b1f** 已实际实核远端==本地干净main、仅main分支、14辅助checkout detached-clean-main祖先/无target，全5154来源及CLI恒同。最终关闭文档push后，以外置 `/private/tmp/jarde-foreach-fold-final-clean-main-root-v1-20261018/execution.json` 的实际HEAD/终态为准；接续先重核Git，不把文档head当产品CI的head。

下一固定项：**Iterator祖先空声明清理**，只有分析，无新Spec/实现。持久 `openspec/evidence/java-syntax-2026-10-18/cf11-foreach-iterator-declaration-analysis-root-v1`（11成员/62b4fdf…）保留root/Luna更正及JADX五源码。local5空声明来自astore@33，祖先Vec在调用栈、当前Builder无待删状态；优先复用已有foreach/slot/SSA/owner证书，必要仅补窄跨列表关联并纳入checkpoint/唯一实际foreach/预算与原子提交，不建通用死变量pass。方法/SSA上下文已有，不重复建模。字面黄金须区分全限定类型和none确定性名字，不能称五个containsOne全过；equals/remove Object转型留待独立重载证明，add(E)包装早已移除。

root负责架构/共享写/Git/工具链/网络和完整验收，Luna私有或只读；本地工具串行，Cargo时不改源或网络。所有历史失败、未执行准备、被替代判据和已交付项保留，下一次不重复做finThrow/nestedFin或制造新add(E)问题。

## 2026-10-18 最新交付：typed foreach 7/7、bulk 4/4，自身CI已完整接受

此节优先，下面运行态均是历史。finThrow5/5、nestedFin16/16、父finally7/7早已交付，勿重做；持续goal active，继续按JADX71单元/612测试账本推进，不因叶子完成关闭全局目标。

最终实际产品 **e69a3494c8fcd0ff4783be1e9ff71499a730adde**（typed b78b34ed6加独立bulk guard入口修复）已推。自身[CI39112513084](https://github.com/LordCasser/jarde/actions/runs/39112513084)已root实际独立接受：4jobs/52全部API步骤成功，双seed各385目标/3759通过/0失败/98旧忽略，54精确关键门各一次。官方ZIP全部52独立step成员齐全、零aggregate重建；21原始采集+1ZIP共22命令44raw，12源码/3fixture绑定e69。文档关闭提交只是产品后继，不冒称另跑产品CI。

两fresh native v2实际终态并独立接受：各385目标/101批/3759-0-98、9精确门各一次；完整3857具名(name,status)逐target相等，来源各5143/构建3190并集5150逐Git blob/bytes/mode绑定e69。collector legacy每轮94批顺序错配原样保留，以真实Cargo串行Running/executable绑定harness；Luna最初cross-stream疑点已按本机Cargo文档撤回，原审与更正都归档。没有待续Cargo进程，target absent。

最终CLI `/private/tmp/jarde-typed-foreach-bulk-interval-broad-root-v4-20261018/candidate-cli`，63405424B/SHA256 **85ca6e7d36c0edeb72f7f6a3c33c405f203028fb21d128522f0b0a3a92e1cf78**。广门822/0/6、fmt/CI Clippy/MSRV1.88、Reader1301/5751/630/4050/14、fingerprint2429均接受。实际20同argv API只去usage键后其余全部JSON/完整Java/raw map逐字相同；明确复用原始final36rebuilt JVM/288case及控制Jarde48JVM/912case，85ca没有重跑这些JDK矩阵。原JADX8类型不匹配整类编译失败和2outer框架javac8边界保留。

typed `recover-typed-list-foreach-elements`7/7、独立 `align-bulk-class-task-execution-interval`4/4。最终严格验证373/0，apply两项均all_done，实际收据见bulk results/delivery-checks-root-v1。两change及下一静态分析共26归档2423成员已root逐字复开（bulk results/all-archives-root-review-v3.json），完整自身CI包140成员/c8edfd95…；所有旧失败/runner契约失败/被替代CI动作保持真实。预交付文档81fb88bc2已实核实际远端==本地干净main、仅本地main分支、14辅助checkout detached-clean-ancestor、5150来源与CLI不变。最后交付文档push后需以外置 `/private/tmp/jarde-typed-foreach-final-clean-main-root-v1-20261018/execution.json` 的真实HEAD/终态为准；接续先重核Git状态。

下一固定项：**原名foreach元素绑定提升**，仅有分析，无新Spec/实现。持久入口 `openspec/evidence/java-syntax-2026-10-18/cf11-foreach-original-bindings-analysis-root-v1`（9成员/41e84a06…）：外层blank s1声明和首Assign同BCI25，须结合节点形态/唯一性/完整scope；ForEach锚点13/42不是next17/47。现有slot/SSA/opaque AST能复用，缺的是全部local-reference到词法范围的窄关系。参考JADX usedOnlyInLoop/assignOnlyInLoop/CodeVar原变量，不照搬其IR/flags或宽类型推断；wrapped路径静态差异不是已确认新bug。后续OpenSpec需明确演进“保留body binding”契约，仅完整证书通过者允许折叠，initialized/循环前后可见/冲突/缺证及Stop保持可靠typed表示。先root定Spec再派Luna私有实现/对照，root架构审查；不加全局solver/pass。

仍保留synthetic头名与body copy，冗余iterator/Object包装、既有AST计费预扫另列；不能声称整条上游黄金或CF11/71/612完成。root负责共享写/Git/Rust/JDK/CLI/网络，Luna私有或只读；本地工具串行，Rust期间不改源或网络，默认stack/至少5GiB/target1GiB每秒guard/exactclean。

## 2026-10-18 最新：双轮native与5150来源绑定已接受，等e69自身CI

此节优先。最终产品e69a3494c8fcd0ff4783be1e9ff71499a730adde仍已推main。两fresh native v2都实际终态exit0：每轮385目标/101批/3759通过/0失败/98旧忽略，9精确门各一次；root已实际运行v3独立checker重建metadata/串行Running/harness，追加全部3857具名(name,status)逐target相等。collector legacy每轮94批顺序错配保留，不拿它当真。来源每轮5143/构建3190并集5150按Git blob/bytes/mode绑定e69，85ca CLI不变、target absent；没有Cargo进程待续。

验收actual `/private/tmp/jarde-typed-foreach-workspace-acceptance-root-v2-20261018`，root launcher `/private/tmp/jarde-typed-foreach-native-close-root-v1-20261018.py`，三命令均exit0。bulk results新增workspace-seed1-root-v2(213)、seed2-root-v2(212)、workspace-acceptance-root-v2(22)，完整源码和工具/复核均保留；旧b78失败和取消CI原动作另存。typed **6/7**、bulk **3/4**；任务最后一项尚未勾。strict刚实际373/0。

实际自身CI39112513084最近仍stable pass2运行（其它3job成功），随后还有ignored JDK/comparison与workflow门；尚未接受，须完整collector/官方ZIP/v3 verifier54门后才close。native结束后可网络操作。最终应root重开所有归档、更新账本/strict/handoff，文档关闭提交推送，核远端==本地/干净main及14旧checkout detached-clean-ancestor/target absent；文档head不冒称跑过产品CI。

下一固定原名foreach分析已存 openspec/evidence/java-syntax-2026-10-18/cf11-foreach-original-bindings-analysis-root-v1（9成员/41e84a06…）：同BCI25不同scope、13/42锚点、物理slot/SSA、所缺local-reference→词法范围关系、JADX算法及未来规格约束显式演进。仅分析，无新Spec/实现/新运行，不加CF11/71/612计数。finThrow5/5、nestedFin16/16、父finally7/7早已交付，不重做；全局goal active。

## 2026-10-18 最新运行态：fresh native seed1通过，seed2运行中

此节优先。最终产品仍 **e69a3494c8fcd0ff4783be1e9ff71499a730adde**，main/origin已推。seed1 v2实际终态exit0：385目标/101批/3759通过/0失败/98旧忽略，5143来源pins未变、exactclean/target absent；尚待双轮独立验收，不能把collector legacy顺序当逐target计数。root当前seed2 v2 `/private/tmp/jarde-typed-foreach-workspace-seed2-root-v2-20261018`，seed5350648285461741570，exec session **41573**。Cargo期间仍不改源码/跑Gitmutation/JDK/CLI/网络。

两轮后使用own-ci-prep-luna-v3 native wrapper9门、source binder；root已额外准备 `/private/tmp/jarde-typed-foreach-native-named-review-root-v1-20261018/review_named.py` 比较全部具名(status)逐目标相等。Luna最初cross-stream疑点经本机Cargo文档“test targets run serially”证据撤回，更正与原review均在acceptance-adversarial-luna-v1私有目录保留；当前385目标中仅task_operations example/test重名，所有54CI/9native门目标无歧义。

实际自身CI **39112513084** 刚核3job成功、stable仍in_progress；状态收据 `/private/tmp/jarde-typed-foreach-own-ci-status-root-v1-20261018`，尚未接受。typed5/7、bulk3/4不变。下一原名fold仍分析，Luna确认首个缺口为local-reference→scope窄关系，未改共享代码/创建Spec，详 `/private/tmp/jarde-foreach-original-bindings-adversarial-luna-v1-20261018`。finThrow/nestedFin已完成，不重做；全局goal active。

## 2026-10-18 最新运行态：e69a3494c已推，fresh native seed1 v2运行中

此节优先。最终实际产品 **e69a3494c8fcd0ff4783be1e9ff71499a730adde** 已推main，独立bulk修复与typed候选b78分开提交；自身CI **39112513084** 已实际in_progress，尚未接受。旧b78 CI39111223906在3job成功/stable仍运行时被root取消为过期候选，动作/原状态收据在 `/private/tmp/jarde-bulk-interval-superseded-ci-root-v1-20261018`，未冒称旧CI完整成功。

root当前运行 `workspace_runner.py --out /private/tmp/jarde-typed-foreach-workspace-seed1-root-v2-20261018 --seed 5350648285461741569`，exec session **91725**；第一失败native v1绝不可复用为通过。等实际终态并exactclean后，再fresh运行seed2-root-v2 /5350648285461741570；不得两轮并行。Cargo期间不改源码/跑Gitmutation/JDK/CLI/网络，可只读核记录/文档归档。默认stack、5GiB余量、target1GiB每秒守卫、finally exactclean。

bulk change3/4、typed5/7；已通过最终822/0/6广门、85ca6e7d… CLI63405424B、20同argv API全部非usage JSON/完整Java/raw maps精确等价，源3190/3301闭合。剩余两fresh native、v3 verifier独立验收、来源Git blob绑定e69、该提交自身CI54门/完整job-step双seed、strict/归档/handoff/clean main后才关任务。产品binder与ownCI/native v3工具路径见下节。CI收集须完整实际日志和必要官方ZIP，不凭绿色状态或旧run接受。

## 2026-10-18 最新：bulk独立修复已过822门和20API，新产品/native/自身CI待验

此节优先。typed foreach候选b78b34ed6已推，其第一fresh native因既有bulk计数区间错位失败，不能算通过；失败与源pins/exactclean已完整保留。独立 `align-bulk-class-task-execution-interval` 修复仅将现有Entered guard移到run_class_task入口、gate/delay/入口panic之前，加一条注释；不改gate/timeout/阈值/assert/调度/模型，无新机制。

该独立change现3/4；typed change现5/7。root广门v4实际26命令/822通过/0失败/6旧忽略，bulk16测试+4精确重叠复跑、CI Clippy/MSRV/Java496/Reader181/P5全部通过，构建3190来源不变、fingerprint2429不变、exactclean/target absent。新最终CLI `/private/tmp/jarde-typed-foreach-bulk-interval-broad-root-v4-20261018/candidate-cli`，63405424B/SHA **85ca6e7d36c0edeb72f7f6a3c33c405f203028fb21d128522f0b0a3a92e1cf78**。实际20同argv API/40raw独立接受全部非usage JSON/whole/projected/raw/map逐字同d130，3301来源和冻结产物pins前后闭合。未重跑JDK，仅精确相同完整Java绑定原始36JVM/288case与控制Jarde48JVM/912case；旧JADX8类型错误编译失败及outer框架边界不掩盖。

下一步单独提交/推送bulk修复，让其自身CI与本地两个fresh native并行。native输出需fresh seed1-root-v2和seed2-root-v2；5350648285461741569/1570，既有workspace_runner，默认stack/5GiB/target1GiB守卫/每秒检查/exactclean。当前尚未开始新native或接受新CI。旧CI39111223906只属于b78候选，不是修复后自身CI。root独占共享修改/Git/Rust/JDK/CLI/网络，工具本地串行，Rust运行中不得改源/跑网络。

验收工具已root审v3差异，位于 `/private/tmp/jarde-typed-foreach-own-ci-prep-luna-v3-20261018`：12源码含bulk.rs/3fixture/54CI关键门；native wrapper9门/385目标基线，完整源/逐target具名结果动态derive，无passing总数硬编码。产品binder `/private/tmp/jarde-typed-foreach-product-bind-prep-root-v1-20261018/bind_product.py`适用本轮allowed_source_changes=[]。证据归档见两个change的results/README；bulk当前5包188成员已全部root逐字复开。v3 runner一命令4binary收尾拒绝单列保留，v4拆单目标后完整通过，未改写旧失败。

finThrow5/5、nestedFin16/16、父finally7/7早已交付，勿重做。后续原名foreach头仅有私有分析（已纳入bulk tools归档），没有Spec/实现；当前合成名字/原copy仍保留，不冒称整个上游黄金/CF11/71单元完成。全局goal active，须两fresh native+实际产品自身CI+strict/干净main全部验收后才关这两叶子。

## 2026-10-18 最新阻断：fresh native发现旧bulk计数区间错位，独立OpenSpec修复中

此节优先。已推候选 **b78b34ed6ee908309de4f85d771adcab8ee24133**，自身CI39111223906仍进行中/未验收。typed foreach已通过最终802/0/6广门和16bd CLI的20API与完整Java矩阵绑定；但第一fresh native在第三测试批command4真实失败，不能算成功seed。target已exactclean且源pins未变。失败为bulk_recovery_lifecycle的真实gate/worker重叠已证2，公开concurrent_classes_high_water却为1。失败原始收据和root复核完整保存在 `openspec/changes/align-bulk-class-task-execution-interval/results/native-failure-root-v1`（19成员/ada99433…），不是typed语法或Java对照失败。

独立change `align-bulk-class-task-execution-interval` 已strict/apply ready，1/4。root已审Luna最小patch并应用到src/bulk.rs：将唯一Entered guard移动至run_class_task体入口、panics_at/gate/delay之前，仍使用既有mutex/RAII，worker和serial共用；不改ClassGate、timeout、阈值、旧assert、调度、重量表或公共模型，无新机制。编译前fmtcheck终态通过。

当前正在root串行执行 `/private/tmp/jarde-typed-foreach-bulk-interval-broad-root-v4-20261018`，包括分开的4个bulk目标、4次精确真实重叠回放及原fmt/CI Clippy/MSRV/Java/Reader/P5/CLI全门（v3的4 bulk共16测试已全通过，但清理helper要求单目标，因一个命令4binary真实拒绝；已exactclean并另存runner-contract-failure-root-v3/8成员，不算完整广门通过）；runner为既有run_validation，默认stack/5GiB/target1GiB守卫/exact clean。任务1.2须等真实focused通过才勾。Rust期间不得改源或执行JDK/CLI/网络/Git mutation。

随后新CLI以既有 replay-prep-root-v3 和 review-prep-root-v3 重放20API（fresh输出v2）并逐字比较除usage外全部JSON/whole/projected/raw/map，明确未重跑JDK；成功后typed task3.1重新勾，当前暂4/7。bulk/typed两项待新最终CLI、两fresh native、单独修复提交/自身新CI全部接受后收口，不重试掩盖旧native失败。旧b78 CI不得冒称新修复产品CI。

新版验收工具 `/private/tmp/jarde-typed-foreach-own-ci-prep-luna-v3-20261018` 已静态准备，collector/verifier同步12源码含bulk.rs/3fixture/54关键门，native wrapper9关键门/385目标基线，尚未执行；root仍须审diff。原native seed1 v1失败后新两轮从fresh v2目录开始；仍用workspace_runner两个固定seed。root负责所有共享/工具链/Git/网络，Luna仅私有。

原名foreach头下一片仅有私有分析 `/private/tmp/jarde-foreach-original-bindings-root-analysis-v2-20261018`，没有规划或实现。Atlas report.rs缓存includeCode字节范围过旧，全部字段已按当前源手动核，不引用其dataflow完整性；工具债务独立。finThrow5/5、nestedFin16/16、父finally7/7早已交付，不重做。全局goal active、CF11/71/612计数不增加。

## 2026-10-18 最新：最终 typed foreach CLI 已通过广门及20API重绑，native/自身CI待验收

此节优先于下方历史。finThrow5/5、nestedFin16/16、父finally7/7已交付，勿重做；持续goal active。当前 `recover-typed-list-foreach-elements` **5/7**：最终代码、来源/计费审查、完整原始/控制矩阵与hardened CLI重绑已接受，余两fresh native和实际产品CI/交付关闭。

CLI `/private/tmp/jarde-typed-foreach-broad-root-v2-20261018/candidate-cli`，63405424B / SHA256 **16bd83857cfbcc6023013a6fc67b79030472f5beb0a448baba0b9b6599c7d27f**。广门802/0/6、fmt/CI Clippy/MSRV1.88通过，Reader1301/5751/630/4050/14、fingerprint2429且旧2427项不变，来源3190完全未变、exact clean/target absent。实际20同argv API/40raw核除usage外全部JSON与d130相等，3301来源及物理产物pins前后闭合，完整Java/每方法raw map逐字同。复用原始final 36JVM/288case、控制Jarde48JVM/912case运行证据；只重跑API，未重跑JDK，不把JADX8个类型错误编译失败冒称runtime。

代码复用既有泛型源码事务/opaque AST，保留合成头名及原Assign/Declare，不新增IR/pass/scope solver。全部primary/derived来源一致校验、final计划闭包重验、新访问提前计费/Stop不发布；三个derived-only内部变形负例单列，不冒称合法JVM输入。原名头折叠/冗余iterator/overload包装独立欠账，CF11和71/612整体计数不增加。v1-v4 focused、collector/checker真实失败和所有私有草稿保留。

下一步：推送本候选以便远端CI与本地两fresh native并行；root独占共享/Git/Rust/JDK/CLI/网络，本地工具串行，不在Cargo运行中改源或跑网络。两seed分别5350648285461741569/1570，使用既有workspace_runner，默认stack/5GiB/target1GiB守卫及exact clean；native独立验收工具v2（全unit module名已核）在 `/private/tmp/jarde-typed-foreach-native-acceptance-luna-v2-20261018`。ownCI准备v2在 `/private/tmp/jarde-typed-foreach-own-ci-prep-luna-v2-20261018`，11源码/3fixture/50关键门；需root供实际产品SHA/runid，官方ZIP补证按实际缺失调用既有helper。产品push/native/自身CI此刻均未完成，勿提前勾3.2/3.3。

证据入口 `openspec/changes/recover-typed-list-foreach-elements/results/README.md`。交付后再写实际产品SHA/CI与干净main终态；全局goal不因叶子完成而complete。接续先核真实Git/进程状态。

## 2026-10-18 最新：typed foreach 最终矩阵已接受，收紧来源与计费后再交付

此节优先于下方历史记录。finThrow 5/5、nestedFin 16/16、父finally 7/7已交付，不重做；持续goal保持active。当前change `recover-typed-list-foreach-elements` 暂为4/7：d130候选的最终原始TestNestedLoops与19case控制矩阵已root独立接受；来源/计费修正已应用，task3.1待新CLI的20同argv API逐字重绑后重新勾选，broad-v2正在跑。

HEAD/main/origin最后已推86741afeb（规划），本地产品未提交。d130 CLI为已验收当前候选，原始生成36VM/288case、控制Jarde48VM/912case全部同原；JADX合法类型不匹配样例8格整类编译失败完整保留。控制12API的raw方法text/map/quality/fallbacks与旧产品逐字不变，raw/mismatch/ctor投影不变；typed正例只改头类型/首cast及既有emitter固定body缩进。完整原始黄金原名头尚未追平，CF11/71/612计数不增加。

广门v1为802/0/6，CI Clippy/MSRV通过、Reader census1301/5751/630/4050/14、fingerprint2429且原2427项不变，CLI与focused-v5同d130，target已exact clean。末审派生foreign来源/owner字节比较计费/new program_node_count预扫三点已由Luna补丁+root最小修正处理，正重跑最终门；旧AST clone计费helper的既有预扫独立记录不扩范围。两fresh native、最终新CLI对应Java复核、实际产品commit/push/自身CI均未完成。证据入口为本change results/README.md及final-root-acceptance-v1.json，各阶段真实失败保留。

root独占共享修改/Git/Rust/JDK/CLI/网络，Luna只私有准备/只读；至少5GiB余量、target1GiB每秒守卫/defaultstack/exact clean。先核工具终态、来源pins及真实Git状态，再接续。

# Jarde 接续入口

持续目标 active，在当前聊天继续。按本地 JADX **71 语法单元 / 612 测试文件**账本先完成大颗粒 MVP，再探索双方未覆盖的形态。局部通过不增加整体单元完成数。

先按下方第一节接续；后续章节是已完成产品的历史检查点，其中的“下一项”和提交号不代表当前状态。

## 2026-10-18 当前：typed List foreach 候选已定向验收，3/7

finThrow5/5、nestedFin16/16和父finally7/7已交付，勿重做。持续goal active。`recover-typed-list-foreach-elements` **3/7**：固定原始基线、私有Luna实现/root架构审查、公开正例与raw来源回归完成；完整最终三方矩阵、全部拒绝/停止、广门/两fresh native及自身产品CI仍待，不得声称产品交付或CF11/71/612完成。

已应用Luna私有实现v3（`/private/tmp/jarde-typed-list-foreach-impl-luna-v3-20261018/task-2.1-v3.patch`），root修正以完整Program节点计新遍历、保留cast全部来源/CP/成员、补类型复制计费。改动仅既有泛型事务/opaque AST和测试；不新增循环识别、公共序列化模型、IR/pass或作用域solver。原Assign/Declare、synthetic名字与外层声明保留；List<E>/physical E cast精确结合，不匹配仅跳过typed候选，原generic List声明不降级。原名头折叠、iterator和Object包装独立欠账。

root focused **v5实际终态exit0并独立接受**：fmt、496 Java库、17公开/相邻用例（1既有ignored）、3 hoisted iterator，共516通过/0失败/1忽略；完成CLI构建和exact clean。CLI `/private/tmp/jarde-typed-foreach-focused-root-v5-20261018/candidate-cli` **63386112B/SHA d1304ad2c3acca940a744eb3023bea0f77f62d279afc675aff84a6c076d447c4**；3190构建来源不变，target absent/defaultstack/5GiB/target1GiB守卫接受。独立收据 `/private/tmp/jarde-typed-foreach-focused-review-root-v1-20261018/review.json`。

前四focused失败保留：v1格式化未等终态导致test文件闭合变化且旧meter断言失败；v2新unit owned/ref类型错误；v3新增复制计费后旧固定meter值再次过时（真实late deadline已通过）；v4三个新AST测试借用了OutputBytes=0的旧helper。root保留原32_531限额，以真实Partial/原始完整text+map/无typed或generic局部发布为断言，不再用旧内部计数当契约；未调旧P5表/限额。新AST测试另给显式输出预算，生产计费保留。

19case控制基线已独立接受：137命令274raw，4原编译profile/8JVM152case、12Jarde API，Jarde24完整编译48JVM912case同原，24+4错误CodeSource拒绝。JADX两JDK8启动错误保留，6命令Java23补跑后四profile均成功反编译，但8次有效整类重编均因mismatch的Iterator<String>→Integer cast不兼容失败，无JADX runtime；旧四次无源javac不混算。证据 `results/controls-baseline-root-v1`（373成员77f87a…）及`controls-jadx-java23-supplement-root-v1`（24成员329b5a…）。完整原始NL基线20+75命令/三包307成员见results README，Seq2 raw方法map补核亦保留。

**当前root正在完整最终原始NL采集**：`/private/tmp/jarde-typed-foreach-original-final-root-v1-20261018/execution.json`，脚本prep-root-v3 SHA91a6c1acd2e3f3c5a1b2addfcbb12dec959367f87dd9c2ec0f82e6488f1ff088，只换上述CLI/Hash，driver4534…不变。先等真实process终态再审/跑对应完整生成矩阵。最终19case collector在 `/private/tmp/jarde-typed-foreach-controls-final-collector-root-v4-20261018/collect_controls.py`（cea527…），只修所有JADX启动使用pinned Java23，尚未执行；等原始NL及矩阵结束后串行运行，root审查不裁源。Rust/JDK/CLI/Git/root共享写入仍串行，agents只读/私有Python准备。

新增fixture `tests/fixtures/typed-list-foreach/` 的class已git add -f，byteSHA eb5e8b…与原javac8-debug相同。独立raw Code步进核1class/6Code/0handler/10targets/0jsr/97BCI，Reader人口元组已按delta改1301/5751/630/4050/14，完整Reader门尚待广验。此改动在focused CLI之后，仅cfg(test)人口断言，最终CLI须由广门再构建/核字节，不能混称focused从后改源码构建。

主线已推送的最近准备提交86741afeb，当前有未提交Rust/测试/证据；不要跳过完整门直接称clean产品。广门私有spec `/private/tmp/jarde-typed-foreach-broad-spec-root-v1-20261018.json`，复用既有guard，允许唯一运行内写fingerprint再生；先等全部Java采集结束、source closure稳定，再运行。后续两fresh native、产品commit/push和自身CI按tasks继续。

## 2026-10-18 当前：Seq2 完整对照接受，下一固定项 TestNestedLoops

持续goal active。finThrow5/5、nestedFin16/16、父finally7/7及后续Seq1/NL3/NL5已交付，勿重做。Seq2当前输入由既有产品 **ee013adb35711366d0aff2f774ee633b34c06082** 覆盖，无新增Rust/机制/OpenSpec实现任务；本次只更新证据与账本，不冒称新增产品CI或整个CF11/71/612完成。

root完整读工具/源码并实际执行、独立接受原始基线24命令48raw和完整生成源码矩阵111命令222raw。Jarde16完整选定子类编译/64隔离运行512case均同原；JADX2完整outer+child编译/8运行64case同原，另2个javac8框架major55/expected52边界无runtime。原始64case另计。default原{'a'}与configured128注入分开；UTF16/identity/异常类型稳定，异常消息raw不跨JDK硬比。18成功格各实际wrong CodeSource拒绝，20class/60Code完整双parser库存/access/BCI接受；driver构造器仅核存在、不称运行。8Jarde请求均3方法Structured/无fallback/全BCI/default-all正文与map一致。

CLI仍为aabf62fe373876d176e6001bf3ed1fd84f80d2169ff5e787c4095c5d253c63d9（63164416B）。3298来源等原构建3187不变文件加111已有exception-only证据/工具；CLI/依赖/JDK/来源前后不变，5GiB/target1GiB/defaultstack守卫接受，target不存在。首matrix driver-MUTF8解析失败保持1命令/0产品格/0runtime，独立保留；后续只收窄driver工具解析，不改目标完整校验。checker Path/表示/Namespace遮蔽和root草稿失败同样保留，最终root-v5实际accepted。

证据为 `openspec/evidence/java-syntax-2026-10-18/cf11-sequential-loops2-{baseline-root-v1,generated-matrix-root-v2,generated-matrix-failed-root-v1,review-tools-root-v1}`。四包391成员全部逐字复开，报告 `cf11-sequential-loops2-archive-review-root-v1.json`。完整范围、原始失败与最终review见各README/archive；历史下方“Seq2尚未执行”已过时。

**下一固定 TestNestedLoops**：root已读原始完整Java/黄金typed-foreach断言；历史接受是完整选定转写类，不冒称原始outer全编译。当前helper与永久测试已支持iterator声明/初始化分离和Elevated owner，旧BCI13/42不能当current首拒；需先核当前实际API输出，区分Object foreach+String cast与typed String header/Signature呈现。Luna已获派私有完整原始基线collector/standalone reflection driver准备，尚未执行；root审后fresh out运行，取真实首拒再写OpenSpec派Luna最小实现。root独占共享/Git/Cargo/JDK/JADX/CLI/网络，Luna仅私有准备/只读。不重做Seq1/NL3/NL5、不先新增solver/pass。

## 2026-10-18：Seq1 产品验收完成（历史检查点）

持续goal active，继续在当前聊天按71单元/612测试文件账本推进。finThrow5/5、nestedFin16/16和父finally7/7均已交付，勿重做。本次 `recover-statementful-loop-headers` **8/8**、`recover-field-write-test-copies` **5/5**；局部通过不增加CF11或71/612整体单元数。

产品 **ee013adb35711366d0aff2f774ee633b34c06082** 已推送main，自身[CI39100001965](https://github.com/LordCasser/jarde/actions/runs/39100001965)由root完整接受4jobs/52 API步骤成功。两CI seed各385目标/3754通过/0失败/98既有忽略、41精确关键门各一次。原始123采集+两次官方ZIP共125命令/250raw，22源码/95fixture绑定产品。证据 `openspec/changes/recover-statementful-loop-headers/results/own-product-ci-root-v1`。后续文档提交不冒称自身跑过该产品CI。

官方ZIP独立step10文件缺失，但官方aggregate含完整命令/executable/具名1通过/0失败。root保留原验收拒绝和两次同SHA下载，按9/11唯一payload边界、API时间及产品workflow核原始step10；timestamp/BOM只作对齐视图，raw不改。补证v2的assert优化模式缺口也保留，v3显式检查通过正例/13负控/真实python -O改名拒绝，再由root实际完整验收。两个重复proof行是同证书被两个既有parser读取，不是额外oracle运行。私有证据适配未改产品Rust或CI workflow。

本地两fresh native v3均已终态exit0并root独立接受：每轮385目标/101批/3754通过/0失败/98忽略，50关键门各一次，416raw/逐target库存与计数同，5140来源相同且未变。最新broad3187与native并集5147逐Git blob/bytes/mode绑定ee013，CLI不变。默认stack/5GiB余量/target1GiB/每秒guard/finally exact clean通过，当前target不存在。原seed v1/v2真实失败及两个旧候选取消CI均在results保留，不算成功轮。

最终CLI `/private/tmp/jarde-statementful-loop-copy-broad-gates-root-v1-20261018/candidate-cli`，63164416B / SHA256 **aabf62fe373876d176e6001bf3ed1fd84f80d2169ff5e787c4095c5d253c63d9**。广门798/0/6（Java493、Reader181、八P5 104/0/5及公开门），fmt/CI Clippy/MSRV1.88均通过。Reader census1300/5745/630/4040/14，标准指纹2427，原项不改。

root审Luna最小实现：默认Frame内精确natural-loop/single入口出口/线性body/latch与gateway来源证明，复用Endless+If.test_effects+Straight+Break，不新增public AST/Frame/solver/pass。字段dup_x1复用保存原值证明与同phi缓存刷新，volatile不重读。最后头部dup准入联合原region目标槽门和既有Builder完整local_assignment_at，只提升crate可见；所有新扫描提前计费/Stop原样传播/末尾一次claim，旧负控快照不改。真实Reader参数目标和MultiCopy拒绝；Seq1第二header23/dup26是明确direct proof-context正例，full cost277/limit86在新copy计费处停止未claim，不冒称普通pure路径调用它。范围与未登记的loop字段复制负例见results/copy-admission-scope.md。

最终56 CLI命令/112raw中44正向完整Java与全部递归非usage JSON等旧0bb版本，12旧复制控制default/all文本/方法质量/fallback/map/诊断一致。仅因同一Java字节复用既有四矩阵：132完整编译/130成功、260 rebuilt VM/2568case同原（Jarde1728/JADX840）；原JVM240case另计，两个旧JADX outer javac8 major55边界无runtime。没有手修/裁剪生成Java或冒称重跑JDK。sources/完整物理BCI/UTF8/default-all均见各root-review。

证据入口为父results/README.md，workspace-seed{1,2}-root-v3和workspace-acceptance-root-v3含完整checker/source binder；父/子32归档3428成员已按相邻capture-review及embedded manifest逐字复开，报告all-archives-root-review-v5.json；下一固定prep/静态3包21成员另计。关闭提交后真实HEAD/远端/干净main/14辅助checkout detached-clean-ancestor/target absent收据外置 `/private/tmp/jarde-statementful-loop-final-head-root-v1-20261018/execution.json`；接续时重核实际Git状态。本节交付文档为产品的后继记录；实现、双轮native和自身CI均已验收。

**下一固定 TestSequentialLoops2，尚未执行、尚无新缺陷/机制结论。** root已读完整上游源及黄金断言：两while/break/两<=127，debug另一个return c，不要求输出字面i--。lowercases原始初值只有{'a'}；default真实异常和configured128注入控制要分开。Jarde已有region.rs:12485 test_is_pure → test_expression_instruction → :12602 snapshot_condition_part证明postfix条件，先复用，不先认定StatementFree。新statementful helper的body只收线性路径，是否实际参与本例须取Reader首拒。

执行准备：`/private/tmp/jarde-seq2-baseline-prep-root-v5-20261018/collect_seq2.py --out fresh目录`，SHA d4d1db9ec3f93088e4cebeca11b406f0e54d35bc9e9e58e98a99ad1f76f2997c，仅将root审过v4绑定到新aabf CLI；算法/driver/parser不变。8-case/mode driver SHA21670301815003eff605bef518c710c033be1cae644c45365a870349aafe0175，fresh parentless loader/独立expected CodeSource/原{'a'}核验，异常消息保留raw但不跨JDK硬比。保留完整outer/child、真实javac8框架major55边界、javac23 debug-none与全部原始双VM；不裁源。完整物理parser c9f064…提供access，compare.parse_class不提供class access，不能混用。换机器恢复工具和依赖后重核全部pins。

未执行prep已持久化 `openspec/evidence/java-syntax-2026-10-18/cf11-sequential-loops2-baseline-prep-root-v2`（9成员/4acca7…），旧v1/v4准备也保留。JADX静态算法备忘 `cf11-sequential-loops2-jadx-static-review-root-v1`（4成员/e01a6a…）保存v1及v2纠正、20源码SHA实核：普通exit/body算法可参考；renderer和通用单用内联不能证明本输入旧栈值已重构为i--，要看原jar实际输出/IR，不照搬猜测。先完成当前收尾/确认干净main，再用fresh out执行冻结基线；实际发现差距后按OpenSpec派Luna私有实现，root做架构审查与完整对照。root独占共享/Git/Cargo/JDK/JADX/CLI/网络，Luna私有或只读。

## 2026-10-18 当前：TestNestedLoops5 已完成产品验收（8/8）

`recover-proved-loop-arm-returns` 产品 **e70b350ee33307294a3d383ffdea5874caf9f5b8** 已推送main，8/8关闭。自身[CI39087192303](https://github.com/LordCasser/jarde/actions/runs/39087192303)已root独立接受：4jobs/52步骤全成功、两seed各384目标/3736通过/0失败/98忽略，20重点门每seed各一次。53采集命令106raw及官方逐步骤ZIP命令2raw保留，gh UNKNOWN STEP显示限制未隐藏。后续文档关闭HEAD不冒称自身跑过该产品CI。

finThrow5/5、nestedFin16/16、父finally7/7以及后续exception-only、直接数组构造、switch和NL3均已交付，勿重复实现。原finThrow/nestedFin同一最终CLI完整对照576 rebuilt case与各自历史CI见下方finally节。持续目标active；本项不增加CF11或71/612整体完成数。

NL5生产只改loop_arm_join_source，复用同Frame/normal join已证明return叶、ForHeader/latch及canonical边闭合，精确natural∪returns所有权与物理gateway，Stop照常传播。implicit_tail_latch_origin保持旧产品逐字原样。实际原tail If/gateway34与A/B tail Straight/gateway96均有ForHeader，因此删除无证据需要的CompoundIf扩展；无新公共IR、pass或solver。

最终CLI `/private/tmp/jarde-loop-arm-gates-root-v4-20261018/candidate-cli`，63070624B/SHA256 **09dda40069dc362bc59f141a7d82c8cdcaee8b64072d304ec1ba848bdc14da25**。定向667通过出现/0失败；本地fmt/CI同款Clippy/MSRV1.88/reader/指纹/P5接受，旧2356指纹不变新增31，总2387。有效fixture22class、独立38Code/2handler/74targets/0jsr，Reader census1280/5677/626/3921/12一致。Reader变形和外来clone测试的proof-context限制明示，不冒称完整合法JVM恢复。

最终24API/48raw核56方法配置、24目标Structured/32非目标投影不变、全BCI/UTF8及12组default/all，Rust公开门核BLAKE3。完整生成Java矩阵277命令/554raw、36源格72重编：Jarde40及JADX20成功，12原始javac8框架major55边界；144运行中132匹配原oracle、372case，另12check8框架边界，无其他差异。60错误CodeSource拒绝；成功产物全部类树、方法/字段/access/Code独立重解析，未手修生成Java。

原matrix collector全部命令完成后oracle Path误作pin导致终态failed，原execution.json冻结。root另建绑定原失败SHA的finalized收据，复核全部未到达末尾不变量后，由独立reviewer v5接受实际矩阵；future collector v4未执行。早期Clippy、Homebrew MSRV入口、P5零测试、helper路径/JDK别名和草稿错误按各自真实范围保留。

两个fresh native seed已root执行完整checker接受：每轮384目标/101批/3736通过/0失败/98忽略，29关键门每seed各一次，416raw与逐target库存一致。构建3134与native5087来源的5094并集按Git blob/字节/mode绑定产品，CLI不变。继承collector把artifact顺序误作运行顺序的表保留，root按stderr Running重建；独立债务未混入产品。默认stack/5GiB余量/target1GiB/每秒守卫/finally exact clean通过，target absent。

证据索引 `openspec/changes/recover-proved-loop-arm-returns/results/README.md`；20归档2289成员已root逐字复开，compact报告all-archives-root-review-v3.json。原五基线729成员audit冻结不变。关闭后实际OpenSpec1.11.0 apply all_done/8-of-8、strict369/0。文档推送后的实际HEAD==origin/main==远端、clean、仅main、14辅助checkout detached/clean/ancestor、target absent与5094来源仍同的终态收据在 `/private/tmp/jarde-loop-arm-final-head-root-v1-20261018/execution.json`，接续核该收据与真实Git状态。

下一固定 **TestSequentialLoops（Seq1）**：已root独立接受原始debug/none完整基线24命令48raw、四VM六有限case同原oracle、major55/CodeSource边界；旧完整源码矩阵38命令76raw、12源格/24编译：Jarde16均缺return，JADX6编译成功并双VM12运行72case同原，另2完整outer javac8框架major55失败无runtime。额外物理checker核成功产物完整方法/字段/access/Code/精确类树。诊断为中文，早期root手动英文断言失败无语义收据；后续独立真实补核单列JSON，不修改冻结包。

当前e70 CLI已实际重放相同八API并由root独立reviewer接受：8格完整JSON递归只去usage/elapsed_millis后与旧版相等，16新raw/16旧raw、原argv仅换CLI、输入/来源/CLI/guard均核。**缺口仍存在，未实施Seq1修复**；不冒称重跑历史JDK矩阵。三独立CF11包225成员已逐字复开，索引 `openspec/evidence/java-syntax-2026-10-18/cf11-sequential-loops-archive-review-root-v1.json`。

下一步先实际Reader/SSA/Frame首拒诊断：旧首拒header2 Store5没有dup，第二header23 Store27前有dup26；静态memo `/private/tmp/jarde-testsequentialloops-next-direction-root-v2-20261018`。优先核现Endless/allow_own_loop_entry能否保持循环头赋值、body-first条件与精确break/return，不放宽StatementFree，不先新增AST/solver。JADX已冻结输出两个while(true)，本地makeEndlessLoop单exit/break和RegionStack算法已读；源码分析不冒称binary动态trace。Atlas Focus ticket未完成不引用其事实。真实SSA与完整owner闭合明确后再建OpenSpec并派Luna，root完成对抗审查及完整新旧JADX/源码验收。root独占共享编辑/Git/Cargo/JDK/JADX/CLI/网络，Luna仅私有准备/只读，不改冻结版本目录。

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
