## 1. 审核 corpus 差异

- [ ] 1.1 按文件列出当前 146 个未登记项，记录用途、来源、测试消费者、生成方式和保留/移除结论；以 `cargo test --test p5_corpus_fingerprint --locked -- --nocapture` 的未登记列表为基线完成核对。
- [ ] 1.2 核验 reader 扫描的 310 个 class，并单独裁定三个 source-only runner 候选；通过查看引用、说明和重建过程形成可复核结论，不因文件名自动删除。

## 2. 更新 reader census

- [ ] 2.1 只在 corpus 审核和候选裁定完成后，按最终保留的 class 集重新测量 class、body、handler、branch-target 和 subroutine 指标并更新 reader 测试断言；用 `cargo test -p jarde-reader --lib` 验证。

## 3. 更新 fingerprint

- [ ] 3.1 更新 `tests/p5_corpus_fingerprint.rs` 分类表以准确表达已审查的输入及来源；核查 146 项的每个保留/移除决定都体现在分类变化中。
- [ ] 3.2 从已审查的分类表运行专用忽略再生成器并审阅 `tests/fixtures/corpus-fingerprint.json` 差异；运行 `cargo test --test p5_corpus_fingerprint --locked -- --nocapture` 验证清单与分类表一致。

## 4. 复核门禁

- [ ] 4.1 复跑 `cargo test -p jarde-reader --lib` 和 `cargo test --test p5_corpus_fingerprint --locked -- --nocapture`，确认两项门禁通过，并保留其结果与分类记录。
