# HANDOFF — jarde 主线接续入口

用户明确要求继续在当前聊天推进。持续目标active，按JADX **71单元/612测试文件**逐片追平，再探索额外场景。root负责架构、OpenSpec、真实对照及对抗验收；确定性patch/script交给Luna。窄片成功不改分母、不冒增整单元完成数。

## 当前状态

静态基线检查点为6fd51a18dd83d980f2f060246c209fb6fb0afba1，已推送并验收。本文件随新的实例数组产品检查点提交；实例产品只有report.rs/facade.rs变化，整数数组常量名产品尚未应用。最新main提交与自己的CI应以git log/gh run list核实，不能把静态基线写成新实例产品身份。

非final静态阶段片已**7/7**：CI38019682442全部成功，root实际verify-ci-product-root-v12接受4jobs/52steps，两seed各354结果记录、3357 passed/0 failed/97 ignored；Temurin25.0.4+7所有完整类对照、MSRV/fuzz/supply通过。完整API/stable/supply及接受文件在preserve-nonfinal-static-initializer-phase/results/ci-product-v5。Unicode两条声明期望修复逐字核Git blob；30命令/6运行腿whole-family对照及独立verifier v4来源核对已接受。

实例共同数组前缀片当前**6/8**：全部direct-super ctor相同连续primitive数组prefix提升；保留super(args)、suffix、fresh分配，不建constructor graph。private v6加root真实编译修正，Java层5/5、facade6/6，CI同范围Clippy、静态6/6/接口4/4回归、reader178/178、fingerprint5/5通过。冻结CLI/meta与完整证据见recover-common-instance-array-initializers/verification-root.md。实际候选58命令/52cases：48渲染、4整类重编、6运行全部同原raw；独立v6接受361闭合文件，32反例完整text恒同。旧literal/ordered24命令/8完整腿也全部通过，root独立接受97闭合文件。自己的新产品CI尚待提交后验收。

## 立即接续

1. 实例产品独立提交推送后核确切新CI，保存完整日志并独立验收，才完成任务4.1/4.2；不能借静态片CI。
2. recover-int-array-constant-names规划4/4、tasks2/8。root已完整审private v1和v2/v3/v4 deltas，v4真实git apply --check通过，但**未应用/编译**。只处理普通方法直接返回一维int数组中的直接typed Integer叶→唯一同类ConstantValue字段名，复用原候选/AST/emitter replay/writer，不动旧switch行为或引入新机制。静态片CI前置已满足，待实例独立检查点后顺序应用。
3. int新五目标类+Runner已准备在results/controls-prepared-v1；prepare-candidate-luna-v1.py由Luna准备，root**尚未全文审读/执行**。需完整源双JDK原程序oracle、新JADX default/none整类对照、新Jarde完整类重编/raw以及确切token Field/MethodPoint origins核对。不得从准备artifact推断已通过。

## 已有里程碑

- returned int数组+=7/7：6ddc77d5cc2a6fcf098aad40998f6d2602731c59，CI38001720578，4jobs/52steps，两seed3353/0/97；新2腿/425checks、旧nested4腿/479checks及旧24腿/1231checks已独立验收。
- CF16本change11/11：acd55c213ac7c670c6e76609871281a341d03ad0，CI38007755097，双seed3355/0/97；85项patterns明确8MiB栈。ImplicitCleanup、Normal完整类运行通过；Completion失败和N33有界拒绝如实保留，不宣称整CF16追平或默认小栈安全。
- nested int更新、BigDecimal→Number各7/7：确切561组合CI37994276707；子数组协变7/7、reifiable wildcard数组返回6/6、构造primitive转换7/7有既有完整验收。
- TestArrayInit.test2已覆盖、TestArrays2.test4真实25命令/8腿全通过，不重复实现。ArrayFill三边界39命令/8腿205闭合文件全成功；long limits/dependent stores已有语义，CONST_INT名称呈现是下一个实际差距。

## 对照原则与真实失败

全部生成源码原样完整重编，empty CP/SP、fresh classes、-Xverify:all；只允许ownRunner必要package适配。逐字比exit/stdout/stderr，不借原class/helper、不删成员、不手改正文。方法物理report/text/source_map保留；evidence请求和预算usage不要求与另一profile逐字段相等。

实例DifferentRhsByteArray的四JADX1.5.6腿编译成功而运行失败（ctor(int) mark32误呈mark31），raw在旧基线；本地dev soft isSame仅是漏洞候选，未证发布二进制具体内部调用。Jarde不同RHS始终拒绝提升。no-clinit基线collector因javap8缺摘要误标失败，root独立v2实际1053 checks通过，原失败不改。验收脚本历史格式/路径/变量错误的版本和真实raw保留；准确接受以verification-root所列实际版本为准。

静态片7196/25c5/2daa/41b8曾依次暴露旧enum断言、A1 golden、A1新增expected未登记fingerprint和Unicode旧裸字段期望；完整raw保留，最终6fd自己的CI接受。2062 corpus文件原2061恒同，唯一新增A1 expected2210 bytes正式登记。

## 资源、工作树及独立债务

只有main；14辅助worktree均detached、干净、main祖先、无target/剩余待合入，保护副本保留，无分支占用。root专有Git/Cargo/rustfmt/JDK/JADX/CLI串行执行，Luna只读或private patch/script。

持续20GiB机器余量/1GiB本仓target守卫，不降低线强跑；截至03:58 UTCtarget峰值904167534 bytes，04:00 UTC已仅cargo clean本仓target，删除4989 files/864.1MiB，target不存在；准确记录results/root-clean-v1。其它进程实时耗盘，Rust前需重新核资源。保留所有源/class/raw、canonical与冻结CLI。实例CLI /private/tmp/jarde-instance-array-cli-v1（SHA5abb4bc...）；静态CLI /private/tmp/jarde-nonfinal-static-cli-v1（SHAdda51122...）；旧returnedCLI /private/tmp/jarde-returned-array-cli-v2（SHA71f0a864...），完整hash以metadata为准。

独立债务：Arithmetic producer iadd@5来源缺失、flat Signature generic arity、receiver-tail预算、CF16默认小栈、handler两ctor双JDK goto@19来源缺口。只记录拆分，不混入当前实现。
