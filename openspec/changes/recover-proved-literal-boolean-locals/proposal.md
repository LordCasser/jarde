## Why

真实 JADX `TestBreakWithLabel` 的 debug/no-debug 原件均把局部布尔标志编译成两次 0/1 写入。Jarde 已恢复带标签循环跳出，却将该变量保留为 int，拒绝 `append(Z)` 并丢失打印语句；完整重编后的 192 次用例观测中有 128 次 stdout 不符，JADX 可编译结果与原件一致。

## What Changes

- 对所有生产与消费均有闭合证明、且至少一个独立布尔描述符消费的非参数局部 0/1 标志，恢复布尔声明、赋值及现有拼接语句。
- 复用现有局部类型决策与停止事务，保持 int 消费、未知写入、复用歧义和未闭合合并的拒绝。
- 冻结完整原始基线，补真实 IR、公共 class/JAR 与完整源码编译运行对照；分开记录剩余来源映射债务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 受证 literal-only 局部布尔标志恢复与其消费语句保留。

## Impact

预计只涉及 jarde-java 的 Declarations 类型证明、相关测试及原始 fixture/Reader 库存。实现前必须验证真实 SSA 的全部写入、读取、local Phi 输入和精确复用归属；上一片循环入口来源产品 2684b4ee7 的自身 CI/源码采集关闭前只允许私有实现。无需新 crate、公共 API、IR/CFG/Region 节点、全局 solver 或依赖。非目标：任意整数布尔化、任意方法调用的参数推断、放宽拼接准入、补齐所有来源映射、DEX 或 CF09/71/612 整体完成。完整基线及 root 逐字复开入口为 `openspec/evidence/java-syntax-2026-10-19/cf09-testbreakwithlabel-original-root-v1/README.md`。
