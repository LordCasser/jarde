# 验收记录

root 审阅了 `src/class_source.rs::resolve_default` 的新增 `ElementValueFacts::Annotation` 分支：只接受可拼写的 `L...;` 类描述符和逐个有效 Java 成员名，按属性顺序递归取得子值；空/多成员分别写 `@Type()` 与完整命名赋值。数组仍通过 `Option<Vec<_>>` 全有或全无地提交；失败后代、数组/原始类型描述符及非法名字都不输出半个默认值。reader 属性解析和 class-source 公开 JSON 结构不变。

root 在代理修复后重建并冻结 CLI，SHA-256 为 `7f9dd55bb020d0308504a9484ad44b157b271f08b1b96c667e538f935c6a38d5`，独立从固定原源码重新编译并核对 class SHA，再原样生成 Original、JADX、Jarde 的完整 Inner/Nested/NestedRunner。三方 Java 8 编译和 `java -Xverify:all` 反射均输出 `6\n2\n`；Basic 三方仍输出 `5\nsource-only\n[2, 4]\n`。证据在 `../../evidence/java-syntax-2026-09-22/annotation-default-boundaries/header-minimal/post-fix-nested-root-replay/`；修前可编译但反射 NPE 的证据留在相邻 `post-fix-root-replay/`。

门禁：`p3_annotation_default` 6/6、`class_source` 16/16、`p5_corpus_fingerprint` 5 通过/1 ignored、reader 160/160。默认/完整 evidence 文本、原始 class 身份、不可拼写后代、属性预算与预取消均有定向测试。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate --all --strict` 63/63 均通过。根库 `--lib` 和 `p3_annotation_default` 的严格 Clippy 在排除既存 `region.rs:1736` 类型复杂度后通过；全目标 Clippy 未全绿：不启用 `test-support` 会触发已有测试的 feature 门控，启用后在无关 `tests/p3_numeric_comparison.rs:630` 的 `useless_conversion` 停止。没有把这处 lint 混入嵌套默认值实现。`p5_bulk_corpus` 的三项计数/分类基线漂移仍是独立维护债务。
