## 1. 取证与基线补全

- [ ] 1.1 读 `tests/p3_anonymous_class_facts.rs`、`tests/class_source.rs`、`tests/recover_synthetic_ctor_super_order.rs`（`recompile_and_run`）与 `tests/assert_statement_sugar.rs`（`run_class`/`recompile_and_run`）的 fixture 消费与 Java 执行模式，选定复用哪个 helper（不新建执行框架）；记录 8 个 fixture 各自的 class 清单与 SHA。
- [ ] 1.2 为无基线记录的三个 fixture（`enum-arity`、`package-info-basic`、`short-circuit-left-false`）补 README：原 class 的 `java -Xverify:all` 输出、javap 关键事实、SHA-256；实测后记录，不凭猜测。
- [ ] 1.3 逐 fixture 分类：可运行且有基线（行为腿）／不可编译（钉"当前不可编译"事实 + 呈现腿）／仅呈现锚。分类结论写进证据 README。

## 2. CI 守卫测试

- [ ] 2.1 呈现腿（8 个 fixture 全部，留在默认套件）：关键文本锚断言——`anonymous-super-dispatch` 的捕获写入序早于 `super()` 且 `visibleDuringSuper` 相关成员完整呈现；`anonymous-super-args` 的 verbatim ctor 序与父类实参转发；其余按 1.3 分类各自钉最小可判伪锚。
- [ ] 2.2 行为腿（有可运行基线者，按 `p3_execution_comparison` 惯例标 `#[ignore]`）：原 class 输出钉为 golden；能重编者加重编运行对照，输出逐行一致；不可编译者断言 javac 退出非 0 且诊断与记录一致（**不得**当通过）。
- [ ] 2.3 负向自检：临时改回 ctor 重排（或用等价扰动）确认 2.1 的锚**会失败**——证明守卫真的能捕获本次事故那类回归；自检结果记录于证据，不留在代码里。

## 3. 纪律与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（当前主线 294 目标 / 2918 passed + 本片新增；已知 flake 家族见 handoff.md，单测复跑两轮判定）；`#[ignore]` 腿单独复跑通过；fmt；clippy **从 `.github/workflows/ci.yml` 逐字生成命令**（含 `--all-features`，29 项 `-A`）；`openspec validate --all --strict`；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 本片**零生产代码改动**（`git diff --stat` 证明 `crates/`、`src/` 无变更）；corpus fingerprint 若因新增 README 变化，按文档规定的 ignored 再生成器重生并确认 diff 为纯新增。
- [ ] 3.3 root 独立复核锚的可判伪性（含 2.3 自检证据）、分类准确性与零生产改动，把登记纪律写入 handoff.md：新增冻结行为 fixture MUST 同时加引用它的 CI 测试，`run.sh` 定位为复现工具而非守卫。
