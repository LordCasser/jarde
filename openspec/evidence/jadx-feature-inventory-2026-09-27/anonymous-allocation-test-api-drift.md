# 匿名分配测试与侧车返回值脱节

主线 `3f5a1b3a` 的 `crates/jarde-java/src/report.rs::class_source_anonymous_return_site` 返回 `(Vec<u32>, String, Vec<u32>)`：来源 BCI、目标类型、构造实参 BCI。`crates/jarde-java/tests/anonymous_allocation_candidates.rs:296` 仍按二元组解构。运行 `cargo test -p jarde-java` 时该集成测试在编译期失败，尚未进入测试执行。此问题早于 CF-19 多出口同步块实现，不能归咎于新 guard 逻辑。

独立维护任务：将该测试改为三元解构，并断言第三项与单次匿名构造的实参来源一致；随后运行该集成测试和 `cargo test -p jarde-java`。生产 API、匿名类恢复与同步块恢复均不在这个修正范围内。
