## Context

[巡查证据](../../evidence/java-syntax-2026-10-02/collection-widening-patrol/README.md)：G1.use 的 `max(xs)` 拒绝点即 `platform_reference_argument_widens`（build.rs 约 24386 行，现 `java_release == 8 && List→Iterable` 单对）。先例 `java_lang_throwable_widens`（同文件）：直接边表 + 步长上界 walk + 同名先行排除 + 逐对 JDK 机械核对（运行时 `getSuperclass`/`getInterfaces` 断言）。java.util 层级在 JDK 8 固定（javadoc extends/implements），闭集安全性同源。

## Goals / Non-Goals

**Goals:** java.util 集合闭集（类→接口直接边 + 接口→超接口边），传递闭包判定；G1.use 与变体族恢复且整类可重编行为一致；`List→Iterable` 既有回答不变。**Non-Goals:** 用户类/自定义集合（升级路径：resolution 层证明，触发条件同前）；泛型 Signature 投影（既有边界）；`Collections.unmodifiableXxx` 等工厂返回类型（返回位非实参位）；非集合 java.util 类（Date 等）。

## Decisions

1. **表 + walk 复用 throwable 模式**：直接边按 JDK 8 javadoc（实现类的 `implements`、接口的 `extends`）；walk 沿边到根（Collection→Iterable 兜底），步长上界 = 边数；同名不属上转型（分派前分支既有）。逐对机械核对脚本产物入证据。
2. **呈现沿用 `cast_argument`**：保留要求类型拼写（防重载重定向），与既有平台对一致。
3. **验收锚定**：G1（`zeta:a`）+ 变体（HashMap→Map 传参、HashSet→Set、ArrayList→Collection 双跳、嵌套泛型 `List<List<String>>` 传 `ArrayList<ArrayList<String>>`）编译运行对照；负例（用户类 `MyList implements List` → List：保持拒绝，升级路径未启用）。

## Risks / Trade-offs

- **表错对** → javadoc 逐对来源注释 + JDK 反射机械核对全过；表外类型回退拒绝。
- **集合工厂/子类复杂化** → 只收固定 JDK 8 java.util 类名；`java.util.concurrent` 等子包不收。
