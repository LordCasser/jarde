## 1. Evidence and implementation

- [x] 1.1 固定当前完整类及root同次Walker trace，核canonical/view/loop、protected start/end/join、返回next与visited closure；结合当前try/split/build和JADX遍历/包装明确实现落点，保存实际pins，不把region_at(7)预期当观察，root审读后关闭。
- [x] 1.2 在ordinary单range入口以当前Catches一一绑定的private named row事实贯通loop header/body异常归属，证明并接回prefix+Loop，完整Sequence保留为Try.body；仅明确新增形态借原物理range-end纯bridge证明成为兄弟continuation。实际树级正例核全部protected blocks唯一owned、catch/bridge/join正确；额外/同handler不同range/catch-all/未设置scope/nested Try/Guard及竞争边/效果/跨range/fallback反例、checkpoint/Stop通过后关闭。
- [x] 1.3 永久测试核default/all同正文及真实source-map、新增扫描预算/取消无部分owner；原Switch、fragmented、retry、transparent/finally控制门保持，必要focused实际通过并经root对抗审查后关闭。

## 2. Full class and delivery

- [x] 2.1 与引用生命周期修复的实际最终代码共同收集双JDK、debug/none、default/all未手修完整UpstreamNestedLoops及两个JADX profile；原17oracle值/异常/集合/副作用逐字同原，11方法完整、no-runtime仅按真实编译失败记录，全部真实raw/report/source/来源独立验收后关闭。
- [x] 2.2 独立harness在同一未手修完整类对iterator()/hasNext()/next()/output.add分别注入RuntimeException及Error，原与生成类的handler/escape、消息、调用事件和effects相同；完整类编译运行门实际通过后关闭，不以局部方法替代。
- [x] 2.3 fmt/当前CIClippy/strict、必要focused及fresh双seed workspace、5GiB/target1GiB/1秒守卫/default stack/finally exact clean实际通过；精确产品提交推送/自身CI/真实HEAD cleanmain验收后更新handoff和账本，不自动增加整CF11/71/612计数。

2026-10-13：1.1按root实际source审读及[同次trace](../../evidence/java-syntax-2026-10-13/outer-try-foreach-trace-root-v1/README.md)关闭；7项当前source/trace/review pins见architecture-evidence.json。补充[同次子调用](../../evidence/java-syntax-2026-10-13/outer-try-foreach-child-probe-root-v1/README.md)真实返回header ExceptionEdge(7,0)，未进入body。root核全部4854来源恢复、1/0/0、finally clean/target absent。已据此修订named scope设计；候选尚未实施，剩余5项全部保持未关闭。

2026-10-13：2.1/2.2按[联合完整类实际对照归档](../recover-debug-reference-lifetimes/results/joint-fullclass-candidate-root-v1/README.md)关闭：root 独立接受双JDK/debug/none/default/all的8条17场景Jarde整类运行，以及同一完整类的8条10异常边界场景运行；390归档成员全部复开，164份candidate raw完整。JADX 16条整类编译失败如实保留no-runtime。当前3/6；1.2/1.3仍待新增真实反例的实际永久测试，2.3仍待最终focused/fresh workspace/精确产品自身CI/clean-main。

2026-10-13：1.2/1.3按[最终局部门与对抗审查](../recover-debug-reference-lifetimes/results/final-focused-root-v9/README.md)关闭：v9真实667/0/2、当前CI Clippy/strict/fresh CLI及finally exact clean全部0，4860来源恒同，13项本次具名门逐项接受。真实效果suffix同次候选入口已成立，再证明完整WalkerState回滚及预算不退款；typed alias/full reuse Plan、default/all与物理source-map门保持。当前5/6，最终fresh双seed workspace/产品自身CI/真实HEAD clean-main仍归2.3，不拼接历史失败。


2026-10-13：2.3 按产品自身 CI 与 clean-main 检查点关闭。产品 `597010a8cb9992eead8276b2f2a67186ada393d9` 的 run `38321041720` 已独立接受（4 jobs / 52 steps；双 seed 各 365 targets、96 batches、3547/0/98；4860 source closure）。产品后的 clean-main 检查点 `bcf426e5ef4d2c49c4f1d519c5ef974393b6bf98` 已接受（15 worktrees，14 aux detached，clean/main ancestor/no target，4860 pins）；逐成员证据见[交付归档](../recover-debug-reference-lifetimes/results/own-product-ci-597010-and-clean-checkpoint-root-v1/README.md)。本次文档提交后的最终 HEAD 审计仍待 root 实际运行；不增加 CF11 / CF18 / 71 / 612 完成计数。
