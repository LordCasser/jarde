## 1. 证明共享前向尾部及唯一归属

- [ ] 1.1 在现有区域 walker 内证明两后继到唯一共同前向入口的正常路径、支配/前驱、无回边/异常边与预算界限；以 `BranchShapes.nested` BCI 0→4/14→24 的正例和额外入口、不可比候选负例验证。
- [ ] 1.2 让嵌套 arm 保留外层 BCI 24 停止界并由外层续接只领取该尾一次；以区域所有权检查、源映射和 `hits++` 恰一次的断言验证，拒绝证据不足的形状。

## 2. 完整源码与非回归验收

- [ ] 2.1 更新 [CF-03 固定脚本](../../evidence/java-syntax-2026-09-27/cf03-branches/replay.py) 的修后门槛，验证原 class、固定 JADX、Jarde 的完整 `BranchShapes` Java 8 源码均重编并以 `java -Xverify:all` 运行 15 行一致；`ChainOnly` 五行及三个 `else if` 保持通过。
- [ ] 2.2 运行相关 P3 区域、条件/早退、来源和预算/取消测试，`cargo fmt --check`、`cargo check --workspace`、`openspec validate own-proved-shared-early-return-tail --strict`；记录仍拒绝的异常/循环/多入口边界并清理本任务 Cargo 产物。
