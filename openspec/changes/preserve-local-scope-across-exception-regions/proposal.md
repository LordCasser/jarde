## Why

恢复器目前把跨 try/catch 的局部变量按方法级“已声明”状态处理，可能在 catch 块内声明变量，却在 catch 之后输出读取它的语句；生成文本因此连 Java 词法作用域都不成立。该缺口已由真实 Java 8 class、原/JADX 行为及 javac 错误确认，需要先让局部声明可见性与 region 结构一致，再谈更广的异常路径恢复。

## What Changes

- 恢复输出中的局部声明与读取遵守 Java 词法作用域；嵌套 region 内声明的局部离开作用域后不得被继续引用。
- 无 LVT 时以 catch clause 与实际定义—使用证据区分同一 JVM slot 上的独立 handler 参数，避免把兄弟或嵌套 catch 误判为一个逃逸变量。
- 当异常边或 quoted/fallback 区域使跨 try 的定义/使用关系无法完整恢复时，拒绝会产生越界读取或丢失原始效果的相关结构，并按既有 refusal 契约保留相关 bytecode、origin 和停止/拒绝信息。
- 为 try/catch 局部作用域补充正向、拒绝及边界验收，涵盖无调试表 class、嵌套作用域与区域外读取。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：补充跨异常 region 的局部声明可见性与拒绝边界要求，不把现有局部声明恢复另立能力。

## Impact

- 影响 `crates/jarde-java` 的 region、局部声明规划、AST 构建与恢复测试；不改变 `src/class_source.rs` 的类级成员拼装职责。
- 前置是现有 canonical CFG、exception table、Frame/SSA、Region 和局部身份事实；未知的 reaching definition 或越界 scope 关系必须拒绝，不能从名称或 BCI 邻近关系推断。
- 不属于 `recover-conditional-values` 的条件值准入，也不处理 bridge/class-source 投影；这些仍由各自变更负责。
