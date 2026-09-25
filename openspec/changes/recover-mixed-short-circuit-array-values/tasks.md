## 1. 冻结证据与前置边界

- [x] 1.1 重编冻结 Java 8 源并核对 class SHA、15 个指令 BCI、原/JADX/Jarde 完整类的 24 路径值、数组/下标/RHS 调用及 null/越界异常；Jarde 当前仍 24/24 不等价，见[根代理重放](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-array/region-trace.md)。
- [x] 1.2 在当前源码独立定位 BCI 24 重复 owner 的两个 Region 位置、真实 CFG/SSA 前驱和最小修复边界；[诊断跟踪](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-array/region-trace.md)说明不能放宽 `refuse-overlapping-region-ownership` 的拒绝合同。

## 2. 数组值消费者

- [x] 2.1 在既有私有短路值图中证明唯一 Phi 与 `bastore` 第三操作数绑定；同时证明数组/下标的精确 SSA 输入、求值顺序、类型、一次性与无额外 owner。
- [x] 2.2 只从数组源类型证明 `[Z` 并检查真实 `bastore`，复用 `array_write`/`IndexAssign` 和现有 Boolean 适配；保留所有测试、producer、数组/下标生产者、store 与后缀 BCI 来源。
- [x] 2.3 构造 verifier-valid `[B` 同 opcode、数组类型未知、额外数组/下标 use、第二 Phi use、额外正常/异常边和不可呈现 operand 的拒绝控制；分别检查无部分数组赋值。

## 3. 独立验收

- [x] 3.1 Java 8 完整类重编及 `java -Xverify:all` 24 路径逐行与原 class 对照；复跑既有 Boolean array store、短路字段/返回/调用/局部/网关回归。
- [x] 3.2 运行定向测试、格式、适用 Clippy、`openspec validate recover-mixed-short-circuit-array-values --strict`；记录预算/取消、剩余拒绝边界及私有 Cargo target 清理。
