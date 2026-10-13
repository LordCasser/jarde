## 1. Evidence and implementation

- [x] 1.1 固化当前真实 debug/none 双JDK完整类基线、50命令/100raw与11方法身份，独立核原17场景、JADX raw alias编译失败、Jarde debug类型错误及两模式handled拒绝；归档实际pins/raw后关闭，不把重复isolated编译计成独立场景。
- [x] 1.2 扩展现有reference拆分为names-plus-Split，把LVT读取范围一一匹配到已证明身份，保留全部旧所有权/alias/CFG门；永久测试核初始化store在范围外、同名不同类型、无LVT、unused身份真实Store邻接及非邻接/跨身份范围反例、歧义/重叠/缺失/跨边界旧值反例，实际通过后关闭。
- [x] 1.3 永久测试核两个身份的真实类型、各自declaration owner、确定性名称后缀、default/all同正文及物理source-map；新增匹配预算耗尽/取消必须发布原Stopped、无部分计划，root对抗审查和实际测试后关闭。

## 2. Integration and delivery

- [x] 2.1 当前候选CLI重新收集双JDK、debug/none、default/all完整类；debug类型冲突消失、none与其他方法不回退、raw别名和cast保留；handled若仍拒绝则如实保留完整compile fail/no-runtime。完整类不得删方法或修补，root比较真实raw/report/source后关闭。
- [x] 2.2 独立外层try修复后，在实际两项最终代码上重新编译运行未手修完整类，17场景值/异常/集合/副作用逐字同原，来源完整、两compiler模式同语义；完整运行门仍失败时本项保持未完成，不增加整CF11/71/612计数。
- [x] 2.3 fmt/当前CIClippy/strict、必要局部及fresh双seed workspace、5GiB/target1GiB/1秒守卫/default stack/finally exact clean实际通过；提交推送确切产品并接受自身CI，更新账本/handoff并核最终真实HEAD干净main后关闭。

2026-10-13：1.1按[当前完整类基线](results/baseline-product-7e93-root-v1/README.md)关闭。root实际重放独立审计，归档155文件/100raw逐成员复开。实现尚未应用，完整类仍compile fail/no-runtime，剩余5任务保持未关闭。

2026-10-13：2.1/2.2按[联合完整类实际对照归档](results/joint-fullclass-candidate-root-v1/README.md)关闭：root 独立接受双JDK/debug/none/default/all的8条17场景Jarde整类运行，以及同一完整类的8条10异常边界场景运行；390归档成员全部复开，164份candidate raw完整。JADX 16条整类编译失败如实保留no-runtime。当前3/6；1.2/1.3仍待新增真实反例的实际永久测试，2.3仍待最终focused/fresh workspace/精确产品自身CI/clean-main。

2026-10-13：1.2/1.3按[最终局部门与对抗审查](results/final-focused-root-v9/README.md)关闭：v9真实667/0/2、当前CI Clippy/strict/fresh CLI及finally exact clean全部0，4860来源恒同，13项本次具名门逐项接受。真实效果suffix同次候选入口已成立，再证明完整WalkerState回滚及预算不退款；typed alias/full reuse Plan、default/all与物理source-map门保持。当前5/6，最终fresh双seed workspace/产品自身CI/真实HEAD clean-main仍归2.3，不拼接历史失败。


2026-10-13：2.3 按产品自身 CI 与 clean-main 检查点关闭。产品 `597010a8cb9992eead8276b2f2a67186ada393d9` 的 run `38321041720` 已独立接受（4 jobs / 52 steps；双 seed 各 365 targets、96 batches、3547/0/98；4860 source closure）。产品后的 clean-main 检查点 `bcf426e5ef4d2c49c4f1d519c5ef974393b6bf98` 已接受（15 worktrees，14 aux detached，clean/main ancestor/no target，4860 pins）；逐成员证据见[交付归档](../recover-debug-reference-lifetimes/results/own-product-ci-597010-and-clean-checkpoint-root-v1/README.md)。本次文档提交后的最终 HEAD 审计仍待 root 实际运行；不增加 CF11 / CF18 / 71 / 612 完成计数。
