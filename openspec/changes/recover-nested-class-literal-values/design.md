## Context

[巡查证据](../../evidence/java-syntax-2026-10-04/nested-class-literal-patrol/README.md)：A12 `nestedLit` 的 BCI 0 `ldc #15 // class A12$Nested` 被引注 "not part of the provable subset"，级联两条（依赖链不有界、saved declaration 未提交）。对照 A10 五形（`A10.class` 全部健康）——差异仅在 CP 名是否含 `$` 嵌套段。**第一个取证义务**：定位类字面量值的准入判据（grep CONSTANT_Class / class literal / `ldc` 的类名分支；`crates/jarde-java` 的表达式证明层），确认它当前对名的要求（顶层形？名==当前类？可解析性检查在哪一步），以及 nested-spelling 片的可拼性事实（`source_spellable_member_row`/`nested_reference_spelling`）是否可直接复用为同一判据源。

## Goals / Non-Goals

**Goals:** 嵌套类名（`Outer$Nested`、多段 `A$B$C`）的类字面量准入；A12/A11 恢复且行为一致。**Non-Goals:** 本地类（`1$Local`）与匿名类（`X$1`）字面量——源码不可拼，保持拒绝；数组类字面量（`int[].class` 形按既有）；泛型参数化字面量（Java 8 无此形态）；呈现拼写规则变更（nested-spelling 片既有口径不动）。

## Decisions

1. **准入判据 = 可拼性事实（复用，不建第二套）**：CP 名可经既有嵌套名判据解析为源码可拼类型（每段为合法标识符、非数字尾段、非本地/匿名形）即准入；顶层名路径不动（现行为保持）。
2. **呈现零新增**：值走既有类字面量呈现通道，文本拼写由 nested-spelling 的规则决定（折叠域内简单名/分离域既有形）——本片只改"能不能证"，不改"怎么拼"。
3. **验收锚定**：主锚为 A12 两形（`nestedLit`/`nestedRecv`——三指令最小形，根引注即 `ldc class A12$Nested`）与 A11 三形（反射读注解全链）；A9.main 为**复合级联形**（根引注 BCI 0/2 即两个嵌套类字面量，其余依赖链/field/local 均为级联）——预期恢复，但若准入后残留独立拒绝（如 `getAnnotation`→`checkcast`→接口调用链的其它证明），如实登记为遗留、不在本片强凑清零；A10 五形逐字不变；负例（本地类/匿名类字面量、不可拼名）保持拒绝。

## Risks / Trade-offs

- **准入放宽误纳匿名/本地形** → 判据含"尾段非纯数字、非 `N$Local` 本地形"排除；负例钉死。
- **与 nested-spelling 的域耦合**（折叠域内拼 `Nested.class` 需折叠成立）→ 折叠不成立时按分离域既有拼写（`A12$Nested.class` 池形可编，因声明在同文本平铺）；两口径都在本片测试内锚定。
- **依赖链级联拒绝残留** → 若准入后仍有 "dependency chain not bounded"（链长/预算），如实报告并单列，不在本片放宽链界。
