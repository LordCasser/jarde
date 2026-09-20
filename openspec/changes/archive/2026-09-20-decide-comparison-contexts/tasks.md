## 1. 复现与证据（修正前完成）

- [x] 1.1 反例先行：用受控 `javac --release 8 -g:none` 样本复现三个形状——`if (1 == n)`、`if (0 < n)`、`if (1 < n)` 分别恢复成 `if (true == arg0)`、`if (false < arg0)`、`if (true < arg0)`；记录命令、报告平面（outcome/representation/quality/content）与无拒绝的事实（A13）。
- [x] 1.2 把三段正文分别套进成员自己的签名（`public static int oneFirst(int arg0)` 形态，由本次运行的事实派生），用 `javac --release 8` 编译，记录 javac 的确切拒绝信息。证据进 verification（A13）。
- [x] 1.3 回归归属核对：确认 `fa6dc6e`（boolean 修正之前）对这三个形状输出整数文本、回归由 `5a8c36a` 引入，并把它写进 proposal 与 verification 的同一措辞，MUST NOT 表述为新发现的缺陷（A13）。
- [x] 1.4 边界记录（design 决策 3）：用一个人工字节样本记录「被证明 boolean 的操作数与 int 字面量组成的 `Test::Pair` 比较」今天的文本与其 javac 处置；写明它既不在本 change 的闭环范围，也不得被当作已闭环；若本 change 的改动使它变差，按本 change 的回归处理（A13）。

## 2. 实现

- [x] 2.1 在 `crates/jarde-java/src/build.rs::condition` 把「该位置是否要求 boolean」的判定放到任一操作数的渲染之前：只有零值测试且操作数被证明 boolean 的真值测试分支才允许映射字面量；`Test::Pair` 的两侧一律按值渲染（`render_value`），MUST NOT 复用为真值测试准备的拼写（A13）。
- [x] 2.2 逐条核对 design 决策 2 的表：真值测试、`Z` 返回、写入已证明 boolean 的变量三处仍映射字面量；未证明 boolean 的零值测试保持整数比较；真正 `int` 返回保持 `1`/`0`；`Test::Pair` 的常量在左与在右写出同一形态（A13）。
- [x] 2.3 核对不改动面：`boolean_value`/`boolean_evidence`/`boolean_literal`/`boolean_local` 的证据清单、`value_type` 对其它类型的决定、`ExprKind`/私有 AST 与 `emit.rs` 的打印规则逐字段不变（由 4.4 与 5.2 的既有回归核对）（A13）。

## 3. 受控 fixture 与语料登记

- [x] 3.1 提交 `tests/fixtures/p3-int-comparisons/`：`IntComparisons.java`（三个缺陷形状 + 常量在右/另一比较方向/变量比较的 int 对照 + 前一轮的 `Z` 返回、`if (arg0)`、`boolean local1 = …` 对照）、`v8/IntComparisons.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码、修正前的恢复正文与 javac 拒绝）、`Baseline.java`（原 class 的驱动，打印输入集合的原值）（A13）。
- [x] 3.2 语料登记：在 `tests/fixtures/README.md` 的 P3 real compiled samples 表新增一行；执行 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过，记录文件数变化（A13）。
- [x] 3.3 更新 reader fixture census：`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections` 元组按实测更新（新增 class/body 与 branch/handler 变化以实测为准并显式记录）。门禁：`cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections`（A13）。

## 4. 回归、对照与变异

- [x] 4.1 常备回归（精确文本）：在 `tests/p3_boolean_contexts.rs`（该目标已拥有这条规则的精确文本回归）增加用例，断言三个缺陷形状的文本是整数字面量形态且不含 `true ==`、`false <`、`true <`；修正前该用例 MUST 在文本断言上失败。门禁：`cargo test --test p3_boolean_contexts --locked`（A13）。
- [x] 4.2 编译与执行对照：把该样本作为 `Sample` 登记进 `tests/p3_execution_comparison.rs`（`bytes`/`members`/`baseline` 与 `REQUIRED` 登记），输入集合覆盖常量在左、常量在右、相等与大小比较的每个成员；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`。修正前该对照 MUST 在该成员被 javac 拒绝处失败（记录拒绝文本），不得以「跳过该成员」通过（A13）。
- [x] 4.3 正向对照（前一轮形状与 int 形态）：样本内与前一轮语料的 `return true;`、`if (flag())`、`if (arg0)`、`boolean local1 = …`、`answer()I`、`nonzero(I)I` 在修正后逐字不变并继续通过 4.2 的对照；MUST NOT 只以 `n != 0` 一条证明 int 对照不变（A13）。
- [x] 4.4 变异：把字面量映射放回测试形态判定之前（恢复 `5a8c36a` 的渲染顺序）后重跑 4.1/4.2/4.3；至少一个已提交检查 MUST 变红，且失败现象是该形态的精确文本不符或 javac 拒绝；恢复变异后 `grep` 无调试残留、文件哈希复原（A13）。

## 5. 门禁与收尾

- [x] 5.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录输出摘要与 ignored 数量（A13）。
- [x] 5.2 相关既有回归：`cargo test --test p3_boolean_contexts --locked`、`cargo test --test p3_eval_context --locked`、`cargo test --test p3_array_types --locked`、`cargo test --test p5_corpus_fingerprint --locked` 通过，确认既有 boolean 形状、分组、类型拼写与语料指纹未被本次修正改变（A13、A17）。
- [x] 5.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（三个反例与 javac 拒绝、回归归属、边界记录、修正前后精确文本、fixture 摘要、census/fingerprint 计数、编译执行对照、对照集、变异、门禁结果）；未完成前不归档、不更新完成判断。
