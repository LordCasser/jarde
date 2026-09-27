## 1. 复核并证明局部汇合

- [x] 1.1 在固定 `LoopIfJoin` 上验证 BCI 26 为内层 join、34 为父 arm boundary，确认内层循环 Region 的唯一出口与来源；若循环本身没有完整 Region，先回报证据，不放宽声明规划。
- [x] 1.2 扩展现有私有 `continue_inner_join_arm` 证明：恰有一个无额外转移的 Loop 臂与一个 Straight 臂到相同 join，准确核对 canonical 入边、父 boundary、scope 和 owner，再消费 BCI 26 的直线尾部。
- [x] 1.3 多入口、额外 break/return、异常边、不同 join、已访问尾部和预算/取消都保持保守拒绝或中止；原简单直线臂路径不退化。

## 2. 三方验收

- [x] 2.1 原/JADX/Jarde 完整 Java 8 类重编、`java -Xverify:all` 三行同为 `4 / 13 / 11`，`run` 无 `@bytecode`，局部 BCI、循环更新与外层 `+10` 各有唯一来源。
- [x] 2.2 重放已验收 CF-08 纯双网关、CF-07/09 和相关 if/loop 测试；原 `NotIndexedLoop` 带效果双出口仍拒绝，不把它混入本修复。
- [x] 2.3 运行 fmt、crate check、`git diff --check`、`openspec validate recover-loop-arm-join-continuation --strict`，记录合并态结果并清理 Cargo 构建残留。
