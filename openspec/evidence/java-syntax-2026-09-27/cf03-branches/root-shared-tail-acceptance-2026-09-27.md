# CF-03 共享早退尾主线独立验收

在主线 `81817884`，root 审查了候选的双臂前向可达、唯一最早共同后继、支配/准确前驱、纯测试、单个 false-return 叶和外层尾所有权。新局部区域只在顶层布尔分支启用，循环、异常、switch 和其它共享出口仍拒绝；`Frame.shared_tail` 不更改循环 scope。额外入口与不可比较共同候选负例均保留引用，预算/取消不发布半份文本和映射。

重新构建主线 CLI 后，独立运行 [固定脚本](replay.py)至 `/tmp/jarde-cf03-root-posttail-replay-20260927`。原 class、固定 JADX、Jarde 的完整 `BranchShapes` Java 8 类源码均重编、`-Xverify:all` 运行 15 行逐字一致；`ChainOnly` 三方五行一致且 Jarde/JADX 各有三个 `else if`。Jarde 两类源码 SHA-256 分别为 `6009338fb92cf9a83fb43e9029eb56b7fd675cba7dd1113134d553861dc24b9b` 和 `07d19e0854e8842832d9e8c078d1988751744d6c589448eba1cc0900703e4b8a`，与实施者修后证据一致。`p3_shared_tail` 5/5、OpenSpec strict、格式及 diff 检查通过。

验收限于这个共享早退尾形状，不代表 CF-03 所有 if/else 图已追平；CF-07 的循环内提前返回叶仍独立处理。
