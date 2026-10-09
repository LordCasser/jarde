# Root 验收 — covariant child-array initializers

规划4/4、任务2/7，1.1确切前片CI及1.2历史基线身份核验完成；CI37968418985实际四job/48steps全部success，前置身份原始JSON和root记录见前片results/ci-code-sha-v1.json及ci-code-sha-root-verification-v1.json。doccommit92e12704a已推送且工作区确认干净后派发Luna限定实现，当前build.rs/集成测试WIP，尚未运行本片门禁，任务仍2/7。

## Frozen Historical Baseline

results/baseline-reference-audit-v1.json/.md来自Luna只读审计；root用独立verify-baseline-root-v2.py重算输入、完整source、原始双流、archive/manifest闭集与jar类内容，2503checks/0errors。结果见baseline-root-verification-v2.json；初版relative stream路径未解析的真实失败脚本与捕获复验保留root-baseline-verifier-attempt-v1，未改旧证据。

三份历史manifest闭集分别304、110、256文件；JADX archive213普通文件、211payload加两个manifest。当前direct全部六类双真实编译器输入，与JADX四profile及numeric CLI输入的class bytes一致，原始Java源亦匹配。zip封装hash可因历史metadata不同而不同，不以jar封装hash相等替代类内容身份。numeric与更早EM18 Jarde的Main.java输出不同，保持两份独立CLI结果；其余五类源一致，不能混成一次运行。

旧Jarde两个完整六源集均compile exit1，无candidate runtime；原始程序两腿exit0，raw stdout/stderr保存于pathfix logs并与JADX provenance hash相符。JADX四个direct profile均compile/run0，但只有rename-flags-none两腿raw双流匹配，default两腿语义不匹配，成功2/4。原程序/JADX/numeric旧Jarde均为已核验的历史运行，本片没有fresh三方结果。

物理三grid全部六份javap指令listing见physical-grid-listings-root-v1.json，准确parent stores为number20/38、collection20/37、own24/45，child stores分别19/37、19/36、23/44。原new/dup/init与所有array操作都保留在listing，不执行或修改故意变体。

root已核对numeric冻结CLI和当前8f624ea64四个产品/构建文件身份（前片worktree-audit-after-local-acceptance-v2.json）；采用当前基线等价的历史reuse，避免重复已冻结执行。Luna所述后续rerun需求用于新candidate验收：产品变化后必须fresh生成全部六类source、空CP/SP重编并-Xverify:all运行，不能以本次hash审计代替。

## Architecture Boundaries

对抗规划审计见results/adversarial-plan-audit-v1.md，root审读实际child branch、Builder与facade调用；移除仅类型相等早截断，保留NewArray shape、真实ValueId/唯一parent store、区间、深度、共同commit与meter。snapshot proof收集的旧Stop转换策略另记results/snapshot-proof-stop-debt-v1.md，不借空facts的类型fallback推断精确停止传播；真实结构/正文Stop单独验收。

只读变体构造见results/child-control-construction-plan-v1.md。extra dup只证明非认可reader，不把一读两distinct SSA outputs写成同一ValueId两uses。root验收verifier-only加载时还须显式link/resolve并查询方法元数据，且用一个已知无效ireturn控制确认启动器实际触发Code验证；仅defineClass成功不足以算完整JVM验证。启动器不初始化或调用变体方法，所有控制目前尚未实测。

## Preparatory JVM Controls

上段记录的是规划阶段。后续勘误保留旧文件与hash：effect插入长度为5字节；Java8启动器改用Paths.get，全部默认包类由同一parent-null URLClassLoader定义，非初始化加载后resolveClass/getDeclaredMethods。见results/child-control-plan-correction-v1.md及child-control-plan-corrections-v2.md。

root审读prepare-child-jvm-controls-v1.py与VerifyOnly.java后实际执行，结果闭集见results/root-jvm-controls-v1/manifest.json。两真实JDK各包含原始、父index交换、dup/pop、独立effect、Object[] child及invalid return六个完整六class变体；18条实际命令的argv/exit/raw双流及hash保留。10条有效类加载检查均成功，两个areturn→ireturn自检的原始VerifyError均准确指向Main.ownGridDirect @46: ireturn，证明方法Code验证真实发生。没有初始化或调用Main/目标方法，也未运行Jarde候选，不把这些控制计为恢复成功。

root重构每个完整class修改并逐字核验，companions保持原字节，闭集114files及所有命令双流核验通过，363checks/0errors，见results/root-jvm-controls-verification-v1.json（SHA e9bd6603df55b59c148c4359a6db53fa9326459ee06b8f2993cc7bb3edbc6fbb）。结构拒绝、Builder拒绝及新产品正文仍需实现后另测；控制准备不完成实现任务；前置CI验收后任务为2/7。

## Focused root acceptance

2.1/2.2已完成，当前4/7。root-focused-builder-v1真实编译失败（Result断言需Debug/PartialEq、Budget API错误）保留；修后v2两测试通过。root-focused-integration-v1/v2分别发现测试导入/迭代器错误及JAVA_8导入层级错误，日志保留；root按实际公开pass出口修正，v3五测试全通过。root-focused-all-java-lib-v1全部324测试通过，包含既有primitive opcode/component、rank/platform与exact proof helpers。未修改生产类型或放宽gate来迎合测试。

真实两套frozen class结构断言：数组allocations1/7/28，parent stores24/45，最终consumer46；构造Sites12/33与父子owned互斥且一次提交。index交换、dup/pop、独立effect均使整个array/Sites计划为空；Object[] child结构成立但完整报告因Object[]→Base[]拒绝；missing Mid及method-only保持整body fallback。实际Builder正确两store proof生成非ragged单return；empty、错BCI/source/target均ragged。共享IrItems在child scan31及最后build push46准确Stop；结构预取消及合法计划准备后build取消不返回半Program。不宣称render_value每个递归节点均有poll，也不借snapshot空facts宣称原始Stop透传。

完整集成已检查三个grid的实际类型、new/call次数与原顺序、allocation/store/constructor所有来源。collectionGrid在完整正文恢复后原Signature门槛通过，无新增独立泛型机制。本集成用实际安装JDK23重编两套冻结输入，真正按各自JDK8/JDK23进行的完整CLI回放尚待3.1；不借前片CI。

## Complete semantic replay and independent Signature finding

root候选CLI已实际构建并冻结为/private/tmp/jarde-child-array-covariance-cli-v1，SHA 27e53f610f1915c16c21fb144d63aebafb7d376c3eff349b6b5de7673a313123。四源身份见results/candidate-cli-v1.json。原runner v1/v2/v3均在任何候选进程前因历史addendum labels/numeric schema读取错误退出，未计成功，原脚本和raw logs全部保留。v4实际全24腿22成功：旧22腿20/22（18矩阵16/18、factory2/2、direct2/2），数值2/2；BigDecimal两腿仍失败且保留全部源/报告/编译与执行结果。原/JADX过程未fresh运行，候选生成/空CP/SP重编/-Xverify:all runtime为fresh。

root独立verify-candidate-root-v1在collectionGrid marker非空处真实失败，发现Signature确有独立拒绝，不能勾3.1。v2不是去掉该门槛验收：明确以semantic_replay_passed_signature_pending状态核验全部raw文件闭集、class字节、完整fields/methods(name/descriptor/access)、生成源、编译全部源及runtime原始exit/bytes、三grid类型/来源/次数，1116checks，task_3_1_complete=false，保留该marker精确负事实。静态根因见results/collection-signature-architecture-audit-v1.md，独立change recover-reifiable-wildcard-array-return-signatures按窄泛型返回证据处理，不扩大本片array ownership规则。root此前称Signature通过是误判，现以完整报告更正；当前仍4/7。

root CI-exact Clippy与MSRV1.88实际全部通过，但仅针对四源子数组版本，不借给后续generic版本。fresh partitioned workspace v1首lib/seed1被20GiB保护中止exit-15，任何目标的完整双seed不能报通过；过程/部分输出保留。root实际cargo clean5945files/818.3MiB，target不存在；17个较早Mach-O frozen CLI可恢复gzip归档并逐字验证，saved_bytes932116990，保留近期五个CLI。归档manifest及restore方式见results/historical-cli-compression-v1.json，不删除失败/证据。外部活动持续用盘，当前本仓已无编译残留，不降低20GiB线；后续组合代码全仓双seed由确切新CI覆盖，明确不是本地成功。

独立泛型片五源CLI fresh24腿22成功，1166checks/0errors已同时闭合完整direct语义和Collection<?>[][]声明，child3.1回验完成；成功证据marker保留而拒绝消失。确切组合代码全仓CI及3.2/3.3仍待，详见../recover-reifiable-wildcard-array-return-signatures/verification-root.md。
