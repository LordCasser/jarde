## Why

合法 Java 8 增强 `for` 的合成数组别名占用槽 2，循环后的 `int first` 又复用槽 2；当前 Jarde 将两段当作一个 `int[]` 局部，完整类输出 `first = sum + 1`，`javac --release 8` 报类型冲突。原 class 与 JADX 有无调试信息的输出均可编译执行，说明这是 Jarde 的局部身份和类型闭环缺口，不能靠清理增强 `for` 的缓存声明解决。

## What Changes

- 在既有 SSA、帧类型、LVT 与 Region 事实内，识别同一 JVM slot 中定义/用途不相交且类型不兼容的先后局部生命周期；一份 LVT 仅命名后半段或完全无调试表时仍可为可证明的段分别命名、定型和声明。
- 非平凡跨段 phi、能从后段返回前段的回边、异常边或用途重叠使分段不确定时，不得把不兼容写入强塞进一份 Java 声明；保留可查拒绝和已有未恢复片段。平凡自 phi 不阻止已证明的循环前段。成功分段保持原执行、来源、预算及取消契约。
- 使用已冻结的同槽 Java 8 三方夹具，另加相同类型赋值、跨分支/处理器和 category-2 槽的拒绝/非回归对照。与数组增强 `for` 缓存声明清理分别实施验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：一个物理局部槽在不相交的词法生命周期内承载不兼容类型时，已证明的两个源局部可分别呈现；不能证明时不发布类型矛盾的完整源码。

## Impact

主要涉及 `crates/jarde-java/src/reuse.rs` 的局部身份决策、`names.rs` 的分段命名和 `build.rs` 的声明/写入类型核对；沿用现有 SSA/帧/Region，不新增 crate、全局优化 pass、外部依赖或公开 API。冻结证据在 [foreach 同槽三方对照](../../evidence/java-syntax-2026-09-24/foreach-cache-declarations/analysis.md)。当前前置条件是已存在的数组增强 `for` 投影及 P3 局部声明规划，邻接的 [缓存声明清理](../prune-proved-foreach-cache-declarations/tasks.md)不负责这个类型错误。
