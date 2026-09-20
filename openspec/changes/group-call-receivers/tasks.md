## 1. 复现与定位

- [ ] 1.1 反例先行（修正前完成）：用受控 `javac --release 8 -g:none` 样本 `return (a + b).substring(1);`，经公开入口 `Engine::recover_method`（或 `jarde-cli recover`，记录确切命令）复现正文 `return arg0 + arg1.substring(1);`，记录报告平面（`Complete`/`Structured`/`ContainsStatements`、无拒绝诊断）与本次运行事实；把原 class 与该正文分别放进各自正确签名编译执行，输入 `("a", "bc")`，记录原值 `"bc"` 与恢复值 `"ac"`。反例、命令与两侧值进 verification（A13）。
- [ ] 1.2 位置枚举：从当前 `crates/jarde-java/src/emit.rs::Emitter::expr` 的代码列出所有写入子表达式的位置（二元左/右、调用与构造实参、调用 receiver、字段读取 receiver、方法引用限定符、`array[index]` 的 array 与 index、`!` 操作数、lambda 体），逐条给出：当前是否补括号、该位置能否承载会被拼接重组的子式（可达性论证与代码证据）、由哪条检查覆盖。MUST NOT 只修 Call 分支后声明该类关闭（A13）。
- [ ] 1.3 记录证据边界：写出 `binary_operand` 只被 Binary 臂调用的事实（文件与行号），并明确归档 `fix-nested-arithmetic-value/verification.md` 的覆盖面清单不是接收者位置的分组证据；不修改归档记录本身（A13）。

## 2. 实现

- [ ] 2.1 在 `crates/jarde-java/src/emit.rs` 实现按位置决定的分组：接收者位置纳入同一判定，`return (a + b).substring(1);` 的呈现为 `(arg0 + arg1).substring(1)`；不新增公共类型、不改 `ExprKind`/私有 AST、不改 `build.rs` 的值/求值点/origin 逻辑；括号写在子式自己的节点上，段表语义不变（A13）。
- [ ] 2.2 不需要分组的位置 MUST 不新增括号：调用/构造实参、下标与 lambda 体保持原文本；既有 `binary_operand` 规则（含同优先级右操作数与同优先级左操作数）逐字段不变，由既有精确文本回归核对（A13）。
- [ ] 2.3 无法证明分组可安全表达时走既有 refusal（保留 bytecode 与 origin、`Representation::Mixed`/`Quality::Fallback`、诊断点名 BCI 与物理方法），不以「没有生成 Java」通过（A13）。

## 3. fixture、回归与变异

- [ ] 3.1 提交 `tests/fixtures/p3-receiver-grouping/`：`ReceiverGrouping.java`（含缺陷形状与正向对照）、`v8/ReceiverGrouping.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码，沿用 `p3-nested-arithmetic` 格式）、`Baseline.java`（原 class 的驱动，打印输入集合的原值，含 `("a", "bc")`）（A13）。
- [ ] 3.2 语料登记：在 `tests/fixtures/README.md` 的 P3 real compiled samples 表新增一行（命令、产物与摘要、被哪些测试读取）；执行 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过，并记录文件数变化（A13）。
- [ ] 3.3 更新 reader fixture census：`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections` 计数按实测更新（预期新增 1 个 class 与其 body，handler/branch/jsr 不变；以实测为准并显式记录）。门禁：`cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections`（A13）。
- [ ] 3.4 常备回归（精确文本与正向对照）：在 `tests/p3_eval_context.rs` 增加用例，断言缺陷正文不再出现（文本 MUST NOT 含 `arg0 + arg1.substring(1)`）、接收者文本恰为分组形态，并断言正向对照逐字不变（`arg0.length()`、嵌套调用、同优先级内部子式只保留必要的一层括号）。门禁：`cargo test --test p3_eval_context --locked`；修正前该用例必须在这些断言上失败（A13）。
- [ ] 3.5 执行对照：把 fixture 作为 `Sample` 登记进 `tests/p3_execution_comparison.rs`（`bytes`/`members`/`baseline` 与 `REQUIRED` 登记）；输入集合 MUST 含一个使两侧可观察结果不同的输入（已实测 `("a", "bc")`：原值 `"bc"`、缺陷值 `"ac"`），既有 `sample_values` 覆盖不到时在该文件内为本样本扩展输入并记录；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`。修正前该对照 MUST 在分歧输入上以值不同失败，且失败行被记录（A13）。
- [ ] 3.6 变异：去掉接收者位置的分组（其余不动）后重跑 3.4/3.5，精确文本断言与执行对照 MUST 变红，且失败现象点名该输入与成员；恢复变异后 `grep` 无调试残留、源码与修正版一致（A13）。

## 4. 门禁与收尾

- [ ] 4.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录摘要与 ignored 数量（A13）。
- [ ] 4.2 相关既有回归：`cargo test --test p3_local_rewrite --locked`、`cargo test --test p3_content --locked`、`cargo test --test p3_isolation --locked`、`cargo test --test p5_corpus_fingerprint --locked` 通过；确认既有算术、content 分类、隔离边界与语料指纹未被本次修正改变（A13、A17）。
- [ ] 4.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（反例与两侧执行值、位置枚举与可达性证据、文本前后对照、fixture 摘要、census/fingerprint 计数、执行对照、变异、门禁结果）；未完成前不归档、不更新完成判断。
