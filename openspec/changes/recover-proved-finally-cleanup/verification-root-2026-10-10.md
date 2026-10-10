# 当前 finally 正文与公共停止验收

本 change 当前 10/11；正文实现不再缺失。当前产品提交 `6ddc77d5cc2a6fcf098aad40998f6d2602731c59` 的冻结 CLI SHA 为 `71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110`。相关产品源和 guard/region 与该不可变提交逐 hash 相同；新增公共测试另属后续检查点，不能借远端产品 CI 声称已验证该新测试。

## 完整类语义

root 以该 CLI 重新生成完整 ImplicitCleanup 类，all/essential 正文相同，双真实 JDK 各重编两份完整 source 和固定 Runner，空 CP/SP，仅新 classes `-Xverify:all` 执行。四腿各四条完成路径与原程序 raw exit/stdout/stderr 一致；cleanup 覆盖返回的 trace 为 `29`。固定历史 JADX source 本次重编运行仍为 `299`，不充当 oracle；不声称重新提取 JADX。范围扩围反例继续拒绝。19 条真实命令、70 个封闭文件、完整 raw 与身份在 `results/current-finally-v1/`，root verifier 实际 167 checks/0 errors。

## 公共停止与回滚审查

Luna v2 patch 接入现有 exact harness；root 强化最后一次压力测试，明确要求 Produced/Partial 且全文与成功正文相同，不能让未命中提交后阶段的测试冒充来源预算验收。zero IR、zero output 和预取消分别核对具体 stop 类型、NotProduced、空 text/map/rules/regions 与无 initializer/init。当前完整 p3_patterns 实测 84 passed/0 failed，强化后的准确单测也实际通过。最终格式化源码再次完整执行 84 项成功，记录 `root-patterns-final-v4`。`root-late-evidence-v2` 是参数入口错误（`--exact` 误送给 Cargo），原失败保留；v3 修正为 libtest 参数后通过。

静态审查：Builder 的 Shape::Finally 子正文 stop、cleanup stop、子树 fallback/未声明值及最终 Try push 失败均走既有 checkpoint 恢复；没有新 AST、pass 或测试钩子。Builder 构建失败后整个结果被丢弃，公共契约保证不发布半个 finally。总 IrItems、BCI 和预取消不能定位内部中途阶段；本测试不宣称动态验证 child 已部分构造后的内部 rollback。深度/其它 finally 家族回归和剩余整类对照仍须按任务验收，2.4/3.1/3.2 未提前勾选。

两项初始聚焦测试 2/2，最终完整测试 84/84；构建峰值 187129829 字节，机器最低余量 25634377728 字节。只允许该较窄 Java target 的 runner 使用 2 GiB 机器余量与 1 GiB target 上限，全 workspace 仍为 20 GiB。汇总与实际源/命令/raw hash 见 `results/local-root-acceptance-v1.json`。

## 深度探针的真实结果：提前回退

root 实际执行 `results/depth-boundary-root-v1/prepare-depth-boundary-root-v1.py`，双 JDK 的 N=2/N=33 原程序完整编译和 `-Xverify:all` 执行均成功；两次调用输出分别是 `run(0)=0 trace=1`、`run(40)=1 trace=2`。冻结当前 CLI 四份完整报告的 `run(I)I` 均为 explanation_only/fallback，execution=complete，包含 finally copy、loop shape 和 uncovered blocks 诊断；不是停止结果，没有命中 recursion_bound。因此该尝试不能完成 2.4 的深度边界任务，原结果/raw/source/class 保留。

N=2 的 loop header BCI0 已未通过证明，finally 异常副本在 BCI26；不能仅凭报告把失败定位到某个内部阶段。它是单保护段的嵌套循环、保存 int 返回值和 finally cleanup 的组合接缝，现有 ImplicitCleanup 分支/throw、固定 Test2 void-loop、Test5 多 protected 段/多 saved return 分别覆盖邻近形状，不能替代此输入。先补浅层完整 source/JADX 对照并沿现有 Region/guard 证明分析，不为未命中的深度限制加生产机制。

浅层 fresh 补验已完成：JADX default/none 双 JDK四完整腿 4/4、Jarde 两份完整 source 原样重编 0/2，实际错误为缺少返回语句；root 独立232checks/0errors。完整输入/raw/闭合 inventory 与架构定位在 `results/shallow-loop-finally-analysis-root.md`。generic finally 证明晚于循环入口，BCI5 时起点已晚于保护段；先记录候选，不把静态改序建议当作已实现或验收，不挤占现有 JADX CF16 剩余任务。

测试检查点后只清本仓 348 文件/178.5 MiB，冻结 CLI 与原始证据保留，真实记录在 `results/root-clean-checkpoint-v1`。

## 真实 Region 深度限制与最终公共测试

早期 loop/branch 探针的回退结果保留。field 读取位于入口的 33 层 branch fixture 最终在双 JDK class 的 BCI450 命中既有 Region 硬限界 `jre_recursion_bound`：CLI exit4、Interrupted、NotProduced、空 text/map/rules/regions、无 init/initializer、Partial。root 独立 verifier v2 实际 204 checks/0 errors，12 条命令及四份原程序完整重编/验证/执行闭合。浅层 N=2 仍回退，不计语法恢复成功；不混淆 container nested_depth 与 Region 的硬限界32，不新增生产 hook。证据在 results/depth-field-branch-root-v1/run-v2 与 root-depth-verification-v2.json。

双版本原 class 字节复制进本 crate 的 cf16-region-depth fixture，源、Runner 与 manifest 同存。永久测试 `cf16_region_depth_stop_never_publishes_class_source` 核对准确停止码/BCI和零发布。v6 首跑在 libtest 默认小栈溢出，未到断言；v7 为此深度 fixture 明确设置8 MiB测试线程栈，格式化后完整 p3_patterns **85 passed/0 failed**，无生产改动。v5 其实仅旧84项通过（新patch未应用），不借其测试名称算深度通过。小栈溢出作为独立限制保留，不声称递归限界防住所有宿主栈大小。v6/v7的原raw和准确源码身份均保留。任务2.4已完成，3.1/3.2待最终整类独立验收及新检查点CI。

## 最终整类对照的 root 独立验收

`results/finally-final-gates-root-v2` 保存26条真实命令、94个闭合文件，双JDK×原源码/固定历史JADX源码/当前Jarde源码共12腿。原源码四份全部完整编译/验证/执行；FinallyNormal 的当前Jarde双份完整源码2/2重编执行、逐原始exit/stdout/stderr匹配原程序，固定JADX双份也一致。FinallyCompletion当前Jarde双份仍完整保留7个物理方法、原样编译缺返回，作为覆盖型负边界；固定JADX虽可编译运行，副作用566/8仍与原56/78不一致。不把10/12编译成功合并为12腿语义通过，不声称fresh JADX提取。

root实际四次`javap -p -s -v`核对全部物理字段/方法name、descriptor、flags与CLI JSON。历史1.1/1.2的3/6 Code计数来自未带-p的javap，遗漏private static mark；物理完整计数应为**4/7 Code**，原始历史证据不篡改。Normal run来源BCI集合与全部实际指令集合完全相同（含保存/重载返回、正常/异常清理副本、重抛）。root独立verifier v3实际 **536checks/0errors**；v2的javap声明识别错误及重现raw保留，v3只将声明识别限定到两空格成员层级，不改raw。Luna prepared v1只作准备，root修复入口/类身份/候选查找与固定oracle后实际验收。

资源/monitor/typed-catch邻近回归继承相同产品提交6dd确切CI38001720578（四job/52steps全通过），包括p3_finally_straight、p3_guard、nested_monitor_regions与p3_java_recovery；没有将其算成新增公共测试的CI。任务3.1完成，当前10/11，3.2等待新测试检查点自己的完整CI。fmt、全OpenSpec strict和diff check已通过，记录root-checkpoint-readonly-v2。完整85测试后再次只清本仓178.5MiB/348文件，见root-clean-checkpoint-v2，冻结CLI与source/class/raw保留。

## CF16 检查点确切 CI 已验收，11/11

acd55c213ac7c670c6e76609871281a341d03ad0 的 CI38007755097 已全成功，四job/52steps，两固定seed各354记录、3355passed/0failed/97ignored；新增两个公共停止测试都真实执行，p3_patterns85/85，nested_monitor4/4、p3_java_recovery54/54。真Temurin25的完整对照、MSRV、Clippy/fuzz/API/tree/OpenSpec与cargo-deny0.20.2 root/fuzz四政策全通过。完整JSON与stable/supply原始gzip保存ci-checkpoint-v1，root实际verifier v3核不可变Git blob与冻结旧CLI、物理fixture和历史local gates后接受，见 results/ci-checkpoint-root-acceptance-v3.json。

API job-log下载失败，改用gh run view --job --log完整输出，capture-method-v1.json记录真实命令；v2在本地历史reader记录的短测试名匹配失败，v3只改为实际完整classfile::tests名字，失败版本/raw保留。当前main另有7196非final静态产品修正，其CI必须另验，不借本检查点。CF16本slice11/11不代表所有finally场景或71单元整单元追平，覆盖型/浅层loop组合与默认小栈限制继续单列。
