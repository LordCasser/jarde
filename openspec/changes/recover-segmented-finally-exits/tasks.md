## 1. 固定输入与拒绝边界

- [x] 1.1 冻结 pinned Test13 的五行异常表、BCI/opcode、七条原/JADX/Jarde 路径及当前 `jre_guard_finally_copy`；用 `cf16-test13-multisegment/replay.sh` 和 `README.md` 的 shape/运行断言核验。
- [x] 1.2 冻结清理目标、分支绕过、范围扩围和重抛值四个 verifier 有效负例；用 `negatives/replay-neighbors.sh`、`neighbor-root-acceptance.md` 和各原 class 的 `-Xverify:all` 轨迹核验。

## 2. 五行证书

- [x] 2.1 在现有 Guard finally 分派中建立私有五行候选，逐行证明两个同目标具名范围、两个同目标 catch-all 范围与 catch-body 行的精确顺序/边界；用固定 Test13 和行缺失、换序、扩围负例的定向测试核验。
- [x] 2.2 证明四份清理的成员/接收者/参数及 SSA 唯一消费等价、早退与两个汇合的所有入口出口、异常保存到原值重抛；用固定正例和四个 verifier 有效近邻的定向 Guard 测试核验，拒绝任何无法证明的额外边。

## 3. 受限区域与源码

- [x] 3.1 在 Region 中只按受证两个 segment 和拥有节点构造一个 try、一个具名 catch 与一个 finally，保留早退和后续汇合，避免重复认领共用 handler；用固定 canonical 图的逐块 owner、正常/异常边和五行 ordinal 测试核验。
- [x] 3.2 在既有 Builder checkpoint 内输出唯一 `finally` 和四种完成路径，折叠其余副本为来源事实；用固定方法源码、每个物理 BCI/异常行来源、预算/取消与构建失败原子性测试核验。

## 4. 三方验收与回归

- [x] 4.1 用 fresh CLI 从固定完整类恢复源码，将原 class、pinned JADX 和 Jarde 的完整 Java 8 源码重编、`java -Xverify:all` 跑七条路径，逐字比较效果顺序、次数、catch 类型与结果；对扩围负例必须保留原 class 的重复清理行为或安全拒绝，不采用 JADX 的错误一次清理。
- [x] 4.2 跑 Test12、共享 catch-all、普通 finally、TWR/monitor 定向回归及 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-segmented-finally-exits --strict` 和 `git diff --check`；清理专用 Cargo target。
- [ ] 4.3 root 独立审阅五行证书、Region/Builder ownership、负例和三方运行，在固定输入通过后写验收记录并更新 CF-16 清单；尚未覆盖的 finally 形态继续单列，不把整个单元标为追平。
