## Why

JADX 的 `TestEnumsWithTernary` 展示 enum 常量参数中的条件表达式。冻结 int fixture 将参数类型固定为已支持的 int literal 后仍在 Jarde 中因条件分支字节码无法进入 enum 前缀证书而编译失败，说明剩余独立差距是受控的 ternary 构造实参恢复；测试原始 String 形态另外与 DT-11 的普通 String 参数交叉。

## What Changes

- 支持完整 enum `<clinit>` 中控制流可证明的 int 条件表达式实参，输出 Java `condition ? left : right`。
- 只有分支条件恰求值一次、左右 arm 唯一汇合到对应 enum constructor argument，且无额外效果时才投影完整常量组。
- 任一 branch/consumer/constructor 对应歧义、未解释操作或停止都原子拒绝 enum 组。
- 不纳入 `String` 普通 constructor argument 或 `String...` 数组实参；它们归 DT-11，JADX 的组合 String 示例只留为相邻证据。

## Capabilities

### New Capabilities

### Modified Capabilities
- `java8-recovery`: 增加受证的 int ternary enum constructor argument 及 branch 拒绝边界。

## Impact

影响 `jarde` enum `<clinit>` 构造实参证明与 enum 常量组 projection。不得把任意 `<clinit>` 分支作为 ternary，也不改变方法内一般 CF-04 恢复。冻结 int 与 String 对照见 `openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/report.md`。
