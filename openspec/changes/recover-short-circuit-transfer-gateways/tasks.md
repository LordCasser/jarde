## 1. 冻结证据与架构边界

- [x] 1.1 核对 `ChainExtraBoundary.class` SHA、16 个已解码 BCI、原/JADX/Jarde 三份完整类的 Java 8 重编与 32 路径 JVM 结果；记录 JADX 错误的具体输入及 Jarde 当前整体引用基线。
- [x] 1.2 审计 `ShortCircuitValue` 的 region 图收集、物理前驱、SSA/Phi proof、递归值树和所有消费者调用点；冻结无效果 gateway 与多入口/效果/异常边负例，不扩公开机制。

## 2. 纯转接的闭合证明

- [x] 2.1 在原有有界无环图中认领单入口/单出口、已解码、纯前向 transfer-only gateway；区分逻辑后继与物理节点，精确核对每条正常/异常边，证明全部节点后一次提交 owner。
- [x] 2.2 使既有测试边、producer、唯一 Phi 和静态 `Z` 字段消费者证明接受上述逻辑折叠；保持真实 1/0 极性、惰性顺序、预算/取消与后缀，拒绝外部入口或第二消费者。
- [x] 2.3 发射现有 `Conditional` 和一个字段写入；测试、转接 BCI 8、两个 producer、字段消费者和后缀的 source map 完整。完整类 32 路径输出与原 class 逐字相同，不能以 JADX 或可编译性替代。

## 3. 独立验收

- [x] 3.1 回归已验收的混合字段 16 路径、直接返回 8 路径、调用实参 8 路径，以及两/三测试、异常边、第二消费与所有权拒绝控制；新增多入口/有副作用/异常转接拒绝。
- [x] 3.2 运行定向测试、格式、适用 Clippy、`openspec validate recover-short-circuit-transfer-gateways --strict`；记录剩余边界与 Cargo target 清理，只勾证据支持项。
