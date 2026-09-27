# CF-03 else-if 主线独立验收

Root 将拼写实现 `cda72517fba8f2ddcc469d0207c27c16d053fe04` 拣入主线为 `dc35268d` 后重新构建 CLI，并在新空目录重放 [replay.py](replay.py)。隔离 `ChainOnly` 的原 class、固定 JADX 和 Jarde **完整类源码**均 Java 8 重编、验证运行五行逐字一致；Jarde 从零个变为三个 `else if`，与 JADX 数量相同，修后源码与提交的 [`post-else-if-chain/chain/source/jarde`](post-else-if-chain/chain/source/jarde) 逐字节相同。34 项 emitter 测试、格式和 OpenSpec strict 验证通过。

`BranchShapes.nested` 的 BCI 24 共享尾仍被判双所有者，Jarde 完整类编译失败；CF-02 实现合入后 root 已重新重放并确认该差距未消失。它需要独立区域任务，不计作本次 emitter 拼写验收。
