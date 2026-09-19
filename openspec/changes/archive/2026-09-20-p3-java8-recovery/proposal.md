## Why

P0/P1、P2（29/29）和分层（7/7）均已归档，P2/分层归档提交为 `7a5f994`。当前进入 P3：`1.1/1.2/1.3/2.1/2.2` 已有交付记录（代码到 `492e31e`，收尾记录 `51a5cac`）；本轮新增基础值流修正 1.3d 后为 **5/12**。已有 Java 方法体、if/loop/switch、lambda 及部分拼接/bridge/accessor 呈现，仍受公开入口和语义边界约束；P4/P5 未实施。 本轮以公开入口和受控行为反例修正规划，见 [复核](verification.md#review-2026-09-19-recovery)；已归档的 P2 不重新实施。

## What Changes

- 先打通普通控制流的单方法 Java 输出、最小命名/source map 和明确 fallback，再增加编译器语法糖；复杂异常处理可在首片降级，最终 Java 8 发布门槛保持不变。
- 增加基于 IR 前置条件的 Java 8 Structured recovery：lambda、method reference、concat、synthetic accessor、inner/capture、bridge、enum、TWR、default/static interface methods、synchronized/finally 和初始化。
- 优先关闭基础表达式的旧值重读与 fallback 丢 effect；随后补齐确定性命名、声明作用域和带物理方法身份的跨方法 source map，再扩展复杂模式。
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

`jarde-java` 已随首个恢复闭环创建，拥有 Region、最小 Java AST/emitter、命名/source map 和模式规则；门面/CLI 已接通 `recover_method`。生产依赖为 java→jvm+reader，图算法沿用 petgraph；1.2 已选择当前最小 emitter，不重复选型或创建新 backend/common 包。当前产物是方法体文本与段表，尚未建立完整 Java 8 源码/重编译覆盖承诺；本轮只修订现有规格、实现顺序和验收条件。