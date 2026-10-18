# CF08 受证双入口 int 前缀循环

五个完整 class 逐字复制自 root 已验收的原始 gate，未重编、删成员或修目标源码。四个正向方法分别覆盖+3、-5、计数槽位/额外未使用参数、反向入口与显式前缀goto。十个控制方法覆盖非int源码类型、别名、调用、额外入口/出口/live块、栈逃逸、真实异常行和legacy jsr/ret。

[manifest.json](manifest.json) 固定全部源码/class SHA、完整Code/BCI/handler和命令来源。原始gate共17条实际命令：writer/driver编译与生成、五份完整javap、原始双VM144条结果、两错根拒绝、旧CLI五完整class-source。负方法从不调用，仅反射库存/构造；legacy class major49，其余52。手写帧由两JVM实际验证，原始运行以parentless加载器、精确CodeSource和空cwd隔离。

使用JDK8可重建：先把writer编译到独立工具目录，再运行 `EntrySplitFixtureWriter <empty-class-root>`。writer编译参数是 `-source 8 -target 8 -g:none -encoding UTF-8 -proc:none`；输入target由DataOutputStream完整写出，writer自身debug配置不影响target。默认driver接受actual-root/expected-root两参数，检查全部五类，仅执行正类四方法；负类只构造，禁止调用alias等可能不终止的控制。

来源证据见 [原始gate](../../../openspec/changes/recover-proved-entry-split-int-loops/results/fixture-original-gate-root-v1/README.md)。旧CLI对四正例与九不可约控制均整方法quote；legacy保持既有两SubroutineEntry和一UncoveredBlocks，quality=fallback/representation=mixed，不冒称整方法quote。该fixture不是原Smali/DEX等价，也不关闭整个CF08。reader增量为5class/19Code/1handler/51target/2jsr。生产实现及完整生成源码对照仍待验收。

## 非零prefix槽与递增计数的补充

独立第六类 `EntrySplitSlotCounterCases` 见 [generalized-manifest.json](generalized-manifest.json)。`permuted(ZII)I` 的skip/counter/value位于0/1/2，前缀+7；`countUp(IZI)I` 的计数器+1，输入仅0/-1/-2使循环有限。两者保持ifeq零比较，完整类独立11命令gate、双VM72观察和两错根已root接受，不改/不重跑原五class。旧CLI两拒绝，新候选default/all两structured且两个前缀span/物理BCI闭合；完整生成源码重编执行仍待最终对照。writer/driver逐字复制自该gate。增量1class/3Code/6target；合计六class/22Code/1handler/57target/2jsr。
