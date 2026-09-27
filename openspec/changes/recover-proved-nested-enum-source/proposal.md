## Why

冻结 DT-13 fixture 表明，Jarde 已能正确打印 `enum implements I` 声明头，但会把嵌套枚举拆成独立 `$` 名顶层声明，使原有 `Outer.Inner` / `Outer.Inner.Deep` 源码 API 无法重编。需要在完整证明父子类型关系和 enum 本身来源后，将嵌套 enum 声明放回其词法 owner。

## What Changes

- 增加受证据约束的嵌套 enum 源声明投影，恢复两级嵌套名及父类成员位置。
- 任一父子关系、定义选择、enum 组证书、类成员表或预算状态不完整时，整棵待投影 enum 子树拒绝内联，保留已有物理类报告。
- 将接口声明子形态与嵌套枚举分开跟踪；现有 `enum implements I` 行为保持不变。
- 首切片不覆盖 enum 构造参数与常量匿名类体；分别由 DT-11、DT-12 边界管理。

## Capabilities

### New Capabilities

### Modified Capabilities
- `java8-recovery`: 增加有完整来源关系证明时恢复嵌套 enum 声明及拒绝边界的行为要求。

## Impact

影响 `jarde` 类级源恢复的父子类装配和枚举声明投影；不改变 JVM 读取层的物理类身份，不增加 CLI 专属行为或新依赖。固定证据见 `openspec/evidence/java-syntax-2026-09-27/dt13-enum-shapes/report.md`。该证据确认 `enum implements I` 简单声明形态当前通过，真实差距限于成员 enum 源文件布局和类型名。
