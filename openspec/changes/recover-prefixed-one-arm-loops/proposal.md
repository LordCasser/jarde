## Why

JADX 活动用例 `TestVariablesDefinitions2` 的 null guard 包围迭代循环，Jarde 仍整方法解释且完整源码缺 return。新普通 while 控制基线（31 命令、111 文件）进一步确认：带初始化前缀、循环后直线尾部、反向条件三个方法都拒绝于外层 branch 0；无前缀方法已有结构化正文，缺口不是增强 for 或局部自增拼写。

## What Changes

- 在准确单臂分支边界内续接已证明的直线前缀、普通自然循环，以及可选的独占直线尾部，保持条件极性和尾部位置。
- 复用现有循环证明与 Region sequence，保留 scope、正常边、唯一入口/出口、block ownership、预算和 Stop 门禁；先动态确认当前分段返回位置。
- 原形及四方法完整类按双 JDK/default-all 原样重编运行、逐字比原/JADX/Jarde，独立核正文、物理成员与来源。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证单臂分支中的循环不因同臂直线前缀或尾部而失去完整恢复。

## Impact

主要涉及 `jarde-java/src/region.rs` 单臂 caller 的 continuation 和相邻定向测试；现有 CFG、Frame、loop prover、Region 与 emitter 保持架构。前置为真实 canonical/SSA、已有自然循环证明和同次边界证据，无新库/pass/Frame 字段。

不修改已完成的 `recover-loop-arm-join-continuation` 专用两臂证书；不恢复任意多入口、多出口、异常/finally 或跨 switch 的新形态，不强制 foreach/++ 拼写，不放宽局部类型/SSA。新基线中既有 `noPrefix` 回跳 BCI14 缺来源另行拆分；本片不掩盖它，完整来源最终验收须待独立来源修复。71/612 分母与整单元完成数保持不变。
