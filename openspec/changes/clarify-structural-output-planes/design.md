## Context

四条缺口与证据见 [proposal](proposal.md)。它们的共同点：报告本身没有错，错的是**读法**。载荷里的 planes（representation/quality/syntax/compile/semantic/verification/coverage/execution）以及恢复侧的 content 各自只回答自己的结构性问题；Produced 只说明产物已交付；诊断码是「该版本在哪次运行里记录了哪个事实」的身份；计数字段是「本次请求在其声明范围内实际做了多少」的代理。四处证据都来自同一轮 benchmark 并已由本人复现，本 change 只把这些读法写入拥有它们的文档与 spec。

依赖关系：第 1 条使用 [fix-nested-arithmetic-value](../fix-nested-arithmetic-value/proposal.md) 的反例结论（`Structured` 产物可以算错值）。本 change **不依赖它的代码**，也不改 `crates/`；只有在 fix change 的根因结论已确定（或至少反例事实已固定）时才写入对应的措辞，避免把未定的根因写进契约。

## Goals / Non-Goals

**Goals:**

- 把四条读法写进合适的所有者：README、支持矩阵、benchmark 协议与拥有这些平面的 capability。
- 明说哪一条可以机械断言、哪一条只能编辑性复核，不伪造散文测试。
- 给调用方明确动作：执行受控比较，或把产物当作假设。

**Non-Goals:**

- 不改任何报告取值、分类逻辑、退出状态或 CLI 输出。
- 不新增 crate、依赖、verifier、报告平面或字段；不新增断言散文的测试。
- 不声称一般语义等价；不重开 R8/R9 或已归档的停止传播修正；不做性能工作；不修 body 解码重新解析类的债务。
- 不重写历史 benchmark 数字或旧报告；只登记读法与所需声明（引擎版本、code 词汇、请求形状）。

## Decisions

### 1. 第 1 条（平面是结构性的）落在 `recovery-validation` 与 `java8-recovery`

`recovery-validation` 的 `Separate representation and validation statuses` 是「不同承诺」的所有者：在该 requirement 里写入「任意平面组合都不构成语义等价声明」，并新增两个 scenario（Structured 产物可以算错；调用方必须执行受控编译/执行对照或把产物记为假设）。`java8-recovery` 的 content requirement 负责「content 也不证明语义等价」这一半。README 的恢复章节与支持矩阵的 decompile-quality/output-level 说明按同一句子同步，不引入新说法。

调用方动作写清楚：受控对照用仓库既有 P3 3.3 流程（`tests/p3_execution_comparison.rs`：原 class 与呈现文本在同一输入上执行比较）；不做对照时，产物是假设，其 `semantic_validation` 保持 `Unproven`、`verification` 保持 `NotPerformed`。

### 2. 第 2 条（Produced ≠ 含语句）落在 `java8-recovery` 的 content requirement

在该 requirement 里明确：`Produced` 与 `content` 分开统计与表述，182,883 个请求中 30,452 个是 Produced 但产物不含语句；报告与文档 MUST NOT 把这类计数表述为语句恢复率或语义恢复率。新增 scenario 覆盖「只报一个数」与「把 produced 当语句数」两种误读。已有的 `recovery-validation` 覆盖指标 requirement（「MUST NOT 把 Produced 比例称为语句恢复率」）保持不变，本 change 不改它的文本，只在 benchmark 协议里补同样的表述与旧/新口径标注要求。

### 3. 第 3 条（诊断码是版本词汇）落在 `analysis-contracts`

以 ADDED requirement 记入：code 是「该版本记录的事实」的身份；语义变化必须登记为词汇变化；跨版本计数 MUST NOT 被读作质量改善。证据写进 requirement 的陈述：`jre_declaration_class_not_in_run` 从单个 artifact 的 10,720 次降到语料全局 0 次，同时 `jre_declaration` 出现在每个 artifact 上；比较报告必须带引擎版本与 code 词汇。支持矩阵与 README 的诊断/声明说明引用同一条规则；benchmark 协议要求每份比较记录引擎 SHA（协议已有冻结基线一节，补「code 词汇」一项）。

### 4. 第 4 条（计数字段按形状解释）落在 `measured-execution`

以 ADDED requirement 记入：每个计数字段必须说明「数的是什么、在哪个 scope 里数」；`archive_entries` 在 standalone class、flat jar、WAR 单 container、整树遍历下含义不同（实测 28 与 29、约 6,009、500–1,296、3,702，其中 container tree root 会走子容器）；比较行必须声明请求形状、声明的 roots/profile 与引擎版本，MUST NOT 跨形状直接比较。支持矩阵的 P5 章节与 benchmark 协议的「记录列（固定）」按此补一行说明。

### 5. 机械断言与编辑性条目

明确分类，避免两边的假证据：

| 条目 | 机械锚点（既有测试） | 本 change 的文档陈述 |
| --- | --- | --- |
| 1 平面结构性 | 无独立机械测试；`tests/p3_eval_context.rs` 的 R8/R9 断言只覆盖各自反例。**真正的机械证明由 `fix-nested-arithmetic-value` 的执行对照提供**（依赖其落地） | 编辑性：四个落点的句子与调用方动作，由本 change 的 verification 逐处复核 |
| 2 Produced/content | `tests/p3_content.rs::explanation_only_means_no_emitted_statement` 及其同族用例已断言 explanation_only 与 Produced 同时成立、`a_stopped_run_reports_not_produced` 断言停止为 not_produced | 编辑性：计数口径与不得称为语句恢复率的表述 |
| 3 诊断码词汇 | `tests/p3_declaration_handoff.rs` 已断言交付声明事实后 `jre_declaration_class_not_in_run` 不出现、`jre_declaration` 出现（另有缺事实时保持 `jre_declaration_class_not_in_run` 的用例） | 编辑性：跨版本读法、比较需带版本与词汇 |
| 4 计数按形状 | `tests/p2_entry_counts.rs`、`tests/p1_artifact_tree.rs`、`tests/p5_container_lookup.rs`、`tests/p5_benchmark.rs` 断言各自形状下的 usage/计费 | 编辑性：跨形状不可比与比较行必须声明形状/声明/版本 |

本 change **不新增**断言散文的测试：文档存在性属于 reviewable statement，由 verification 的复核清单逐项核对（四个落点各一条句子），机械部分只是把上表既有测试重跑一遍作为行为未变的对照。

## Risks / Trade-offs

- **文档说了、代码没跟上** → 第 1 条的机械证明依赖 fix change；在它落地前，措辞写成「已知反例」而不是「已修复」，并在 verification 记录该依赖。
- **四处各说各话** → 同一句子只在一个所有者里定义，其余落点引用它；benchmark 协议只补记录要求。
- **把编辑性条目包装成机械门禁** → design 已给出分类表；tasks 中的门禁只跑既有测试与 `openspec validate`，不新增散文断言。
- **历史数字被改写** → 只登记读法与证据引用，不改写历史 benchmark 报告与归档记录。

## Migration Plan

按所有者顺序落地：先在 spec delta 写清四条契约，再同步 README、支持矩阵与 benchmark 协议；随后跑既有机械锚点测试与 `openspec validate`，在 verification 里逐条记录四个落点的复核结果与未落地依赖（fix change）。本 change 无运行期迁移、无回退风险；若某条措辞与 fix change 的最终结论不符，修正措辞而不是放宽该条。

## Open Questions

无。四个落点与措辞边界由本 design 决定；唯一外部依赖是 fix change 的反例结论，已用「已知反例」措辞与其解耦。
