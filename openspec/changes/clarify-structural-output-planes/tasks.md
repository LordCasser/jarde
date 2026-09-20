## 1. 契约陈述（spec delta）

- [ ] 1.1 `recovery-validation`：在 `Separate representation and validation statuses` 的 MODIFIED delta 里写入「每个平面只回答自己的结构性问题，任意组合不构成语义等价声明」与调用方动作（受控对照或当作假设）；逐条保留主 spec 原有的 6 个 scenario，再新增 2 个（Structured 是结构声明、调用方动作）。门禁：`openspec validate --all --strict --no-interactive`（验收 A13）。
- [ ] 1.2 `java8-recovery`：在 content requirement 的 MODIFIED delta 里写入 `Produced` 与 `content` 分开统计（含 30,452/182,883 的口径证据）与两种分类口径分别命名；保留原有 4 个 scenario，新增「produced 计数不是语句计数」与「无语句的 Produced 保持可见」两个 scenario。门禁：`openspec validate --all --strict --no-interactive`。
- [ ] 1.3 `analysis-contracts`：ADDED requirement「诊断码是带版本的词汇身份」；写入声明码含义变化证据（`jre_declaration_class_not_in_run` 从 10,720 次降至语料全局 0 次、`jre_declaration` 出现在每个 artifact）与比较必须带引擎版本/code 词汇的规则。门禁：`openspec validate --all --strict --no-interactive`。
- [ ] 1.4 `measured-execution`：ADDED requirement「计数字段按请求形状解释」；写入 `archive_entries` 的形状证据（28/29、约 6,009、500–1,296、3,702，container tree root 会走子容器）与比较行必须声明形状/声明/版本的规则。门禁：`openspec validate --all --strict --no-interactive`。

## 2. 文档同步

- [ ] 2.1 `README.md`：在恢复产物说明与 JSON CLI 恢复示例附近补平面/Produced/content 边界，引用 owner spec 的句子，不引入新说法（与 1.1/1.2 的措辞逐字一致）。复核方式：与 owner spec 句子逐字对照。
- [ ] 2.2 `docs/support-matrix.md`：在 decompile-quality/output-level 说明与 P5 实测边界一节补同样的平面边界与「计数按形状」说明（引用 1.2/1.4）。复核方式：与 owner spec 句子逐字对照，且不改变已有状态词。
- [ ] 2.3 `openspec/benchmark-protocol.md`：在「记录列（固定）」与冻结基线一节补三项记录要求——code 词汇随引擎版本记录、比较行声明请求形状与 roots/profile、计数字段说明其 scope（引用 1.3/1.4）。复核方式：改动只新增记录要求，不改写既有语料表与历史数字。
- [ ] 2.4 四处落点复核：逐处确认句子存在、彼此一致、且与 owner spec 不矛盾；按 design 的「机械断言与编辑性条目」表标注哪些条目有机械锚点、哪些只能复核。MUST NOT 新增断言散文的测试。复核方式：verification 里逐条列出四处落点与对应句子。

## 3. 复核与门禁

- [ ] 3.1 机械锚点重跑，证明文档变更未改行为：`cargo test --test p3_content --locked`、`cargo test --test p3_declaration_handoff --locked`、`cargo test --test p5_corpus_fingerprint --locked` 全部通过（A13）。
- [ ] 3.2 全量门禁：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 通过（本 change 预期无 Rust 改动，运行用于证明这一点）。
- [ ] 3.3 `openspec validate --all --strict --no-interactive` 通过；写 verification（四处落点复核、机械/编辑性分类、对 [fix-nested-arithmetic-value](../fix-nested-arithmetic-value/proposal.md) 反例结论的引用状态）。未确认措辞依赖前不归档。
