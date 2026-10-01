## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/collection-widening-patrol/README.md)：G1.use 的 `max(xs)` 拒绝点即 `platform_reference_argument_widens`（build.rs 约 24386 行，现 `java_release == 8 && List→Iterable` 单对）。先例 `java_lang_throwable_widens`（同文件）：直接边表 + 步长上界 walk + 同名先行排除 + 逐对 JDK 机械核对（运行时 `getSuperclass`/`getInterfaces` 断言）。java.util 层级在 JDK 8 固定（javadoc extends/implements），闭集安全性同源。

## Goals / Non-Goals

**Goals:** java.util 集合闭集（类→接口直接边 + 接口→超接口边），传递闭包判定；G1.use 与变体族恢复且整类可重编行为一致；`List→Iterable` 既有回答不变。**Non-Goals:** 用户类/自定义集合（升级路径：resolution 层证明，触发条件同前）；泛型 Signature 投影（既有边界）；`Collections.unmodifiableXxx` 等工厂返回类型（返回位非实参位）；非集合 java.util 类（Date 等）。

## Decisions

1. **表 + walk 复用 throwable 模式，口径为严表（root 裁定）**：每行是 JDK 8 javadoc 声明的**一条直接边**（class 的 `extends` 父类或 `implements` 接口、interface 的 `extends`），含 `Abstract*` 骨架、`Stack extends Vector`、`Properties extends Hashtable`、`Deque extends Queue`、`SortedSet`/`NavigableSet` 与 `SortedMap`/`NavigableMap` 链——同族真边不裁剪为「15 对」。java.util 超类型图是 DAG（`ArrayList` 同时有 `AbstractList` 与 `List` 两个直接父），throwable 的线性链 walk 不适用，故 walk 改为按层 frontier BFS，步长上界 = 行数，每个父在入队时即与要求类型比较；同名不属上转型（分派前同名分支既有）。逐对机械核对脚本产物入证据（A–E 五项断言）。
2. **呈现沿用 `cast_argument`**：保留要求类型拼写（防重载重定向），与既有平台对一致。
3. **验收锚定**：G1（`zeta:a`）+ 变体（HashMap→Map 传参、HashSet→Set、ArrayList→Collection 双跳、嵌套泛型 `List<List<String>>` 传 `ArrayList<List<String>>`、接口链 List/Set/Queue→Collection、Collection→Iterable）编译运行对照；负例（用户类 `MyList implements List` → List、用户子类、`java.util.concurrent`、`EnumSet`/`IdentityHashMap`、`Date`：保持拒绝，升级路径未启用）。

## Risks / Trade-offs

- **表错对** → javadoc 逐对来源注释 + JDK 反射机械核对全过（A 直接边、B 包边界、C 域内 812 对 walk≡`isAssignableFrom`、D 行集恰为域内直接边、E 域外类型不入 walk）；表外类型回退拒绝。
- **集合工厂/子类复杂化** → 只收固定 JDK 8 java.util 集合类名与接口名；`EnumSet`/`IdentityHashMap`/`PriorityQueue` 等域外实现、`Collections`/`Arrays` 工厂、`java.util.concurrent` 等子包、`Dictionary`/`Date` 等非集合 java.util 类型不收。
- **发布版漂移** → 表按 JDK 8 层书写，`java_release != 8` 直接拒绝；机械核对脚本在 JDK 23 上于 A 处预期失败（`LinkedHashSet → SequencedSet`），该分歧记录为门禁依据。
