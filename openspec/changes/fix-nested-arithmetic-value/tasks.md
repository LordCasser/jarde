## 1. 诊断与修正

- [ ] 1.1 在固定行为基线上确认反例：用受控源码与字节复现四行 `local1 = local1 * 2 - arg0 * local1;`，记录 `quality=structured`、`content=contains_statements`、`execution=complete`、无诊断；确认此时呈现文本可编译且算错（不是拒绝），并记录 `d = -1` 的原值与恢复值（A13）。
- [ ] 1.2 在本次运行的 SSA 值表上诊断根因：核对每块算术的操作数定义、消费指令与实际求值点，写出缺陷发生的边界（操作数枚举、求值点传递或渲染映射）并附 `ValueId`/BCI 与指令序列证据；结论只按证据陈述，不得只按文本形态下判断（A13）。
- [ ] 1.3 实施最小修正：算术操作数使用自己的值与求值点，保留结果嵌套；不能证明的操作数走既有 refusal（引用 bytecode、保留 origin、Mixed/Fallback）；只在既有私有 AST 内做最小必要表示。核对 `(x + 1) + (x + 2)`、`p3-local-rewrite` 语料与既有 controls 逐字段不变（A13）。

## 2. 受控 fixture 与语料登记

- [ ] 2.1 提交 `tests/fixtures/p3-nested-arithmetic/`：`ModLike.java`（受控源码）、`v8/ModLike.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码，沿用 `p3-nested-eval` 格式）、`Baseline.java`（原 class 的驱动，打印输入集合的原值）（A13）。
- [ ] 2.2 再生成并核对语料 fingerprint：`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过；在 `tests/fixtures/README.md` 的 P3 real-samples 表登记一行，记录新计数（A13）。
- [ ] 2.3 更新 reader fixture census：`repository_class_fixtures_validate_without_false_target_rejections` 的计数按实测更新（预期新增 1 个 class、1 个 body，handler/branch/jsr 不变；以实测为准并显式记录）。门禁：`cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections`（A13）。

## 3. 执行对照与变异

- [ ] 3.1 常备回归（结构化呈现）：在 `tests/p3_eval_context.rs` 增加用例，断言该 fixture 的呈现是 `local1 * (2 - arg0 * local1)` 的嵌套形态而不是 `local1 * 2 - arg0 * local1`，并保留拒绝路径的边界断言；门禁：`cargo test --test p3_eval_context --locked`（A13）。
- [ ] 3.2 执行对照：把 fixture 与 `Baseline.java` 接入 `tests/p3_execution_comparison.rs` 的既有流程（从本次事实派生声明、`javac --release 8` 编译呈现文本、两侧 trace 逐行比较），输入集合含 `d = -1` 与至少一个正向输入；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`，修正前该对照必须以值不同失败（A13）。
- [ ] 3.3 拒绝路径验收：若修正选择拒绝该区域，核对被引用 BCI、物理方法映射与 Mixed/Fallback；不得以「没有生成 Java」通过（A13）。
- [ ] 3.4 变异：恢复错误呈现（例如去掉外层运算的呈现）后重跑 3.1/3.2，至少一个输入必须因值不同变红；恢复变异后树中不遗留调试改动（A13）。

## 4. 门禁与收尾

- [ ] 4.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录输出摘要与 ignored 数量。
- [ ] 4.2 相关既有回归：`cargo test --test p3_local_rewrite --locked`、`cargo test --test p3_content --locked`、`cargo test --test p3_isolation --locked` 通过，证明修正未改变既有算术、content 分类与隔离边界（A13、A17）。
- [ ] 4.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（反例、根因证据、fixture 与摘要、census/fingerprint 计数、执行对照、变异、门禁），同步受影响的公开状态；未完成前不归档、不更新完成判断。
