## Context

事故驱动：`recover-synthetic-ctor-super-order` 的静默回归（`visibleDuringSuper` true→false）在 CI 全绿下通过，只因 `anonymous-super-dispatch` 无测试引用。证据见 [ctor-reorder-dispatch-regression](../../evidence/java-syntax-2026-10-04/ctor-reorder-dispatch-regression/README.md)（含可跑的 `results/repro.sh`）。

**第一个取证义务**：读 `tests/p3_anonymous_class_facts.rs` 与 `tests/class_source.rs` 的 fixture 消费模式（`include_bytes!` + `Engine::open` + `ClassSourceRequest` + 断言），确认哪一种最省成本地表达"原 class 运行输出"腿——是否已有 helper 在测试内编译并运行 Java（`tests/recover_synthetic_ctor_super_order.rs` 的 `recompile_and_run`、`tests/assert_statement_sugar.rs` 的 `run_class`/`recompile_and_run` 都是先例），能否直接复用而非新写。

## Goals / Non-Goals

**Goals:** 6 个未守卫 fixture 的行为与关键呈现锚进入 CI；无基线者先补记录再钉断言；登记纪律入 handoff。**Non-Goals:** 不改任何生产代码；不新增恢复能力；不扩展到 `proved-java-structure` 以外的 fixture 目录（那是 `audit-corpus-gates` 的登记域）；不把当前不可编译的形"修好"（那属各专项 change）。

## Decisions

1. **两类腿，按 fixture 能力选择**：可运行且有基线 → 行为腿（原 class 输出 golden；能重编者加重编对照）；不可编译 → 钉"当前不可编译"事实 + 呈现腿（关键文本锚）。**不得**把不可编译当通过，也不得为凑绿而放宽锚。
2. **复用既有测试 helper**，不新建执行框架（`recompile_and_run`/`run_class` 已是先例）。
3. **与 `recover-ctor-reorder-dispatch-guard` 的分工**：guard 片断言 dispatch 形不重排（判据测试，已合入 `fc868aba`）；本片把行为腿与呈现锚推广到 6 个 fixture（覆盖测试）。实施顺序在 guard 之后（已满足），避免同一 fixture 的断言被两片重复或冲突。
4. **登记纪律**：新增冻结行为 fixture MUST 同时加引用它的 CI 测试；`run.sh` 定位为复现工具而非守卫。

## Risks / Trade-offs

- **测试运行时间**（6 个 fixture × 编译/运行 Java）→ 按既有 `p3_execution_comparison` 的 `#[ignore]` 惯例把重的行为腿标 ignored（手动/CI 定期跑），呈现腿（纯文本断言、无 JVM 调用）留在默认套件——保证改序类回归至少在呈现腿可见。
- **钉错基线**（无 README 的 3 个）→ 先 `java -Xverify:all` 实测并记录 SHA，再写断言；不凭猜测。
