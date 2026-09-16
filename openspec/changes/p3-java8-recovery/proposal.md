## Why

以 P2 完成为进入条件。本 change 规划在 Java 8 RuntimeProfile 和历史 45–52 输入上收敛高频恢复模式、变量命名和 source map，并把无法证明等价的结果明确降级；当前尚未实现。

## What Changes

- 增加基于 IR 前置条件的 Java 8 Structured recovery：lambda、method reference、concat、synthetic accessor、inner/capture、bridge、enum、TWR、default/static interface methods、synchronized/finally 和初始化。
- 增加确定性变量命名、作用域、重命名冲突处理和 BCI 到 Java source map。
- 增加 recovery profile、语法状态、重编译/行为验证边界和 Java 8 生产语料验收。
- 保留 X1 原始引用边；恢复只生成派生可读层，不把可读性当作证据替换。

## Capabilities

### New Capabilities

- `java8-recovery`: 在已满足 IR 前置条件时恢复 Java 8 高频语义结构。
- `source-maps`: 输出稳定命名、作用域与 Java 节点到原始 BCI/evidence 的映射。
- `recovery-validation`: 区分 representation（Java/Bytecode/Mixed）、quality（Structured/Conservative/Fallback）和 syntax/verification 状态，并以语料决定覆盖矩阵。

### Modified Capabilities

无。P3 消费 P2 的 IR/Resolver 和 P1 的 XRef evidence，不回写或删除原始结构事实。

## Impact

影响 `jarde` 的 recovery、Java AST/formatter、source map、diagnostics 和 CLI 输出层。优先评估可用、活跃且质量符合要求的 Rust 语法树、格式化和文档组合库，记录选型依据；JVM 语义恢复由项目负责，不引入 JVM 运行时依赖。阶段门槛包括 A09、A10、A12、A13、A16，以及架构第 20 节的恢复语料矩阵；本 change 当前尚未实现。
