## 1. 取证与基线补全

- [x] 1.1 读 `tests/p3_anonymous_class_facts.rs`、`tests/class_source.rs`、`tests/recover_synthetic_ctor_super_order.rs`（`recompile_and_run`）与 `tests/assert_statement_sugar.rs`（`run_class`/`recompile_and_run`）的 fixture 消费与 Java 执行模式，选定复用哪个 helper（不新建执行框架）；记录 **6 个在范围内** fixture（`anonymous-member-base`、`anonymous-top-level`、`lambda-body-inline`、`enum-arity`、`package-info-basic`、`short-circuit-left-false`）各自的 class 清单与 SHA。**注意布局**：`enum-arity` 在嵌套 `v8/probe/`、`package-info-basic` 在嵌套 `v8/p/`，其余四个平铺；`include_bytes!` 路径逐个核对（proposal 的布局陷阱段）。**不在本片范围**：`anonymous-super-args`（已被环 0/环 1 的 2 个测试文件守卫）、`anonymous-super-dispatch`（已被 ctor-reorder-guard 守卫，且是环 3 的锚）——不重复守卫。
  证据：`results/README.md` §0（开工复核当前 CI 引用）与 §1.1（18 个 class 的实测 SHA-256 + 布局核对）。复用形态：`include_bytes!` + `Engine::open` + `ClassSourceRequest`（`p3_anonymous_class_facts` 模式）与 `run_class`/`recompile_and_run` 的 JDK 执行形态（`assert_statement_sugar`/`recover_synthetic_ctor_super_order`）都搬进 `tests/fixture_behavior_guards.rs` 的本地 helper，**未新建执行框架**；条目名用包路径（`probe/…`、`p/…`），磁盘路径保留 `v8/`。
- [x] 1.2 **6 个 fixture 全部须实测补记行为 golden**（root 复核 proposal：没有一个 README 记录了实际 `java -Xverify:all` 运行输出——`lambda-body-inline`/`anonymous-top-level`/`anonymous-member-base` 只说明 `run.sh` 会跑 `-Xverify:all`，非钉基线；`enum-arity` 无 README 只有 `SHA256SUMS`；`package-info-basic`/`short-circuit-left-false` 有 README 但无运行输出）：补原 class 的 `java -Xverify:all` 输出、javap 关键事实、SHA-256；实测后记录，不凭猜测。
  证据：`results/README.md` §1.2 六个 golden + 逐 fixture javap 关键事实；实测输出与各 fixture 既有证据目录的 `original-run*` **逐字节相同**（对比脚本自检：空产出即中止）。golden 写进 `tests/fixture_behavior_guards.rs` 的 `*_GOLDEN` 常量，并补记进六个 fixture 目录的 README（`enum-arity` 新建 README）。
- [x] 1.3 逐 fixture 分类：可运行且有基线（行为腿）／不可编译（钉"当前不可编译"事实 + 呈现腿）／仅呈现锚。分类结论写进证据 README。**对 `anonymous-top-level`（环 2 锚）与 `anonymous-member-base`（`this$0` 三者并存锚）**：当前物理文本呈现，呈现腿 golden 须标注"当前物理呈现，后续切片解锁时主动更新"，避免后续切片被误判为回归（proposal 的协调段）。
  证据：`results/README.md` §1.3。**一处措辞更正**：`anonymous-top-level` 的"当前物理文本呈现"已过期——环 2（`recover-anonymous-supertype-return`，合并 `23b69bcf`）在本片开工前已落地，其当前呈现是**投影形**（`return new Base(choose()) { … }`），本片按投影形钉锚并注明环 2 已解锁；`anonymous-member-base`（5.3 锚）与 `lambda-body-inline`（5.2 锚）当前仍是拒绝/转发形，锚处标注解锁切片须在同一次提交更新。

## 2. CI 守卫测试

- [x] 2.1 呈现腿（**6 个在范围内 fixture**，留在默认套件）：关键文本锚断言，按 1.3 分类各自钉最小可判伪锚。**不钉** `anonymous-super-dispatch`/`anonymous-super-args` 的呈现（前者是环 3 锚、呈现即将翻转；后者已被环 0/1 守卫）。对 `anonymous-top-level`/`anonymous-member-base` 钉"当前物理呈现"并标注后续切片解锁时须主动更新（1.3）。
  证据：`tests/fixture_behavior_guards.rs` 默认套件 7 项：限定接收者拒绝 + 成员基类构造器语句序（`anonymous-member-base` 两项）、子类捕获写在 `super` 前 + 根投影（`anonymous-top-level`）、转发箭头/helper 语句序/`main` 拒绝（`lambda-body-inline`）、0/1/4 常量头 + runner 读取（`enum-arity`）、`@java.lang.Deprecated` 先于 `package p;`（整文本相等，`package-info-basic`）、单次短路写入（`short-circuit-left-false`）。`cargo test --test fixture_behavior_guards --locked` = 7 passed / 0 failed / 6 ignored。
- [x] 2.2 行为腿（有可运行基线者，按 `p3_execution_comparison` 惯例标 `#[ignore]`）：原 class 输出钉为 golden；能重编者加重编运行对照，输出逐行一致；不可编译者断言 javac 退出非 0 且诊断与记录一致（**不得**当通过）。
  证据：同文件 `#[ignore]` 6 项，`cargo test --test fixture_behavior_guards --locked -- --ignored` = 6 passed / 0 failed。四个可重编 fixture（`anonymous-top-level`/`enum-arity`/`package-info-basic`/`short-circuit-left-false`）重编后运行输出与 golden 逐行一致；两个不可编译 fixture（`anonymous-member-base`/`lambda-body-inline`）断言 javac 非 0 且诊断含记录的 `missing return statement`/`cannot find symbol`（`-J-Duser.language=en` 固定诊断语言），测试名与文档明写这是**当前事实的钉**而非行为已验证。
- [x] 2.3 负向自检：对本片**实际守卫的 6 个 fixture 之一**做等价扰动（如临时改坏某呈现锚对应的生产逻辑，或临时改 ctor 重排——但注意 ctor 重排影响的 `anonymous-super-dispatch` 已不在本片呈现腿范围，故须选一个本片真正钉了锚的 fixture），确认其锚断言**会失败**——证明守卫真能捕获回归而非恒真断言。自检结果记录于证据，不留在代码里。
  证据：`results/README.md` §2.3 + `results/03-self-check-perturbation.out`/`03-self-check-perturbation-ignored.out`。扰动：`crates/jarde-java/src/ctor_order.rs` 的越序门临时放宽为"任何构造器调用"（事故的同一个洞）→ `anonymous_top_level_pins_the_childs_capture_write_before_the_super_call` 失败（`left: ["super(seed);", …]` vs `right: ["this.val$captured = arg3;", …]`），默认套件 6 passed / 1 failed，同扰动下 `#[ignore]` 腿 6/6 仍通过（改序在该运行不可观测，故呈现腿是默认套件里的可见守卫）；`git checkout -- crates/jarde-java/src/ctor_order.rs` 完整回退，回退后 7/7 + 6/6 全绿且 `git diff --stat HEAD -- crates src` 为空。

## 3. 纪律与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（**基线以开工时主线实测为准**——root 2026-10-04 环 1 合入后为 **296 目标 / 2943 passed / 0 failed**；本片会再新增若干守卫测试。已知 flake 家族见 handoff.md，含 `bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too` 与 `bulk_recovery_lifecycle`，单测复跑两轮判定）；`#[ignore]` 腿单独复跑通过；fmt；clippy **从 `.github/workflows/ci.yml` 46–76 行逐字生成命令**（含 `--all-features`，29 项 `-A`，`-D warnings`）；`openspec validate --all --strict`（root 2026-10-04 为 **272 项**）；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
  证据：`results/06-gates.md`（权威判定与原始日志都在 `results/`）。权威轮（命令为父任务书指定形：
  `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`）**exit 0 / `test result: ok` 332 条 /
  `test result: FAILED` 0 条 / 319 个测试二进制**（`06-workspace-tests.out`）；新守卫二进制在其中
  `7 passed; 0 failed; 6 ignored`。前 4 轮各中 1–2 个 handoff 已登记家族的 flake（`p3_two_exit_return` scratch
  `AlreadyExists`、`d3_artifact_binding` 与 `engine::standalone` 计时、`p4_plugins` 计时），每例单测复跑两轮全绿
  （`06-flake-adjudication-*.out`），干净轮在同一棵树上 332/0。`#[ignore]` 腿：`cargo test --test fixture_behavior_guards --locked -- --ignored`
  = **6 passed / 0 failed**。fmt `--check` exit 0；clippy 逐字形 exit 0 且 29 项 `-A`，另跑补回 ci.yml 第 76 行
  `-D warnings` 的 CI 等价形 exit 0（该 grep 模式 `^cargo|^-A` 不含 `-D`，逐字形会漏掉它）；`openspec validate --all --strict`
  = **307 passed / 0 failed**；`cargo test --test p5_corpus_fingerprint --locked` 5 passed（manifest 未改动）。
  磁盘：每轮构建前 `df -h /` 实测 44→22Gi，全程未低于 15Gi（未触发 `cargo clean` 阈值）；报告前已 `cargo clean`（见最终报告）。
- [x] 3.2 本片**零生产代码改动**（`git diff --stat` 证明 `crates/`、`src/` 无变更）；corpus fingerprint 若因新增 README 变化，按文档规定的 ignored 再生成器重生并确认 diff 为纯新增。
  证据：`git diff --stat HEAD -- crates src` 为空；`tests/fixtures/corpus-fingerprint.json` 未改动（`git status` 无该文件）——
  本片新增的 fixture 侧文件全是 `.md`（README），指纹把 `md` 作为"关于语料的记录"排除在输入外，故零 corpus 位移，
  也没有触发任何 census/fingerprint 再生成（`results/06-gates.md` 的 fingerprint 段）。
- [x] 3.3 root 独立复核锚的可判伪性（含 2.3 自检证据）、分类准确性与零生产改动，把登记纪律写入 handoff.md：新增冻结行为 fixture MUST 同时加引用它的 CI 测试，`run.sh` 定位为复现工具而非守卫。（root 2026-10-07 完成，见 [verification-root.md](verification-root.md)：零生产改动 root 复核 ✓、负向自检=事故同款洞被新测试捕获 ✓、golden 双源交叉核对 ✓、前提更正（投影形）追认 ✓、7+6 套件 + 332 targets ok/0 FAILED 全绿 ✓；handoff 登记句已刷新为"补覆盖已完成"）
