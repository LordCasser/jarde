# HANDOFF — jarde 主线接续入口

用户明确要求「继续在当前聊天推进」。持续目标 active；root 负责架构、OpenSpec、真实 Java/JADX 对照及独立对抗验收，确定性 patch/script 交给 Luna。基于 JADX **71单元/612测试文件**逐片追平，优先大颗粒MVP，再探索额外场景。分母与整单元状态不得由窄片成功冒增。

## 当前状态与立即下一步

上一已推送检查点为 **2daa21c99db7e126d91b5ccbe3d016208fd54aba**；本次仅修复A1语料指纹登记并保存完整审查证据，修复提交的确切CI待验。非final静态产品逻辑仍是7196的一项flags修正；随后只修两个陈旧测试预期，生产10pins/冻结CLI不变。7196的CI38010503464失败于ordinary static旧断言，25c5的CI38012927864失败于A1旧golden；两次真实失败raw保留。A1新expected来自实际CLI完整输出，历史baseline不改；本机文件6/6与相关Clippy通过。

[确切CI38014660202](https://github.com/LordCasser/jarde/actions/runs/38014660202)已失败：stable seed1到P5 corpus指纹只发现新增A1 expected未登记，A1六项已通过，MSRV/supply/fuzz成功。完整API/stable日志保留ci-2daa-failure-v1。root实际独立BLAKE3核全部2062 corpus文件，原2061恒同、唯一新增登记；仅fingerprint JSON增加一条2210byte entry，classification与原所有pins不变（register-a1-fingerprint-root-v1.json）。本机低于20GiB，官方Rustgenerator/P5此次未运行；下一修复提交自己的完整CI待验，本片仍 **6/7**。v5准备版未完整执行；v6准备版遗留旧fingerprint等值断言，root拒绝使用；v7只保留reader旧pin，额外严格核指纹唯一登记，不改冻结metadata，尚未完整执行。

下一 EM18 实例数组完整基线已实际完成，见 [README](openspec/evidence/java-syntax-2026-10-10/instance-field-init-next/README.md) 和 results/baseline-root-verification-v7.json：三类+Runner共 **31命令/8腿**，原双JDK **2/2**、当前Jarde **2/2** raw匹配；fresh JADX1.5.6 default/none从同一javac23 jar各提取一次，完整源码各双JDK重编 **4/4编译成功、0/4语义成功**。DifferentRhsByteArray的ctor(int)把mark32误改mark31。失败raw必须保留；不能说8/8通过。

CommonDirectSuperByteArray的共同初值JADX提升而Jarde保留；ThisDelegatingByteArray两者都保留target ctor赋值且恰一次。第一MVP只恢复全部direct-super构造器共同连续数组初始化前缀，this链/参数依赖/不同RHS/不安全顺序先保留；不先建constructor graph。本地JADX dev checkout的soft isSame是可信问题候选，不等于已证1.5.6内部调用路径。root已核InsnNode/InvokeNode/ExtractFieldInit原代码。设计接缝见 [instance-promotion-architecture-root](openspec/evidence/java-syntax-2026-10-10/array-field-initializers/instance-promotion-architecture-root.md)；已建立recover-common-instance-array-initializers规划4/4、tasks2/8。Luna准备中的adapter草稿因丢super/字符串RHS比较/缺预算与field census被root拒绝，尚未应用，实例生产未改。

## 已独立验收的里程碑

- returned int数组+= **7/7**：6ddc77d5cc2a6fcf098aad40998f6d2602731c59，[CI38001720578](https://github.com/LordCasser/jarde/actions/runs/38001720578) 4job/52steps，双seed各354records/3353passed/0failed/97ignored。新完整类2/2/425checks、旧nested4/4/479checks、旧24腿24/24/1231checks。见 recover-returned-int-array-compound-updates/verification-root.md。
- CF16本change **11/11**：检查点 **acd55c213ac7c670c6e76609871281a341d03ad0**，[CI38007755097](https://github.com/LordCasser/jarde/actions/runs/38007755097) 4job/52steps成功；双seed各354records/**3355passed/0failed/97ignored**，p3_patterns85/85、nested_monitor4/4、p3_java_recovery54/54；真Temurin25/全部门禁通过。完整API/stable/supply raw在 results/ci-checkpoint-v1，root实际verify-ci-checkpoint-root-v3.py接受，结果ci-checkpoint-root-acceptance-v3.json。
- CF16 ImplicitCleanup四完整腿16路径/root167checks，返回覆盖trace29，历史JADX重编trace299不是oracle。N33在BCI450 jre_recursion_bound/NotProduced/空正文，root204checks；永久双fixture测试85/85明确8MiB线程栈，历史默认小栈溢出保留。Normal2/2完整执行，Completion2/2完整编译拒绝、JADX历史566/8错误保留，root536checks。浅层nested-loop+saved-int-return仍fallback，另列；不冒称整CF16追平或生产小栈安全/动态内部rollback。
- nested int数组更新、BigDecimal→Number各 **7/7**，确切561组合CI37994276707成功；子数组协变7/7、reifiable wildcard数组返回6/6、构造primitive转换7/7均有独立完整CI验收。不得重复实现旧shortlist已存在的主体。

## EM18 静态阶段片证据

TestArrayInit.test2 方法byte[]字段写现已正确，原2/2/JADX4/4/Jarde2/2，root294checks；不立重复实现spec。TestArrayInitField literal/ordered两组基线各31命令/8腿，root2093checks。实例b保持构造器语义正确；ordered旧非final trace=0被过于保守的阶段条件拒绝。

[preserve-nonfinal-static-initializer-phase](openspec/changes/preserve-nonfinal-static-initializer-phase/verification-root.md) 生产仅一项flags准入：只有final仍可能常量表达式时拒绝阶段变化。完整表/身份/类型/前向读/未知外部静态字段/效果/异常/预算/原子投影不变。静态6/6、接口4/4、scoped Clippy/fmt/all OpenSpec strict通过；新candidate四腿4/4，root2245checks，ordered四静态字段按原顺序声明、无重复static块，物理clinit及原来源保留。

当前冻结CLI `/private/tmp/jarde-nonfinal-static-cli-v1`，SHA **dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e**，metadata SHA **d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a**；10产品/4test/16canonical pins不改。旧CLI `/private/tmp/jarde-returned-array-cli-v2` SHA71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110仍保留。enum此次cfg(test)修复独立pin，不能重写冻结metadata冒称binary含测试。

准确OrdinaryInit SOURCE补验24命令/76闭合文件：原双JDK2/2、default/all四完整生成源码4/4 raw等于原程序，results/ordinary-static-regression-root-acceptance-v1.json。首轮错要求CLI未序列化的Engine enum_constant_proof键，false失败/raw保留，v3脚本修正并重新执行采集v2；Engine enum断言由本机准确Rust测试证明。

## 工作树、磁盘与执行纪律

只有main/origin/main；14辅助worktree均detached、干净、main祖先、无target、无剩余待合入工作。Codex保护副本保留，不再有分支占用。root专有Git/Cargo/rustfmt/JDK/JADX/CLI串行执行；Luna只读或patch/script，禁止共享运行编译。

本阶段 **20GiB机器余量/1GiB本仓target** 双守卫。实时空间会由其它进程变化，低于余量不启动编译；不降低线强跑。只clean本仓target，不动其他项目，所有原始source/class/raw、canonical和冻结CLI保留。静态片前清658441395字节，本轮准确Rust测试后再清 **286902139字节**，results/root-clean-v2，target不存在。

全部完整生成源码原样重编，空CP/SP、fresh classes -Xverify:all；只允许自己的Runner必要package适配。逐字核原exit/stdout/stderr，不借原class/helper、不删失败成员、不手改生成正文。完整日志不裁剪，历史失败不篡改。

独立债务：拒绝Arithmetic producer的iadd@5来源缺失、平坦Signature generic arity、receiver-tail计费、CF16默认小栈限制。记录并拆分，不混入当前窄片。后续明确队列：实例数组共同前缀、TestArrays2 primitive分支、signed byte/long/ConstantValue。

## 本轮下一片实际状态

TestArrays2.test4已完成fresh25命令/8腿，原2/2、JADX4/4、Jarde2/2全部raw一致；root实际verify-baseline-root-v7接受99闭合文件/完整3methods0fields/原BCI。现有功能覆盖，不新增实现。实例prefix7控制+原3类与2Runner的旧CLI基线已完成；prepare-controls-root-v2.py的新CLI候选回放尚未执行，须显式传新CLI/meta pins，完整类集不得裁剪。

本轮A1验证后只cargo clean本仓target362.4MiB（实际stderr/results/root-clean-v3），target不存在。20GiB守卫保留；实时其它进程可能继续耗盘。

实例10完整控制类/2Runner旧CLI基线已实际80命令/44cases全通过，root独立acceptance-v5。只有handler两ctor双JDKgoto@19来源缺口，精确记录不作全BCI覆盖声明，合同待独立审查。typed value shadow tree被root收敛掉，Luna改为opaque原AST预算化strong compare，产品patch尚未应用。

三项ArrayFill边界已实际baseline-root-v2：39命令/8完整腿，原2/2、fresh JADX4/4、当前Jarde2/2重编/raw成功；root实际独立verifier v2已接受205闭合文件，39命令/78raw streams/12CLI/每类2methods与全部BCI；v1仅准备未执行。long极值与dependent stores已有语义恢复；CONST_INT符号与Long.MAX_VALUE呈现较JADX仍有差距，正在基于JADX真实算法分析歧义与复用接缝。

2026-10-10 02:03 UTC实时可用约14GiB，本仓target不存在，低于20GiB守卫不启动Cargo。继续独立审查/小Java证据与CI，勿清其他项目或修改停止线。

实例patch v2 root完整审查拒绝、未应用：全methods与ctor数量误比、单ctor误拒、无static order准入和整体nonstatic ConstantValue遗漏及API/预算问题；v3私有补丁已准备，尚未应用/编译/验收，root继续对抗审查。原AST RHS比值方向保持，不开放新typed shadow tree。
