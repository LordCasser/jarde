## 1. 出口证书与区域归属

- [ ] 1.1 在现有循环恢复中有界证明头部失败网关与一个体内 `break` 网关：两者均为无效果直接转移、具有精确唯一正常入边并指向同一真实后继，异常/额外入口拒绝；以正例及至少额外入口、不同目标、带效果负例的定向测试验证。
- [ ] 1.2 仅对已证网关区分循环的物理正常出口和 Java `break` 最终目标，令体内网关被 Region 实际认领一次、其条件边输出正确退出；以 BCI 4/7/12/15/18/21/24/25 来源、无未覆盖块、相邻单出口循环测试验证。

## 2. 行为与拒绝验收

- [ ] 2.1 用 `cf08-endless-loops/replay-simple.py` 对原 class、固定 JADX 和 Jarde 三份完整源码做 Java 8 重编及 `java -Xverify:all`，六行输出均为 `0,1,2,3,3,3`，Jarde 无 `@bytecode`，且 `limit=4` 不超时；保存修后证据与 CLI SHA。
- [ ] 2.2 验证额外入口、异常边、带效果/不同后继网关，以及预算与取消均不能发布遗漏出口的完整源码；运行 CF-07 返回叶、CF-09 嵌套标签和循环基础定向回归，证明未改变已有出口目标与更新次数。
- [ ] 2.3 运行 `cargo fmt --all -- --check`、相关包检查、`git diff --check` 和 `openspec validate recover-proved-loop-exit-gateways --strict`，记录结果与独立 Cargo target 清理量，供 root 重新构建和三方验收。
