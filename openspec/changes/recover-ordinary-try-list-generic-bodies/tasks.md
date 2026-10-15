## 1. Baseline and proof integration

- [x] 1.1 固定产品597010的双JDK/debug-none完整17方法与44运行行基线、每份27命令/54raw及JADX整类失败；root实际独立审计、逐成员复开归档并记录纯goto映射缺口后关闭，不把静态分析、工具准备或no-runtime当通过。
- [x] 1.2 在同次候选内复用普通named row、完整prefix+Loop及owner事实，绑定单条typed row的ordinal/type/handler/range与实际Try/canonical/code；永久测试真实进入新路径后核正确body/catch/continuation及缺失/重复owner、同handler不同range、catch-all、多range、同handler/range的multi-catch、nested Try/Guard反例，root审读且实际通过后关闭。
- [x] 1.3 把该私有准入接入既有完整void用途扫描，整个Code/operands/SSA/effects/Operations一一配对、逐effect handler归属核验，包含catch读写；保留no-handler路径及参数writes/alias/unique receiver门。永久测试库存差异、异常row差异、catch重赋值/未知消费、实际新扫描late-budget/取消且无部分candidate，root审读和实际测试后关闭。
- [x] 1.4 沿既有List源级证明/签名擦除/overlay完成统一精确E投影，至少String与Integer的分支/null、remove(Object)前缀和已恢复nestedforeach；default/all相同，add合成包装可删但物理next-checkcast/removeObject重载及来源保持。永久公开测试正例、一个合格try方法改成员名及formal/local名字仍准入、unknowncall/rawalias/wildcard/indexOverload/重赋值旧控制和overlay晚Stop通过后关闭，不硬编码fixture或新增公共carrier/solver/pass。

## 2. Whole class and delivery

- [x] 2.1 在实际最终候选重新跑双JDK/debug-none/default-all未手修完整17方法类和至少45场景Runner（原44行完整保留，加可变output的handled正常完成行）；五个目标方法的泛型投影成功，unknowncall及旧反例继续拒绝，原17行oracle不变，String/Integer的iterator/hasNext/next/output.add RuntimeException/Error消息/escape/events/effects逐字同原；JADX每JDK使用自身input，真实失败保留no-runtime。root独立核raw/source/report/class身份/来源并复开归档后关闭。
- [x] 2.2 fmt/当前CI Clippy/global strict、必要focused及fresh双seed workspace、5GiB/target1GiB/1秒进程组守卫/default stack/finally exact cargo clean实际通过；root提交推送确切产品并接受自身CI，更新handoff/完成审查与账本，核最终真实HEAD clean-main后关闭。本项通过不自动增加整CF11/71单元/612测试计数。

当前实现、最终修复CLI的完整矩阵及交付均已验收。整数carry追加反例已修复并重新绑定矩阵、双seed与自身CI；root负责应用、工具链和最终真实HEAD复核。

2026-10-13：1.1按[实际基线归档](results/baseline-product-597010-root-v1/README.md)关闭；root重放auditor-v2及独立复开276成员/108基线raw/134总raw绑定。第一次复开因排除全部jar的root断言误拦受控输入；第二次逐jar核仅含一个完整原class、与同模式同JDK输入逐字一致，安装二进制仍全部排除，原失败保留。44行整类运行同原，五个目标仍raw签名，unknowncall保留拒绝。当前1/6；对抗审查补handled正常完成行（候选至少45行）、multi-catch与合格try改名门，其余任务未实施，计数不变。

2026-10-13：root另实际执行auditor-v4接受[45行补充基线](results/baseline-45-supplement-product-597010-root-v1/README.md)（errors空），逐成员复开251成员/118raw绑定；两mode/JDK原44行逐字完整保留，新增handled可变output正常完成/effects=2。冻结产品五个目标仍raw签名，当前仍1/6，实现任务未关闭。

2026-10-14：按 [implementation focused 与补充整类对照](results/implementation-focused-and-supplement-root-v1/README.md) 关闭 1.2–1.4。root 实际验收 442/0/1、CI 固定版本 strict 与 Clippy、typed-row/catch/census/lateStop 门，并独立核完整17方法在 JDK25、JDK8 none、JAR/WAR 入口的未手修源码与45行运行（8 cells/136 maps）。211成员逐一复开。任务2.1的最终双JDK/JADX矩阵与2.2的fresh workspace/自身CI交付仍开放，当前4/6，能力计数不变。

2026-10-14：按[最终matrix与forward时序root验收](results/final-matrix-and-forward-order-root-v2/README.md)关闭2.1。最终CLI v11，16个Jarde完整类cells全部compile/runtime同45/33行原oracle，真实JADX8失败/8成功分类均保留；root主审计accepted/errors空，逐一复开665成员。补充类consume缺口由独立forward change修复，没有改输入/Runner。本项当前5/6；fresh双seed、额外CI runtime与自身CI/clean-main交付仍待2.2。

2026-10-14 root workspace补充：fresh workspace-forward-v5在第70批捕获ICN.armCall既有whole-refusal回归（exit101），原断言保留，finally exact cargo clean exit0/target absent。旧focused-v11及matrix-v2仅证明其冻结源码，不能代替修复后的最终验收。carry调整为完整terminal/join/dependency证明成功后提交，正在focused-v12重验；双seed及自身CI仍开放。

2026-10-14 root 修复版冻结验证：focused-v13 451/0/2、CLI SHA0a30cf2cef1974006105e141789d5a9a8116497fae12d20479b2644d3a0ad2d7；matrix-v3主审计accepted/errors0，16完整Jarde cells及45/33行oracle。forward-order-v3独立root审计v2 accepted/errors0、64方法map/四cells12行；javac23仅保留原有source/target8的四条options警告，运行stderr空（首次审计误要求编译stderr空的rejected raw也保留）。BI v6/v13两编译器及历史class/default-all共12次render正文/quality/map/身份逐字一致；v3审计误用schema键失败保留，v4按真实outcome.report复核accepted。fresh workspace-forward-v6运行中，不先关闭3.2/最终交付。

2026-10-14 workspace-v6：原ICN批次已通过；第93批reader遍历所有fixture成功后，仅旧population tuple失败。实测新值(1169,5225,521,3429,10)，增量4class/62 Code/18handler/145branch/0jsr。保留该失败raw与exactclean；按新增冻结fixture精确更新测试内tuple/注释，未修改parser生产代码或跳过样本。focused-v14新增完整reader lib gate，最终双seed重新绑定该测试版本。

2026-10-14 root：最终本地归档final-local-delivery-root-v3已经接受，1642 payload/9编译来源逐成员复开，focused-v14 629/0/2、fresh双seed7124/0/196、全部额外本地CI门通过。2.2仍待确切产品提交/push/自身实际CI与最终clean-main，不以本地hybrid工具链代证。

2026-10-14：f014已提交推送，但新完整工厂receiver carry对照揭示可编译错码，当前交付未接受。forward change 2.2/3.1/3.2已重开；新focused-v17独立协议接受636/0/3，4882source closure不变，双JDK/default-all完整10方法dependency类四cells/six行oracle及40maps接受。原45/33行和16方法时序probe未改，新冻结CLI矩阵与双seed尚待验收；2.2及总体计数不关闭。

2026-10-15：root接受并复开[最终本地修复归档](results/final-local-delivery-root-v4/README.md)3536 payload/十编译来源，focused642/0/4、fresh双seed7130/0/196、额外CI门均通过；三旧P5精确费用按真实新增扫描重测更新，原比较保留。2.2仅剩确切修复产品提交/push/自身CI与最终clean-main，不扩大总体计数。

2026-10-15：修复产品644882d6db69f7d5b0bc1165c035a12b4f7cd8aa已提交推送main，[自身CI与clean-primary证据](results/own-ci-644882-and-clean-main-root-v1/README.md)根实际742项审查接受；4jobs/52steps成功，两seed各365suites、3565通过/0失败/98忽略，55具名门每seed一次通过。primary HEAD/remote main一致、Git clean、无target，4886编译来源逐字同最终冻结版本。关闭既定局部交付，文档提交后再核最终真实HEAD；不增加CF11/CF18/71/612总体计数。活动core候选、新问题与后续规格归档另行处理。
