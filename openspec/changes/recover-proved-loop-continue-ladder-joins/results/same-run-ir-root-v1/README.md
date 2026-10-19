# 同轮 IR 与首次内部重复所有权

原始debug/no-debug两个输入在同一精确unit中实际1/0通过，分析/Region全来自同轮Reader。实际ForHeader成功，init1/update50/slot3，header Phi V6=[V22,V34]；自然循环header2、唯一latch50，非不可约。首个内部gap是Loop at42（已被另一arm占有），继而发布ArmsDoNotMeet at8和LoopShape at2。不是for更新识别缺失；全局ipdom56不能替代本循环局部完成证明。

完整原始[JADX/Jarde基线](../../../../evidence/java-syntax-2026-10-19/cf09-testcontinueinloop-original-root-v1/README.md)和所有失败单独归档。此处包含原始现有JRE_JOIN_PROBE（stdout除usage外全JSON与基线一致）、临时IR源码、root两个test-only观测打印、所有raw/receipt、v1/v2架构备忘的更正；v1误称receipt无确切branch BCI，v2据公开regions纠正为已发布8，内部first gap由本次实际探针另行确证。不能把旧备忘当当前结论。

临时region.rs已恢复SHA209de1ed354203b9f2289f19d74d7a5a453afd5a785a8abe12852a6e8983a242，额外test文件已删除、target absent；峰值170241062B，默认栈、5GiB/target1GiB/每秒guard/finally exact clean。独立直接for_header探针使用空terminal-return/visited，但实际header_tested_loop另行观测同样证实update50与空terminal returns，不将孤立调用冒称实际路径。

capture.tar.gz SHA cd7f04d0af145c270f85b9a3fa3fd3c2262fb43aa5f42f7493aa7397cb1f1d26，41payload条目加inventory，root逐字/mode复开。只是原始事实和架构边界，不是实现或候选验收。
