## 1. 固定证据与最小证明

- [ ] 1.1 重放 [EM-22 基线](../../evidence/java-syntax-2026-09-27/em22-arithmetic/replay.py)，确认固定 JADX `TestXor` 的布尔常量断言、`TestArithNot` 的 Smali 限制，以及现有 `Operation::Bitwise`/布尔证据/`ExprKind::Not` 路径。
- [ ] 1.2 在既有位运算表达式构造处仅证明 `ixor`、独立 boolean 左值、准确右侧 0/1 literal 和完整 SSA 使用；定向测试覆盖参数、带副作用调用及 int/long、非常量和类型不明负例。

## 2. 源码投影与拒绝

- [ ] 2.1 用现有 `Not` 或原左表达式原子替代已证明 `^ 1`/`^ 0`，维护 XOR BCI 来源；调用一次，eager `|`/`&` 原样。
- [ ] 2.2 验证不完整证明、预算/取消继续按现有路径拒绝或停止，无错误 `!` 或部分表达式。

## 3. 冻结三方验收

- [ ] 3.1 为 [EM-22 replay.py](../../evidence/java-syntax-2026-09-27/em22-arithmetic/replay.py) 增加 fixed 拼写断言；原/JADX/Jarde 完整 Java 8 源码重编、`java -Xverify:all` 八行输出及调用计数一致。
- [ ] 3.2 执行定向算术/布尔恢复测试、格式与适用 workspace check，严格验证本 OpenSpec；不把数值 `~` 或更广的括号美化混入实现。
