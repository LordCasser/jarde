## Why

JADX 的 `TestEnumsWithCustomInit` 声明在标准 enum 常量初始化之后遍历 `values()` 并填充 Map；冻结对照显示 Jarde 没有投影这种用户 `<clinit>` 后缀，完整源码连 Java 8 编译都无法通过。现有 `Measure` 静态赋值窄证书不覆盖循环和多步副作用，因此需要单独建立受证的 enum suffix 投影边界。

## What Changes

- 在标准 enum 常量前缀完整获证后，支持一个有界的用户 static field 初始化及 `values()` 增强 for-each Map 写入后缀。
- 按 class initializer 原次序输出 enum 常量、静态字段初始化和静态块；只有后缀每条操作都能安全拼写时才提交完整投影。
- 缺失/额外指令、未知调用、循环控制关系或预算不完整时整组拒绝投影并保留物理来源。
- 不扩展到任意用户 `<clinit>` 反编译器、不引入对非 enum initializer 的更改。

## Capabilities

### New Capabilities

### Modified Capabilities
- `java8-recovery`: 增加完整 enum prefix 后已证明用户静态初始化后缀的恢复行为。

## Impact

影响 `jarde` 类源装配中的 enum `<clinit>` 后缀证明和 enum 声明投影。使用既有 class/member Code 与同次 initializer 结构事实；不增加依赖、不改变 CLI 契约。冻结对照见 `openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/report.md` 的 CustomInit fixture。
