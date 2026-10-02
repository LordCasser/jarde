## Why

[varargs 构造实参巡查](../../evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/README.md)确认：`new ArrayList<>(Arrays.asList(1,2,3))`（集合拷贝构造接 varargs 工厂——`new ArrayList/Set/HashMap<>(Arrays.asList/…)` 高频形态）整方法拒绝——varargs 降低的内联匿名数组存储链（`anewarray; [dup; index; box; aastore]×n`）位于外层构造 new-dup 与 ctor 之间，被 `init.rs` 走查当交错效果（`jre_new_interleaved_effect`）；级联断裂下游（W1.use 的 sumExt 链）。判别钉死：裸 varargs 调用位（return/赋值/拼接）全部健康且内联数组已按 `new T[]{…}` 初始器呈现；嵌套构造实参位已由前切片闭合——**唯构造实参位的数组链缺口**。

## What Changes

- `init.rs` 构造走查接受实参位的内联匿名数组存储链：外层实参扫描遇 `anewarray` 起始的完整链（元素生产与既有 varargs 内联数组判据同源、区间连续、值单用途为外层该实参）时证明之并跳过区间；呈现复用 `new T[]{…}` 初始器拼写于实参位。
- W3.viaArrays/viaArraysEmpty、W1.use 恢复且行为一致；裸调用位、嵌套构造位、普通调用实参位逐字不变；链不完整/双用途/跨块保持拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：构造实参位的 varargs 内联数组可呈现为数组初始器实参。

## Impact

仅 `crates/jarde-java` 私有 init.rs 走查与呈现及测试；判据与既有 varargs 内联数组证明同源复用，无新机制。既有构造/varargs 切片零回退。
