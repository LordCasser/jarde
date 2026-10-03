## Context

[sif 取证](../../evidence/java-syntax-2026-10-03/single-interface-fold-patrol/sif/README.md)：两门实测（根重投影门/ token 锚定门）。lambda 内联产物在 `root.text` 中由 `plan_class_source_lambda_inline`/`emit_class_source_lambda_member`（report.rs）产生；折叠 context 构造位在 facade 装配。**第一个取证义务**：(a) 列全折叠 context 置 `None` 的投影输入集合与各自在报告/侧车中的既有载体（lambda 内联决策是否已可从 `ClassSourceReport` 重建）；(b) 评估两案——**案 1 保留**：报告携带投影输入（serde 新可选字段，消费方仅折叠通道）；**案 2 重建**：折叠 context 以与首轮相同的输入重建（要求首轮输入可确定地重取）——以 Y1 + corpus 全族两案对比择一；(c) token 锚定门的覆盖段问题（`Y1$StrFn local1` 段仅 `astore` 无 CP 索引）是否随案 1/2 自然消解（重投影一致后覆盖段由首轮投影锚定）或需独立健全锚补丁。

## Goals / Non-Goals

**Goals:** Y1 族折叠且 lambda 内联保留；重跑不降级不变量测试钉死；无 lambda 根家族零回退。**Non-Goals:** 孙代；接口方法引用锚定（另片已立）；匿名类内联；泛型 Signature。

## Decisions

1. **案择一（取证驱动）**，判据：serde 兼容性、消费面窄度、重跑等价可证性；两案均须 Y1 双呈现（折叠嵌套 + lambda 内联）同时成立。
2. **不变量**：折叠产物二次运行 diff 断言（不劣于首轮）；首轮投影输入保留为事实（非重算）。
3. **验收锚定**：Y1、`Y1M`（类子+lambda）、corpus 该族全量；负例（数组/枚举投影根各自验一形，如实登记非 lambda 形的现状）。

## Risks / Trade-offs

- **报告膨胀** → 投影输入按需携带（仅折叠候选根）；计费不变。
- **两案都不达 Y1**（token 锚定门独立阻塞）→ 按取证 (c) 如实报告，锚定补丁并入本片或另片——不静默绕过门。
