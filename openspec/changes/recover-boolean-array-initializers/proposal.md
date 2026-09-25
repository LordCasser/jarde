## Why

已验证的 Java 8 class 可以把原始整数写入新建的 `boolean[]`，JVM 逐元素保存最低位。当前 Jarde 在可证明的初始化器链折叠时把这些整数视作非法 Java 元素，并在整体返回 BCI 引用；JADX 1.5.6 生成 `new boolean[]{0, 1, 2, 3, -1}`，同样无法编译。[独立审计](../../evidence/java-syntax-2026-09-25/boolean-array-initializer/analysis.md)证明这不是普通 `array_write` 修复能覆盖的路径。

## What Changes

- 在现有已证明的数组初始化器计划内保留每个元素与真实 `bastore` BCI 的对应关系，让 `boolean[]` 元素的整数最低位转换归属到正确写入位置。
- 仅在元素类型为 boolean、对应真实 store 为 `bastore`、元素已呈现为 B/C/S/I 时，复用已有最低位表达式；已证明 boolean 元素保持原样，未证明的初始化器仍拒绝或按原语句路径处理。
- 冻结合法 Java 8 补丁 class 的原 JVM、JADX、Jarde 阶段，完成整类 Java 8 重编与运行、逐元素来源和相邻数组初始化器回归。

先决条件是已完成的数组初始化器闭包证明、普通 `[Z` `bastore` 低位写入规则和现有 AST/来源/预算机制。本项不新建通用数组 pass，不折叠未闭合、跨异常处理器或副作用顺序不安全的 store 链，也不处理 boolean 局部/phi 推理。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 增加有证明的新建 `boolean[]` 初始化器对原始整数元素的最低位、逐 store 来源及保守折叠要求。

## Impact

主要涉及 `crates/jarde-java/src/build.rs` 的现有 `ArrayInitializer` 证明记录与元素呈现，增加永久 Java 8 fixture、集成测试和验收证据；不改公开 API、reader/JVM IR、依赖或 CLI 分层。
