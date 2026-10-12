# Jarde 接续入口

持续目标保持 active，用户要求继续在当前聊天推进。按固定 JADX **71 语法单元 / 612 测试文件**的账本完成大颗粒 MVP，再探索双方尚未覆盖的情况。组件测试通过不能直接增加整体完成数。

## 最近完成的 MVP

OpenSpec [recover-structured-list-generic-bodies](openspec/changes/recover-structured-list-generic-bodies/tasks.md) 当前 **9/9**。结构化 void 正文的物理 census、List 参数真实用途证明和泛型声明/调用正文事务已经实现；复用现有 AST/精确 argument edits，不新增 solver、pass 或公共 carrier。永久测试覆盖 String/Integer、inner option、null add、控制反例、三 profile、来源和停止。详见 [verification-root.md](openspec/changes/recover-structured-list-generic-bodies/verification-root.md)。

**本轮本地验收起点是 e9e3a0ec27e5bf7698bdf4817ff08d38bcb63374，交付提交以当前 Git 与下述自身 CI 记录为准。** 产品 `ca45c0ed2193fa8709eeba1196ef325f1804f63e` 已提交推送；自身 CI `38225064059` 已独立接受，3.3 闭合。已推送检查点 `70cd4b48410b118011b8be3d4411fd421f3e3dd2` 的干净主线审计已通过，3.4关闭。关闭文档推送后真实finalHEAD还会单独外置审计；检查点不是未来提交的证明。旧产品锚点 5f5fb2c6、旧 CI 38197592542 和 f12668ed0 的最终审计仅证明前一项，不能用于关闭当前项。

## 最新有效源码与本地证据

最终 Clippy 曾发现 report.rs 两处 iter_cloned_collect 和 class_source.rs 一个 unneeded_wildcard_pattern，root 已作等价写法修正。当前 report.rs SHA ecf8c4cb10199555ae756cc3754a8a7dfe0cbc4255e7acd6280f94aea51903fb；class_source.rs SHA 16e7a07f5f133f948a5d88ea467e397f9685a18e56cd8a4734792d6efbfaa1bb。失败与后续 Clippy 通过分别保留于当前 change 的 final-clippy-failure-root-v1 / clippy-preflight-root-v2。

最新 focused-root-v12 已实际通过八命令：API 7 passed/1 ignored、jarde-java 375 passed、reader census 1 passed、P5 fingerprint 5 passed/1 ignored、fmt/build/finally clean 0；4822 输入闭包前后相同。receipt SHA 74350dfc154c2fd77aa8a9636dd8d29b301eb119e167be56a0f18af011996a73。冻结 CLI `/private/tmp/jarde-cf11-structured-list-candidate-cli-root-v4`，61920032 bytes、0555、SHA d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f。归档见 [focused-census-and-registration-root-v12](openspec/changes/recover-structured-list-generic-bodies/results/focused-census-and-registration-root-v12/README.md)。旧 CLI/root-v11 证据对应旧来源，不覆盖当前源码。

## 已完成门禁与下一步

全工作区 **session18940 已 exit0**。fresh workspace-root-v3 正常 passed，195 命令全部 0，无 guard stop；两组 seed 各 96 批/364 targets/3502 passed/0 failed/98 ignored，4822 来源一致、finally clean 0、target absent。execution SHA 3c2b8d52b4f1927091ea273b2503e371f066efcf64e62c944c5cdfa4cd08a7ba；独立 acceptance SHA 56d4bf06a166e2074da8951ad37f66fc2df165c045983b17ad6c21657373a461。root 两个解析路径实际通过，归档见 [full-workspace-accepted-root-v3](openspec/changes/recover-structured-list-generic-bodies/results/full-workspace-accepted-root-v3/README.md)。

最终本地门禁 **session49761 已 exit0**：fmt/当前 CI Clippy/global strict357-0/CLI rebuild/exact clean 五命令0，新CLI字节恒同、4822来源一致、target absent，root独立 accept-final.py exit0。final receipt SHA 3cf03577eea64a7d19f1164cbc2da3d322686c7eb5d5baca3dd50f1e22286b04，acceptance SHA f91f3ab30fea420d4d39ba1662d1cb9ef229b64aaef4af9359ef3ae29d43ad0c；见 [最终门禁](openspec/changes/recover-structured-list-generic-bodies/results/final-validation-root-v2/README.md)。3.2 已关闭。

新 CLI v4 的整类对照 **session6599 已 exit0**，root 独立 checker 接受12份/352命令/704raw/80成功运行对/24控制编译失败无runtime/41 oracle labels。新 CLI 精确停止两调用实际4/0；collector因方法name读取层级错误failed，root独立接受其完整raw且保留原失败状态。见 [整类及停止归档](openspec/changes/recover-structured-list-generic-bodies/results/final-comparison-cli-v4-root-v1/README.md)。3.1/3.2均已关闭。自身 CI 已实际接受4jobs/52steps、双seed各364targets/3502-0-98与全部具名结果，4822来源及CLI字节同产品；见 [自身CI](openspec/changes/recover-structured-list-generic-bodies/results/own-product-ci-ca45-root-v1/README.md)。已推送文档检查点的only main/14aux detached/全干净无target/主线祖先/4822来源实审通过；见 [交付检查点](openspec/changes/recover-structured-list-generic-bodies/results/clean-main-checkpoint-root-v1/README.md)。关闭文档推送后，再对真实finalHEAD作外置审计。旧矩阵空间失败没有拼接，71/612和整CF11完成数不增。

## 工具和资源约束

root 独占共享 Git/Cargo/rustfmt/JDK/JADX/CLI；Luna 仅只读或私有准备，不改共享源码，不运行工具链，不创建分支/worktree。外部命令最低 5 GiB 可用空间、target 上限 1 GiB、1 秒进程组守卫、default Rust stack、最终 exact cargo clean。现有构建仍运行时不得清 target 或并行运行其他工具链。/tmp 清理只移除可再生成缓存或做保留字节的历史去重，当前依赖与独有样本保留。

## 后续大颗粒方向

循环体 try/catch 当前 **2/8**，OpenSpec 实际8个checkbox；此前1/9为计数误记，要求没有减少，尚无产品实现验收。当前CLI固定DF/DV完整基线已独立接受56命令/112raw：debug/none × Java8/23原18条oracle同值，JADX default经独立Runner包适配后与none共8个runtime均复现loopCatch错误异常逃逸；4条Jarde整类编译失败/no-runtime准确保留。见 [基线](openspec/changes/recover-loop-body-try-catch/results/baseline-cli-v4-root-v1/README.md)。

固定四方法的同次IR真实四测试均1/0/0，最后clean0、4822源码按字节恢复、targetabsent；root独立接受16个精确physical方法观察，见 [instrumentation.md](openspec/changes/recover-loop-body-try-catch/instrumentation.md)。for/while与loopCatch的终止throw分支不在普通自然环且普通路径不达join，现有catch证明拒绝，handler→join形成第二SCC入口。foreach现有区域证明已通过，另卡在try外aload0@2/astore2@3引用producer的呈现门。优先分别补现有所有权与SSA证明，不能删CFG边强制归约或放宽fragmented特化条件。

无限retry/break两变体已实际独立接受88命令/176raw：原与JADX完整类运行同五个计数/事件oracle；Jarde test()I首拒block0的ownership overlap、整类compile1/no-runtime。v3固定事件文字消除v2的exc nested StringBuilder干扰，但test拒绝相同。八lane同次IR真实9命令/18raw已root独立接受；普通图无环，catch回header重入同一个根Try，block0/handler owners[0,0]。4822源码恢复、clean0、targetabsent。见 [重试归档](openspec/changes/recover-loop-body-try-catch/results/retry-baseline-and-ir-root-v1/README.md)。

[design.md](openspec/changes/recover-loop-body-try-catch/design.md) 已完成并严格validate通过/applyready。方案为同一次finite terminal-body事实供join/scope/coverage/induction、foreach实际引用Load原位呈现、retry复用Endless/Try/Break/Continue并区分protected exit与catch header边界。不能把Try.normal_exit_bci=None误认成guard join=None；候选须用真实Transfer/唯一successor互证。任务1.2已关闭，三个Luna私有候选准备中：`jarde-loop-finite-ownership-impl-luna-v1`、`jarde-loop-retry-ownership-impl-luna-v1`、`jarde-loop-reference-load-impl-luna-v1`（均在/private/tmp，尚未应用或构建）。root串行合并验收。固定DF.loopFinally当前handler51 uncovered尚需独立处理，完整DF门继续开放，禁止删方法/改生成源码。

Structured List真实finalHEAD `9dc64e4df464b157f430bae63c74cbb2c72d195e` 已完成外置clean-main实际审计，`/private/tmp/jarde-structured-list-clean-main-audit-final-root-v1/execution.json` accepted_clean_main；该审计只覆盖该HEAD。后续仅循环设计/证据文档变化不替代新产品CI；新产品修改后必须重新完成门禁。

历史验收与架构债务分别保留在各 change 的 results 和 [completion-review.md](openspec/completion-review.md)，不能混入当前产品改动扩大范围。
