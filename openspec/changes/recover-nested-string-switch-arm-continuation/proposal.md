## Why

[CF-14 固定审计与主线复核](../../evidence/java-syntax-2026-09-27/cf14-string-switch/root-acceptance-2026-09-27.md)确认普通 Java 8 String switch 已三方重编运行一致，但外层 `default` 中再嵌一个 String switch 时，Jarde 留下 BCI 123/152/154/156 未获 Region owner，`choose(String)` 只能引用字节码，完整类缺返回而不能编译。内层 hash switch 的已证局部 join 是 BCI 123；现有 `switch_region` 构造一个 case arm 时只保留首次 `region_at` 的 Region，未消费该次返回的 arm continuation。局部 slot 4 跨引用区是控制流断裂后的次生拒绝。

## What Changes

- 对已证属于同一 case arm 的嵌套 switch 续接块继续作有界 Region 走访，使内层 hash dispatch 与最终整数 dispatch 在同一臂中获得完整且唯一的所有权。
- 复用现有相邻二级 switch 的 `project_string_switches` 证书与 `Region::Sequence`，在其条件满足时恢复嵌套 String switch；不按局部变量报错或文本模式直接生成结构。
- 保持普通碰撞/分组 switch 与独立 hash 用途负边界；以 CF-14 三方完整 Java 8 重编、运行及来源验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证嵌套于 switch case 的 String switch 两级分派可在臂内闭合恢复。

## Impact

主要落在 `crates/jarde-java/src/region.rs` 的现有 switch arm 走访、`Region::Sequence` 与 String-switch 投影；仅在必要时对 `build.rs` 的声明/来源作验证，不增加新公共 IR、全局 CFG 改写或局部声明宽松规则。
