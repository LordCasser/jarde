# Root 验收记录

当前是组合代码 WIP，尚未提交推送或通过本次确切代码 CI。

## 候选 focused

root-focused-report-v1 实际编译通过，但测试失败：包装 recover_for_class_source 不保留 AST；该次实际 ArrayCreation 正候选断言已通过，随后 .ast.expect 失败。只修测试改用既有 retain-all-AST 入口，没有另造 Program 或修改产品证明。原始失败 streams/result 保留。

root-focused-report-v2 实际成功，1 个测试内核对真实 javac8/javac23 class 的候选 allocation1/areturn38、来源与类型负控、共享预算在 allocation1 及 SSA lookup37 的停止、取消。随后添加 ragged 与多语句真实 Program 负控，root-all-java-lib-v1 再执行全部 325 个 Java lib 测试，325 passed/0 failed，覆盖新增断言以及原始 rank/primitive/ownership/handler 边界。2.1 已完成；声明投影、五源新 CLI、双真实 JDK 全24腿与后续门禁仍未验收，不能借旧四源 CLI 或 CI。

## 声明 focused

root-focused-projection-v1 实际编译失败：新 probe 将 ClassSourceRecovery 混作 RecoveryReport，三处字段/boxed 类型错误；只修私测 `.report` 字段后 root-focused-projection-v2 实际16/16通过（含真实同次候选的生产声明负控、Map两wildcard形状及逐实参Budget停止）。完整 direct 集成及五源CLI回放尚待执行，2.2不提前勾。原始失败与成功命令/streams均保留。

root-focused-integration-v1 实际5/5通过，明确 installed OpenJDK23重编两个冻结输入编译腿（不是两套真实JDK工具）。complete direct包含全部六类，collectionGrid实际泛型声明、无拒绝与raw正文通过；2.2已完成。两套真实JDK新CLI回放仍待3.1。

## 五源 CLI 与完整语义/Signature

新 CLI `/private/tmp/jarde-wildcard-array-return-cli-v1` SHA196bb3e1e1bd074bb851f363938951a6d2dea81d8e895cd67e1a2560b3c2f761，五源精确身份及build成功记录见results/candidate-cli-v1.json。raw complete candidate-root-v1完成102commands、384文件闭集，全24腿22成功；两direct完整六类均使用真实Corretto8/OpenJDK23、空CP/SP全部生成源码重编、runtime只新classes且-Xverify:all，与冻结原exit/raw stdout/stderr逐字一致。两个collectionGrid声明均为Collection<?>[][]，保留原raw Collection[][]初始化/ArrayList[]与HashSet[]、实际单次producer顺序、所有物理成员及来源。

root verifier v1实际失败是误把成功证据marker当refusal；产品真实marker是Signature投影成功说明，不能删除它。保留v1脚本/失败，v2要求该准确成功marker并排除refused/ordinary_generic_source_unproved，其他grid空marker断言不变。v2实际1166checks/0errors，状态semantic_and_wildcard_signature_passed、task_3_1_complete=true。还独立解析两原输入Main.class Signature及allocation类型/store/producer真实指令。3.1完成，并回验child3.1。

legacy22腿20/22：18矩阵16/18、factory2/2、direct2/2；新增数值两腿2/2。BigDecimal两已知失败仍保留，拒绝不计恢复。original/JADX流使用逐hash验证的历史记录，未fresh重跑；本次仅candidate新执行，分母未缩小。

本次MSRV1.88及CI-exact Clippy已实际exit0；fmt通过。P5/指纹/reader/显式Java和确切新代码CI仍待，不借旧产品CI；3.2/3.3不勾。平坦Signature真实泛型arity属于共享旧路径债务，另见results/flat-signature-arity-debt-v1.md；本片没有证明未知类元数，未宣称所有物理有效Signature均是Java source-valid。

## 本次本地收尾

root-local-acceptance-v1.json核对11门禁各实际exit0、准确argv、双流hash、当前五源+两集成测试身份不变：fmt、MSRV1.88、CI-exact Clippy、P5严格pins5pass/1ignored、fingerprint5pass/1ignored、reader census1pass、显式P3 Java3pass、functional constructor完整class1pass、两片OpenSpec strict与diff。既有严格pins/指纹无需变更。当地不是JDK25，不宣称本地执行JDK25 oracle。

全workspace双seed由当前组合代码确切CI验收，3.2/3.3仍未完成；前片本地全仓首次seed1被20GiB guard停止，不冒称成功，不借旧CI。所有raw失败和执行记录保留。main是唯一分支，14辅助worktree均detached/clean/main ancestor/无target，实际审计results/worktree-audit-before-commit-v1.json。
