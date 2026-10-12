## 1. Baseline and proof integration

- [x] 1.1 固定产品597010的双JDK/debug-none完整17方法与44运行行基线、每份27命令/54raw及JADX整类失败；root实际独立审计、逐成员复开归档并记录纯goto映射缺口后关闭，不把静态分析、工具准备或no-runtime当通过。
- [ ] 1.2 在同次候选内复用普通named row、完整prefix+Loop及owner事实，绑定单条typed row的ordinal/type/handler/range与实际Try/canonical/code；永久测试真实进入新路径后核正确body/catch/continuation及缺失/重复owner、同handler不同range、catch-all、多range、同handler/range的multi-catch、nested Try/Guard反例，root审读且实际通过后关闭。
- [ ] 1.3 把该私有准入接入既有完整void用途扫描，整个Code/operands/SSA/effects/Operations一一配对、逐effect handler归属核验，包含catch读写；保留no-handler路径及参数writes/alias/unique receiver门。永久测试库存差异、异常row差异、catch重赋值/未知消费、实际新扫描late-budget/取消且无部分candidate，root审读和实际测试后关闭。
- [ ] 1.4 沿既有List源级证明/签名擦除/overlay完成统一精确E投影，至少String与Integer的分支/null、remove(Object)前缀和已恢复nestedforeach；default/all相同，add合成包装可删但物理next-checkcast/removeObject重载及来源保持。永久公开测试正例、一个合格try方法改成员名及formal/local名字仍准入、unknowncall/rawalias/wildcard/indexOverload/重赋值旧控制和overlay晚Stop通过后关闭，不硬编码fixture或新增公共carrier/solver/pass。

## 2. Whole class and delivery

- [ ] 2.1 在实际最终候选重新跑双JDK/debug-none/default-all未手修完整17方法类和至少45场景Runner（原44行完整保留，加可变output的handled正常完成行）；五个目标方法的泛型投影成功，unknowncall及旧反例继续拒绝，原17行oracle不变，String/Integer的iterator/hasNext/next/output.add RuntimeException/Error消息/escape/events/effects逐字同原；JADX每JDK使用自身input，真实失败保留no-runtime。root独立核raw/source/report/class身份/来源并复开归档后关闭。
- [ ] 2.2 fmt/当前CI Clippy/global strict、必要focused及fresh双seed workspace、5GiB/target1GiB/1秒进程组守卫/default stack/finally exact cargo clean实际通过；root提交推送确切产品并接受自身CI，更新handoff/完成审查与账本，核最终真实HEAD clean-main后关闭。本项通过不自动增加整CF11/71单元/612测试计数。

前置：两前置change的产品自身CI与clean-main交付完成后应用代码。目前仅规划与基线观察，全部实现任务保持未关闭。持续目标继续，Luna在私有目录准备候选，root负责应用、工具链和验收。

2026-10-13：1.1按[实际基线归档](results/baseline-product-597010-root-v1/README.md)关闭；root重放auditor-v2及独立复开276成员/108基线raw/134总raw绑定。第一次复开因排除全部jar的root断言误拦受控输入；第二次逐jar核仅含一个完整原class、与同模式同JDK输入逐字一致，安装二进制仍全部排除，原失败保留。44行整类运行同原，五个目标仍raw签名，unknowncall保留拒绝。当前1/6；对抗审查补handled正常完成行（候选至少45行）、multi-catch与合格try改名门，其余任务未实施，计数不变。
