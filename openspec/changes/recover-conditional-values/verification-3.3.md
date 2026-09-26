# Root 门禁与结构计数

2026-09-26 在隔离 checkout `bfad1b9c` 运行 `cargo fmt --all -- --check`、`cargo test --test p5_corpus_fingerprint --locked`（5 通过、1 个显式再生成测试 ignored）及 [3.2](verification-3.2.md) 所列条件值、deferred 和 switch 定向测试，均通过；`openspec validate recover-conditional-values --strict` 通过。

Reader 的全 fixture 结构普查初次失败：实际 `(316 class, 1674 Code, 153 handler, 902 branch/switch target, 8 jsr)`，旧断言为 `(310, 1645, 144, 880, 8)`。逐项追溯后，`965d8cf5` 的两份 `ConditionalFieldWrites` class 增加 `(2, 10, 0, 12, 0)`，`54c02b9a` 的四份 precise-rethrow class 增加 `(4, 19, 9, 10, 0)`；相加恰为差额。已在 `9a0c2a11` 更新 `crates/jarde-reader/src/classfile.rs` 的解释与断言及 `tests/fixtures/README.md` 汇总。隔离 checkout 重跑 `repository_class_fixtures_validate_without_false_target_rejections` 通过，fmt、指纹和 `git diff --check` 通过。只重录结构门禁，未改 fixture 字节或本条件值证明。

本变更留下的一般 Phi、异常区域及局部作用域问题仍是独立债务，不纳入条件值规则的准入。
