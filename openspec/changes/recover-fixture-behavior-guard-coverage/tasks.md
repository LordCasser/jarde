## 1. 取证与基线补全

- [ ] 1.1 读 `tests/p3_anonymous_class_facts.rs`、`tests/class_source.rs`、`tests/recover_synthetic_ctor_super_order.rs`（`recompile_and_run`）与 `tests/assert_statement_sugar.rs`（`run_class`/`recompile_and_run`）的 fixture 消费与 Java 执行模式，选定复用哪个 helper（不新建执行框架）；记录 **6 个在范围内** fixture（`anonymous-member-base`、`anonymous-top-level`、`lambda-body-inline`、`enum-arity`、`package-info-basic`、`short-circuit-left-false`）各自的 class 清单与 SHA。**注意布局**：`enum-arity` 在嵌套 `v8/probe/`、`package-info-basic` 在嵌套 `v8/p/`，其余四个平铺；`include_bytes!` 路径逐个核对（proposal 的布局陷阱段）。**不在本片范围**：`anonymous-super-args`（已被环 0/环 1 的 2 个测试文件守卫）、`anonymous-super-dispatch`（已被 ctor-reorder-guard 守卫，且是环 3 的锚）——不重复守卫。
- [ ] 1.2 **6 个 fixture 全部须实测补记行为 golden**（root 复核 proposal：没有一个 README 记录了实际 `java -Xverify:all` 运行输出——`lambda-body-inline`/`anonymous-top-level`/`anonymous-member-base` 只说明 `run.sh` 会跑 `-Xverify:all`，非钉基线；`enum-arity` 无 README 只有 `SHA256SUMS`；`package-info-basic`/`short-circuit-left-false` 有 README 但无运行输出）：补原 class 的 `java -Xverify:all` 输出、javap 关键事实、SHA-256；实测后记录，不凭猜测。
- [ ] 1.3 逐 fixture 分类：可运行且有基线（行为腿）／不可编译（钉"当前不可编译"事实 + 呈现腿）／仅呈现锚。分类结论写进证据 README。**对 `anonymous-top-level`（环 2 锚）与 `anonymous-member-base`（`this$0` 三者并存锚）**：当前物理文本呈现，呈现腿 golden 须标注"当前物理呈现，后续切片解锁时主动更新"，避免后续切片被误判为回归（proposal 的协调段）。

## 2. CI 守卫测试

- [ ] 2.1 呈现腿（**6 个在范围内 fixture**，留在默认套件）：关键文本锚断言，按 1.3 分类各自钉最小可判伪锚。**不钉** `anonymous-super-dispatch`/`anonymous-super-args` 的呈现（前者是环 3 锚、呈现即将翻转；后者已被环 0/1 守卫）。对 `anonymous-top-level`/`anonymous-member-base` 钉"当前物理呈现"并标注后续切片解锁时须主动更新（1.3）。
- [ ] 2.2 行为腿（有可运行基线者，按 `p3_execution_comparison` 惯例标 `#[ignore]`）：原 class 输出钉为 golden；能重编者加重编运行对照，输出逐行一致；不可编译者断言 javac 退出非 0 且诊断与记录一致（**不得**当通过）。
- [ ] 2.3 负向自检：对本片**实际守卫的 6 个 fixture 之一**做等价扰动（如临时改坏某呈现锚对应的生产逻辑，或临时改 ctor 重排——但注意 ctor 重排影响的 `anonymous-super-dispatch` 已不在本片呈现腿范围，故须选一个本片真正钉了锚的 fixture），确认其锚断言**会失败**——证明守卫真能捕获回归而非恒真断言。自检结果记录于证据，不留在代码里。

## 3. 纪律与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（**基线以开工时主线实测为准**——root 2026-10-04 环 1 合入后为 **296 目标 / 2943 passed / 0 failed**；本片会再新增若干守卫测试。已知 flake 家族见 handoff.md，含 `bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too` 与 `bulk_recovery_lifecycle`，单测复跑两轮判定）；`#[ignore]` 腿单独复跑通过；fmt；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成命令**（含 `--all-features`，29 项 `-A`，`-D warnings`）；`openspec validate --all --strict`（root 2026-10-04 为 **272 项**）；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 本片**零生产代码改动**（`git diff --stat` 证明 `crates/`、`src/` 无变更）；corpus fingerprint 若因新增 README 变化，按文档规定的 ignored 再生成器重生并确认 diff 为纯新增。
- [ ] 3.3 root 独立复核锚的可判伪性（含 2.3 自检证据）、分类准确性与零生产改动，把登记纪律写入 handoff.md：新增冻结行为 fixture MUST 同时加引用它的 CI 测试，`run.sh` 定位为复现工具而非守卫。
