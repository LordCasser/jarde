## Why

[CF-03 固定对照](../../evidence/java-syntax-2026-09-27/cf03-branches/report.md)中，`nested(ZII)Z` 的两条早退分支均可能汇入 BCI 24 的一次 `hits++` 与 `return true`。原 class 和固定 JADX 的完整 Java 8 源码运行一致；Jarde 将 BCI 24 归给两个区域，整方法引用，导致完整类缺少返回而无法重编。CF-02 与 else-if 拼写修复后复测仍存在该差距。

## What Changes

- 在既有 `Region::If` 归属与前向汇合证明内，识别两臂经早退路径汇入同一、单入口范围内的正常前向尾部；尾部由外层结构只拥有并发布一次。
- 保留早退路径、条件求值顺序和尾部副作用；不能证明前驱、边、来源、异常/循环边界或预算时继续原子拒绝。
- 以 CF-03 的完整类三方 Java 8 重编和验证运行验收，并覆盖同形状的重复归属与非共享尾负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证早退分支的单一共享尾恢复为可编译 Java 控制结构。

## Impact

主要影响 `crates/jarde-java/src/region.rs` 的 `If` 区域边界/归属；优先复用 `forward_join_predecessors`、`Frame` 边界和现有区域所有权检查。`build.rs` 与 emit 层仅在现有 `Region::If` 无法表达证明后的结构时作最小调整。JADX `IfRegionMaker` 的出口判定可作参考，但以本项目的 CFG、物理 BCI、完整源码和预算证书为准。本任务不处理 CF-04 条件值、一般不可约 CFG、异常 handler、循环中共享尾或别的语法单元。
