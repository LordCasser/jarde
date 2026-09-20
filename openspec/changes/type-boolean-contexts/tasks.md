## 1. 复现与证据

- [ ] 1.1 反例先行（修正前完成）：用受控 `javac --release 8 -g:none` 样本复现两个形状——`Z` 返回方法（`if (x == 0) return true; return false;`）的正文出现 `return 1;`/`return 0;`；boolean 调用结果作条件的正文出现 `if (flag() != 0)`。记录命令、报告平面（outcome/representation/quality/content）与无拒绝的事实（A13）。
- [ ] 1.2 把两段正文分别套进方法自己的签名（`public static boolean isZero(int arg0)`、`public static int parity(int arg0)` 形态，由本次运行的事实派生），用 `javac --release 8` 编译，记录 javac 的确切拒绝信息（`int cannot be converted to boolean`、`incomparable types: boolean and int`）。证据进 verification（A13）。
- [ ] 1.3 复核「被证明为 boolean」的证据清单：在当前代码上逐条验证 `boolean_parameter`/`parameter_boolean`（参数 `load`）、`Operation::Invoke(CallTarget)` 的返回 descriptor、`iconst_0/1` 字面量、boolean 局部与 `Not` 组合；把最终清单、每条的代码位置与 `Z` 返回形状里合并值（phi/区域）能否携带证据的结论写进 verification。MUST NOT 引入清单之外的类型推断（A13）。

## 2. 实现

- [ ] 2.1 把成员自己的返回类型作为本次运行的事实接入 `crates/jarde-java/src/build.rs` 的构建入口（`report.rs` 的 `build::build` 调用处已可读 `request.facts.method()`）；像 `parameter_types` 一样派生后再交给构建器，MUST NOT 在构建器内重新解析 descriptor 或用文本猜测（A13）。
- [ ] 2.2 `Z` 方法的 `Return` 分支按返回 descriptor 定型：boolean 字面量与已证明 boolean 的值以 boolean 呈现（`return true;`/`return false;`）；没有 boolean 证据的值走既有 refusal（保留 bytecode 与 origin、Mixed/Fallback、诊断点名 BCI 与物理方法）（A13）。
- [ ] 2.3 `condition` 的 boolean 特判接受「返回 descriptor 为 `Z` 的调用结果」等已证明 boolean 的操作数，写成真值测试（`if (flag())`、`if (arg0)`、必要时 `if (!arg0)`）；**未证明 boolean 的操作数保持既有整数比较**（`if (arg0 == 0)`），不得改成拒绝（A13）。
- [ ] 2.4 核对不改动面：`value_type` 对 int/long/引用等类型的决定、`ExprKind`/私有 AST、`emit.rs` 的打印规则与 `typed_arguments` 的既有行为逐字段不变（由 3.5 的正向对照与 4.2 的既有回归核对）（A13）。

## 3. fixture、回归与变异

- [ ] 3.1 提交 `tests/fixtures/p3-boolean-contexts/`：`BooleanContexts.java`（两个缺陷形状 + 三条对照）、`v8/BooleanContexts.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码、修正前的 javac 拒绝）、`Baseline.java`（原 class 的驱动，打印输入集合的原值）（A13）。
- [ ] 3.2 语料登记：在 `tests/fixtures/README.md` 的 P3 real compiled samples 表新增一行；执行 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过，记录文件数变化（A13）。
- [ ] 3.3 更新 reader fixture census：`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections` 计数按实测更新（新增 class/body 与 branch/handler 变化以实测为准并显式记录）。门禁：`cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections`（A13）。
- [ ] 3.4 常备回归（精确文本与拒绝边界）：新建 `tests/p3_boolean_contexts.rs`，断言两个缺陷形状的呈现文本为 boolean 形态（`return true;`/`return false;`、`if (flag())`）且不含 `return 1;`/`!= 0` 形态；若某个形状按 1.3 的结论走拒绝，按拒绝边界断言（范围、被引用 BCI、物理方法映射），不以「没有生成 Java」通过。门禁：`cargo test --test p3_boolean_contexts --locked`；修正前该用例 MUST 在文本断言上失败（A13）。
- [ ] 3.5 编译与执行对照：把 fixture 作为 `Sample` 登记进 `tests/p3_execution_comparison.rs`（`bytes`/`members`/`baseline` 与 `REQUIRED` 登记），输入集合覆盖 `isZero(0)`/`isZero(1)` 与 `parity` 的若干输入；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`。修正前该对照 MUST 在该成员的 javac 拒绝上失败（记录拒绝文本），不得以「跳过该成员」通过（A13）。
- [ ] 3.6 正向对照（int 形态）：`nonzero(I)I`、`answer()I` 与既有 boolean 参数路径 `count(Z)I` 的精确文本在修正后逐字不变，并继续通过 3.5 的编译执行对照；MUST NOT 用普遍改写成 boolean 换取反例通过（A13）。
- [ ] 3.7 变异：分别恢复「按值渲染、不按返回 descriptor 定型」与「只识别 boolean 参数」的行为后重跑 3.4/3.5/3.6；至少一个已提交检查 MUST 变红，且失败现象是该形态的 javac 拒绝或精确文本不符；恢复变异后 `grep` 无调试残留（A13）。

## 4. 门禁与收尾

- [ ] 4.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录摘要与 ignored 数量（A13）。
- [ ] 4.2 相关既有回归：`cargo test --test p3_eval_context --locked`、`cargo test --test p3_local_rewrite --locked`、`cargo test --test p3_content --locked`、`cargo test --test p5_corpus_fingerprint --locked` 通过，确认既有呈现、分类与语料指纹未被本次修正改变（A13、A17）。
- [ ] 4.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（两个反例与 javac 拒绝、证据清单与代码位置、修正前后精确文本、fixture 摘要、census/fingerprint 计数、编译执行对照、int 对照、变异、门禁结果）；未完成前不归档、不更新完成判断。
