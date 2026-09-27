## 1. 固定物理事实与同轮值流

- [x] 1.1 复核固定 `HandlerLoopProbe` 与完整 `ExceptionRegionsAudit` 的原/JADX/Jarde 三方 Java 8 基线、异常表、首拒绝及 JADX 完整类的 catch 错放；证据见 [架构复核](../../evidence/java-syntax-2026-09-27/cf18-handler-region-triage/architecture.md) 和 [CF-18 完整报告](../../evidence/java-syntax-2026-09-27/cf18-exception-regions/report.md)。
- [ ] 1.2 保存两类 `run()` 的同轮 Canonical CFG、handler row、SSA 定义/使用和来源清单，逐 BCI 核内外 handler 到循环更新点、累计值/索引及 catch 参数；以可重放 dump/测试证明设计的唯一归属前提，发现不符先修订本 change 而非放宽门槛。

## 2. 一个受限分段 catch 证书

- [ ] 2.1 在现有事实中按 ordinal 证明同 handler、同类型的分段行与内层 handler 对每个可抛 BCI 的保护、Canonical 异常边及 JVM 实际优先级，得到唯一外层词法 owner；用正例和经 `java -Xverify:all` 确认的错类型、错表序、遗漏保护及额外 throw-site 负例核派发与拒绝。
- [ ] 2.2 仅对获完整证书、handler 无普通前驱且每条有限正常路径汇入同一循环更新点的异常根建立不可约例外；分别核缩小类的直达 51 和完整类 BCI 48/52、55..69→85 的分支/continue/fallthrough，另用相同普通环但 handler 落入其它环内点或额外普通入口的 verifier 有效负例保持整方法拒绝。
- [ ] 2.3 让既有 Guard/Region 复用该证书，将内层具名 catch、分段外层 catch、两个 handler 与循环更新/continue 各认领一次，保持原异常表顺序；以固定两类的 Region/source-map 测试及交叉/共享 handler 负例核物理 BCI 和异常行无漏领/重领。

## 3. 完整类源码与回归

- [ ] 3.1 按同轮 SSA 证明并呈现 handler 参数、累计值及循环索引的完整局部作用域；仅修正本候选阻碍，核两固定完整类均无 `@bytecode` 且 Java 8 重编成功，额外值来源不能闭合时保持原子拒绝。
- [ ] 3.2 分别重编原/JADX/Jarde 的两份完整 Java 8 类并运行 `java -Xverify:all`：缩小类三方 `4:110`，完整类 Jarde 与原类 `124:115`；固定 JADX 完整类的错误外抛/非零退出作为负对照保存。
- [ ] 3.3 复跑既有 catch、循环、finally/TWR、预算/取消及 verifier 有效负例；`cargo test -p jarde-java --tests --locked`、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`openspec validate recover-fragmented-loop-catches --strict`、`git diff --check` 通过，清理专用 Cargo target。
- [ ] 3.4 root 独立审阅证书、异常派发、SSA、来源和完整类三方行为并记录验收；通过后更新 CF-18 清单，仅把已验证的固定形态标为恢复。
