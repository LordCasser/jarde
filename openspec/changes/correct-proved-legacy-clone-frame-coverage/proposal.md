## Why

双 JDK 验证合法的共享 JSR 条件子程序在 Jarde 被拒绝：克隆入口已有正确 Call 入边，但内部普通分支块没有直接 Call/Return 入边。当前规范化后置校验把每块直接调用误作上下文覆盖，阻止后续 Frame/SSA，连带阻止真实复合条件边界验收。

## What Changes

- 在已有 legacy normalization 前置证明成功后，按真实调用路径与内部控制流校验克隆覆盖，允许合法普通分支和异常路径；仍拒绝无证明入口或断开的克隆。
- 保留死调用点的显式克隆与 unreachable 真值、物理来源及合法嵌套返回；新增工作遵守既有预算和取消传播。
- 用已通过双 JDK 验证的完整 class、图级对抗负例、历史子程序和停止测试独立验收。现有诊断失败原样保留。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `jvm-ir`：明确合法 legacy 克隆内部控制流的上下文覆盖与拒绝边界。

## Impact

主要修改 `crates/jarde-jvm/src/canonical.rs` 和必要的真实 Reader 回归。无新增库、公开 IR、pass 或跨 crate 依赖。复合条件变更只消费修复后的前端产物，不承担本债务实现。前置条件为 raw CFG、returnAddress、CallContexts 已成功证明；不扩张不合法返回地址、现代 class 版本的 jsr/ret 或 Java 源码恢复范围，不承诺完整 verifier 或此 class 可恢复 Java。
