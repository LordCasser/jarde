# 验证记录

后续相邻回归校准：`tests/p3_required_conversions.rs` 的 `castPart(C)` 本身没有字段写入，也没有新增转换 IR。此 change 的 `prepare_deferred_bindings` 对每个 SSA value 收取一项 `ir_items`，使该旧测试的运行基线从 140 变成 224；root 将断言更新为 224，仍固定 `normalization_clones=0` 和 224 字节转换正文，8 项转换测试通过。这是预算基线维护，不是窄字段转换的增量成本。

本轮使用 `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1` 串行执行。首次行为验收的 CLI 为
`target/debug/jarde-cli`，SHA-256 为
`7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34`。

## 生产与 Rust 回归

- `cargo build -p jarde-cli` 通过。
- `cargo test -p jarde-java` 通过：104 个单元测试、32 个 `p3_java_recovery`、46 个
  `p3_patterns`。
- `cargo test -p jarde --test p3_deferred_value_order -- --nocapture` 通过：2 个正常测试，
  1 个 JDK 测试按标记忽略。
- `cargo test -p jarde --test p3_deferred_value_order recovered_deferred_value_order_matches_patched_class_runtime -- --ignored --nocapture`
  通过：完整类重编译、`-Xverify:all` 和运行结果对照均通过。
- `names::tests::free_name_with_visits_every_reserved_local_and_candidate` 通过，覆盖
  `free_name` 内部 reserved/local 收集和 suffix 候选逐项计费、取消入口。
- 新 binding refusal 的 quote 来源 BCI 在发布前逐项 poll/charge；旧
  `quoted_bcis → deferred_producers` 只读递归由 `MAX_VALUE_DEPTH=24` 和 `first_visit` 去重
  封顶，随后仍经过既有 fallback/output 预算边界。已有取消和原子输出回归保持通过。

## 当前直线审计

四组既有输入均使用上述 CLI 重放，结果为 0 quote、完整 Java 8 编译、0 运行差异和 JADX
重编译一致：

- `current-replay/values/summary.json`：18 项；
- `current-replay/checks/summary.json`：24 项；
- `current-replay/allocations/summary.json`：12 项；
- `current-replay/structured/summary.json`：12 项。

`composite-boundary/fixed/summary.json` 的 15 项也为 0 quote、编译通过、0 运行差异，且
记录了当前 CLI SHA。构造、调用、字段和除法组合均在 producer 与独立 `mark()` 之间保存
值，未回到旧的表达式内联。

## 新边界回归

- `nested-producer-boundary/fixed/summary.json`：5 项；原/JADX/jarde 完整运行一致，0
  quote。内层值跨独立效应保留自己的声明，外层 consumer 使用已提交声明。
- `negative-review/switch-core/fixed/summary.json`：6 行；JVM 校验、原/JADX 编译通过，
  jarde 对已拒绝的 join consumer 保留完整来源 quote（`jarde_javac_returncode=1` 是该
  完整 quote 的预期结果），输出中不再有可执行的 `SwitchSupport.value()` 或
  `SwitchSupport.other()`。
- `guard-inline-core/fixed/summary.json`：14 项；0 quote、完整编译、`-Xverify:all`
  运行一致。
- `negative-review/failed-declaration/fixed/summary.json`：6 行；失败声明后的 consumer
  继续完整引用，未生成未声明的 saved/local 名称，JADX 输入一致。

上述输入均由各目录的 `run_audit.py` 生成精确 Code patch 和日志，root 的原始失败基线
仍保留在对应 `root-ecab` 目录。本项没有修改 census/fingerprint，也没有把未获证明的
跨循环、guard 或跨 region 情形改写成成功恢复。

## root 对预算修订版的独立复核

预算修订版 CLI 冻结为 `/tmp/jarde-cli-deferred-budget-908c`，SHA-256
`908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570`。
root 使用该确切二进制重新执行上述审计，原始日志保存在
`openspec/evidence/java-syntax-2026-09-22/deferred-evaluation/**/root-908c/`：四组直线
18/24/12/12 项、构造、复合 15 项、嵌套 producer 5 项、guard 内联 14 项与普通内联
60 项均为零引用、完整重编译成功，原始/JADX/jarde 运行逐项一致。两个拒绝样本的原始
class 与 JADX 均可编译执行；jarde 保留完整来源引用，生成源码按预期不能编译，也没有
留下可执行的错误调用或虚构的局部名。

新增共享 `dup` SSA 夹具经 `java -Xverify:all` 验证；quote 遍历以共享 `ValueId` 集合
去重并逐节点检查取消/期限，结束后按实际访问的不同节点数计入 `IrItems`。极低预算
测试仅验证整个恢复受限额约束，不能单独证明限额恰在 quote 遍历中触发；这部分的
计费结论同时依赖对 `binding_quote_bcis` 的代码审读。

预算修订后的 Java 包测试为 104 个单元、32 个 recovery、47 个 patterns 全通过；
deferred 集成正常 2 项及 JDK 执行 1 项通过。root 另跑 `p3_eval_context` 10 项、
`p3_local_rewrite` 7 项、`p3_reference_cast` 5 项及其 JDK 执行 1 项、JSON CLI 的
`nested-eval` 跨入口一致性，均通过。`p3_execution_comparison` 在更新普通 cast、throw、
延期调用三类过期预期后，用真实 Java 8 包装类重编译并执行全部样本，逐项运行对照通过。
本轮还运行了 `p3_lambda_adaptation`：2 项结构/证据测试通过，动态 String 检查测试
仍按独立待实施的 `preserve-lambda-descriptor-adaptation` 呈 RED，不属于本项实现或验收。
全仓 `cargo fmt --all -- --check`、`git diff --check` 和 49 项 OpenSpec strict 均通过；
reader census 与 corpus fingerprint 已由 root 钉在 100/668/81/236/8、259 文件并通过。
