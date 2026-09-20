## 1. 复现与定位

- [x] 1.1 反例先行（修正前完成）：用受控 `javac --release 8 -g:none` 样本复现 `[B local1 = arg0;` 与 `[Ljava.lang.String; local1 = arg0;`（数组参数填进局部），记录命令、恢复正文、报告平面与无拒绝的事实；把正文放进成员自己签名的 wrapper 后用 `javac --release 8` 编译，记录 `[B` 处的确切拒绝信息（A13）。
- [x] 1.2 类型位置枚举：从当前代码列出所有把 descriptor/类型写成 Java 文本的位置（`declare` → `Type::spell` 经 `value_type`/`spell_reference`；`ExprKind::New { ty }`；`ExprKind::Path(owner)`；`spell_reference` 的全部调用点），逐条记录：是否需要合法 Java 类型拼写、当前是否会收到数组 descriptor（可达性论证与代码位置）、由哪条检查覆盖。MUST NOT 只改声明路径后声明该类关闭（A13）。
- [x] 1.3 记录拼写现状：写出 `build.rs::spell_reference` 只处理 `L…;` 的事实（文件与行号）、帧把数组 descriptor 放进 `RefType::Named` 的证据（`parse_field_type` 的 `[` 分支）、以及 `lambda.rs::parse_type` 已有的正确数组拼写；说明本次采用「合并成一条」还是「两处由测试固定一致」的理由（A13）。

## 2. 实现

- [x] 2.1 实施数组拼写：`[` 前缀计数维度、递归拼写元素、补 `[]`；对象元素沿用既有拼写。`[B` → `byte[]`、`[Ljava/lang/String;` → `java.lang.String[]`、`[[I` → `int[][]`、`[[Ljava/lang/String;` → `java.lang.String[][]`；与 `lambda.rs` 的同一拼写一致（合并或由测试固定），不新增类型模型、不改 `value_type` 对非数组类型的决定（A13）。
- [x] 2.2 不接收数组 descriptor 的位置 MUST 行为不变：`ExprKind::Path(owner)`、`ExprKind::New { ty }` 的对象名拼写与 `text(Ljava/lang/String;)Ljava/lang/String;` 对照逐字不变（由 3.6 的精确文本核对）（A13）。
- [x] 2.3 无法拼成合法 Java 类型的输入走既有 refusal（保留 bytecode 与 origin、Mixed/Fallback、诊断点名 BCI 与物理方法），MUST NOT 原样输出，也 MUST NOT 用 `Object` 之类占位符替换已读到的类型（A13）。

## 3. fixture、回归与变异

- [x] 3.1 提交 `tests/fixtures/p3-array-types/`：`ArrayTypes.java`（`echoed`/`named`/`grid`/`table` 与对照 `text`，加 `copy` helper）、`v8/ArrayTypes.class`（`javac --release 8 -g:none` 的真实输出）、`README.md`（编译器版本与确切命令、字节数、SHA-256、逐成员字节码、修正前 javac 的拒绝）、`Baseline.java`（原 class 的驱动，打印输入集合的原值）（A13）。
- [x] 3.2 语料登记：在 `tests/fixtures/README.md` 的 P3 real compiled samples 表新增一行；执行 `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`，随后 `cargo test --test p5_corpus_fingerprint --locked` 通过，记录文件数变化（A13）。
- [x] 3.3 更新 reader fixture census：`crates/jarde-reader/src/classfile.rs` 的 `repository_class_fixtures_validate_without_false_target_rejections` 计数按实测更新（以实测为准并显式记录）。门禁：`cargo test -p jarde-reader --lib --locked repository_class_fixtures_validate_without_false_target_rejections`（A13）。
- [x] 3.4 常备回归（精确文本）：新建 `tests/p3_array_types.rs`，断言四个缺陷成员的类型位置为合法 Java 数组拼写、且文本 MUST NOT 含 `[B`、`[[I`、`[Ljava`、`[Lja`；门禁：`cargo test --test p3_array_types --locked`；修正前该用例 MUST 在上述断言上失败（A13）。
- [x] 3.5 javac 编译对照：把 fixture 作为 `Sample` 登记进 `tests/p3_execution_comparison.rs`（`bytes`/`members`/`baseline` 与 `REQUIRED` 登记）；数组参数按既有 `sample_values` 兜底以 `null` 调用；门禁：`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture`。修正前该对照 MUST 因 wrapper 在 `[B` 处编译失败而变红，且失败文本被记录（A13）。
- [x] 3.6 正向对照：对照成员 `text(Ljava/lang/String;)Ljava/lang/String;`（对象类型）与基本类型位置的样本在修正后文本逐字不变，并继续通过 3.5 的编译对照；`ExprKind::Path`/`ExprKind::New` 的对象名拼写同样逐字不变。MUST NOT 用普遍改写拼写换取反例通过（A13）。
- [x] 3.7 拒绝路径验收：对 1.2 判定为「无法拼成合法 Java 类型」的输入（若存在），核对拒绝范围、被引用 BCI 与物理方法映射；不得以「没有生成 Java」通过（A13）。
- [x] 3.8 变异：把数组处理恢复成原样通过（只处理 `L…;`）后重跑 3.4/3.5/3.6，精确文本断言与编译对照 MUST 变红；恢复变异后 `grep` 无调试残留（A13）。

## 4. 门禁与收尾

- [x] 4.1 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全部通过；记录摘要与 ignored 数量（A13）。
- [x] 4.2 相关既有回归：`cargo test -p jarde-java --lib --locked`（lambda/描述符解析的单测）、`cargo test --test p3_eval_context --locked`、`cargo test --test p3_declaration_handoff --locked`、`cargo test --test p5_corpus_fingerprint --locked` 通过，确认既有拼写、声明与语料指纹未被本次修正改变（A13、A17）。
- [x] 4.3 `openspec validate --all --strict --no-interactive` 通过；写本 change 的 verification（两个反例与 javac 拒绝、类型位置枚举与可达性、修正前后精确文本、fixture 摘要、census/fingerprint 计数、编译对照、对照成员不变、变异、门禁结果）；未完成前不归档、不更新完成判断。
