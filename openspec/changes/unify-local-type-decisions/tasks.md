## 1. 复现与证据（修正前完成）

- [ ] 1.1 反例先行：用受控 `javac --release 8 -g:none` 样本复现提升形状——`boolean a = b; boolean c; if (n == 0) c = a; else c = b; if (c) …` 被提升为 `int local3` 并赋入 boolean 值，再写成 `if (local3 != 0)`；记录命令、报告平面与无拒绝的事实（A13）。
- [ ] 1.2 把正文套进成员自己的签名（由本次运行的事实派生），用 `javac --release 8` 编译，记录 javac 的确切拒绝信息；另记录 literal-armed 形状的 `int local1; local1 = 1;` 自洽正文作为边界证据（A13）。
- [ ] 1.3 两条路径的证据核对：在当前源码上逐项核对 `declarations()` 的 `boolean_proof` 与 `declare()` 的 `boolean_evidence || boolean_local`（含 `boolean_local` 不跟随值链的边界），并记录「同一值两个答案、结论取决于遍历顺序」的实测（A13）。
- [ ] 1.4 `declare()` 返回值核对：确认 `Ok(None)` 同时承担「没有声明到期」（`build.rs` 约 1621/1628）与「声明失败、fallback 已写」（约 1653/1670），并记录 `write_statement` 在后者之后仍写赋值（A13）。

## 2. 规划与实现

- [ ] 2.1 在既有声明规划里产出按 `LocalVariable` 身份索引的类型结果（design 决策 1/2）：descriptor 事实发起 boolean；「一次对本 run 已判为 boolean 的局部的读取」传播；`0`/`1` 不发起、只在目标确定为 boolean 时适配；其余取帧的既有类型事实（A13）。
- [ ] 2.2 传播实现为有界工作队列（design 决策 4）：每条目在处理前 `charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))` 并 `poll(budget, Some(at))`；预算或取消返回既有 `StopReason`，MUST NOT 新增预算维度或跳过计费（A13、A14）。
- [ ] 2.3 每个写入按决定检查兼容性（design 决策 3）：不只是第一次写入；冲突或未知时终止该结构（既有 refusal 契约：bytecode/origin 保留、Mixed/Fallback、诊断点名 BCI 与物理方法），MUST NOT 发布猜测或类型矛盾的赋值（A13）。
- [ ] 2.4 `declare()` 结果分为可区分的三种（`Declared(ty)`/`NotDue`/`Refused`），`write_statement` 在 `Refused` 之后 MUST NOT 写那条赋值；`NotDue` 的既有语义（参数、已声明、无名）保持不变（A13）。
- [ ] 2.5 消费点接线：提升声明、就地声明、赋值、条件与返回都读同一结果，不再各自判定；`condition` 的真值测试分支读该结果（不重开 [decide-comparison-contexts](../decide-comparison-contexts/design.md) 的顺序决定）（A13）。
- [ ] 2.6 核对不改动面：`value_type` 对其它类型的决定、`reuse::Plan` 的变量身份、`ExprKind`/私有 AST 与 `emit.rs` 的打印规则逐字段不变（由 4.5 与 5.2 的既有回归核对）（A13）。

## 3. 受控 fixture 与语料登记

- [ ] 3.1 提交 `tests/fixtures/p3-hoisted-boolean/`：`HoistedBoolean.java`（`copied`、分支交换的 `swapped`、复制链 `relayed`、边界形状 `literalArmed`、`fromParameter`、`intLocal`、`unproven`）、`v8/HoistedBoolean.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码、修正前的恢复正文与 javac 拒绝、literal-armed 的边界说明）、`Baseline.java`（原 class 的驱动，打印输入集合的原值）（A13）。
- [ ] 3.2 语料登记：在 `tests/fixtures/README.md` 的 P3 real compiled samples 表新增一行；执行 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过，记录文件数变化（A13）。
- [ ] 3.3 更新 reader fixture census：`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections` 元组按实测更新并显式记录。门禁：`cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections`（A13）。

## 4. 回归、对照与变异

- [ ] 4.1 常备回归（精确文本与拒绝边界）：新建 `tests/p3_hoisted_boolean.rs`，断言提升形状的 `boolean` 声明与使用拼写、`int` 对照逐字不变、literal-armed 形状的自洽文本（边界）、拒绝形状的拒绝范围与 BCI；修正前该用例 MUST 在文本断言上失败。门禁：`cargo test --test p3_hoisted_boolean --locked`（A13）。
- [ ] 4.2 编译与执行对照：把该样本作为 `Sample` 登记进 `tests/p3_execution_comparison.rs`（`bytes`/`members`/`baseline` 与 `REQUIRED` 登记），输入集合覆盖 `copied`/`swapped` 的两个分支与复制链；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`。修正前该对照 MUST 在该成员被 javac 拒绝处失败，不得以「跳过该成员」通过（A13）。
- [ ] 4.3 顺序不变对照：`swapped` 与 `copied` 的类型与使用拼写 MUST 一致（只有分支文本按源码位置不同），两侧执行逐项相同；把「结论随分支顺序改变」写成可失败的断言（A13、A14）。
- [ ] 4.4 变异（两个方向）：(a) 把提升路径退回只读 `boolean_proof`；(b) 让一致性检查只读第一次写入。各自重跑 4.1/4.2/4.3，至少一个已提交检查 MUST 变红（文本不符、javac 拒绝或冲突未被拒绝）；MUST NOT 用其它用例的红代替；恢复后 `grep` 无调试残留、文件哈希复原（A13）。
- [ ] 4.5 正向对照：`fromParameter`、`intLocal`、`literalArmed` 与 `p3-boolean-contexts` 样本在修正后逐字不变并继续通过 4.2 的对照；拒绝形状（`unproven`）保持拒绝（A13）。

## 5. 门禁与收尾

- [ ] 5.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录输出摘要与 ignored 数量（A13）。
- [ ] 5.2 相关既有回归：`cargo test --test p3_boolean_contexts --locked`、`cargo test --test p3_eval_context --locked`、`cargo test --test p3_local_rewrite --locked`、`cargo test --test p5_corpus_fingerprint --locked` 通过，确认既有 boolean 上下文、分组、局部重写与语料指纹未被本次修正改变（A13、A17）。
- [ ] 5.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（两个形状与 javac 拒绝、literal-armed 边界、两条路径的证据核对、`declare()` 返回值核对、修正前后精确文本、fixture 摘要、census/fingerprint 计数、编译执行对照、顺序不变对照、两个变异、门禁结果）；未完成前不归档、不更新完成判断。
