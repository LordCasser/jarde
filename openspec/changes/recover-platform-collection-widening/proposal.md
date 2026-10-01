## Why

[集合层级巡查](../../evidence/java-syntax-2026-10-02/collection-widening-patrol/README.md)确认：调用实参的集合实现类到接口上转型（`ArrayList → List`，任何把集合传给以接口声明的形参——集合 API 最常见形态）被 `platform_reference_argument_widens` 拒绝：当前白名单仅 `List → Iterable` 一对。与已合入的 `java_lang_throwable_widens`（recover-throwable-wrap-arguments 的 45 对闭集 + 传递闭包）同通道同形态，只是缺 java.util 集合层级表。级联效应：被拒调用的结果局部随之拒绝（G1.use 固定复现）。

## What Changes

- `platform_reference_argument_widens` 的 java.util 集合闭集：JDK 8 固定层级直接边（ArrayList/LinkedList/Vector→List；HashMap/TreeMap/LinkedHashMap/Hashtable→Map；HashSet/TreeSet/LinkedHashSet→Set；ArrayDeque→Deque；接口链 List/Set/Queue→Collection→Iterable，含 List→Queue 之外的既有精确对），同款传递闭包 walk；逐对以 JDK 运行时反射机械核对。
- G1.use 完整恢复（`max(local0)` 呈现，结果局部可用）；`List→Iterable` 既有行为不变；用户类与非 java.util 不放宽。
- 以 G1 与变体族（Map/Set/Queue 各型、嵌套泛型调用、接口形参多重上转型 `ArrayList→Collection`）验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：java.util 集合层级的调用实参上转型可呈现，集合 API 调用完整恢复。

## Impact

仅 `crates/jarde-java` 私有 build.rs 白名单函数及测试；复用 throwable 闭集先例（表 + walk），无新机制。既有转换回答零回退。
