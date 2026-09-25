## 1. 固定正反边界

- [x] 1.1 将冻结的 `InstanceOfMerge` 类和 Runner 接入定向测试，先钉修前八行原/JADX/Jarde 一致、调用次数均为 1，以及旧 Jarde `?: 0 : 1` 加 `% 2 != 0` 的源码形状；检查 class SHA 与 BCI 4/7/10/11/14/15。修前证据与修后独立验收分开保留。
- [ ] 1.2 将已冻结的 `BooleanMergeControls` 正极性 1/0 与反极性 0/1 纳入测试，再补 verifier 有效的 2/3、额外消费者、不闭合入口负例；用原 class 的 `-Xverify:all` 轨迹和现有 `recover-integer-boolean-returns` 基线固定每条路径行为。

## 2. 精确布尔缩写

- [x] 2.1 仅在既有条件值证明和唯一 `ireturn Z` 消费已成立时，验证 Boolean 测试、两臂精确 0/1 SSA 常量和真实极性；让正反两种映射分别输出测试或其 `Not`，以定向源码断言和 2/3 负例证明不放宽其它值图。额外 consumer/入口的专用负例仍归 1.2。
- [x] 2.2 保留 test、两臂 producer、transfer、Phi consumer 与 return 的来源；让已证明 Boolean 返回跳过通用低位适配，其余整数仍按最低位输出。以来源 BCI、预算/取消与 2/3 回归验证原子提交。

## 3. 独立整类验收

- [x] 3.1 root 用重建 CLI 的完整类文本原样 `javac --release 8`，再以 `java -Xverify:all` 与原 class/JADX 比较八行、正极性样本和低位负例；核对一次求值及无 `% 2` 的目标返回文本。
- [x] 3.2 root 复跑条件值、`instanceof`、整数布尔返回及来源/预算定向测试，执行 fmt、适用 Clippy、`openspec validate simplify-proved-boolean-conditional-returns --strict` 和磁盘清理，并记录未覆盖的其它消费者形状。
