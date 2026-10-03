## Why

**系统性验证债务**，由 root 2026-10-04 验收 `recover-synthetic-ctor-super-order` 引入的静默行为回归时发现：`tests/fixtures/proved-java-structure/` 的 16 个冻结 fixture 中 **8 个未被任何 CI 测试引用**，其记录的行为基线只存在于 README 与**手动** `run.sh`（实测：CI workflow 与全部测试文件均不调用 `run.sh`）。后果已由真实事故证明——ctor 重排使 `anonymous-super-dispatch` 的 `visibleDuringSuper` 从 `true` 翻转为 `false`，而 CI 全绿（2918 passed），回归只能靠 root 手动重放才发现。

未被引用的 8 个（`ls` + 逐名 grep 实测）：

| fixture | run.sh | README | 记录的行为基线 |
| --- | --- | --- | --- |
| `anonymous-super-dispatch` | 有（手动） | 有 | `observed=captured-value` / `visibleDuringSuper=true` |
| `anonymous-super-args` | 无 | 有 | 事件日志（构造顺序），当前完整源集编译退出 1 |
| `anonymous-member-base` | 有（手动） | 有 | — |
| `anonymous-top-level` | 有（手动） | 有 | — |
| `lambda-body-inline` | 有（手动） | 有 | — |
| `enum-arity` | 无 | 无 | — |
| `package-info-basic` | 无 | 有 | — |
| `short-circuit-left-false` | 无 | 有 | — |

**与既有归属的关系（查重）**：`present-proved-java-structure` 的 7.2 有执行对照机制（`cargo test --test p3_execution_comparison --locked -- --ignored`），但其范围明写"只覆盖**本 change** 使文本可单独编译的 fixture"——上表中 `anonymous-super-args` 等当前**不可编译**，落在 7.2 之外。`audit-corpus-gates`（6 未勾）处理的是 fixture **登记/census 索引**完整性（`p5_corpus_fingerprint` 的文件哈希与 reader 人口计数），**不守行为**——该测试自述"asserts nothing about whether an acceptance row passes"。故本片不重复二者：它把已冻结的**行为事实**变成 CI 断言。

## What Changes

- 为上表 8 个 fixture 补 CI 守卫测试，按既有模式（`include_bytes!` + `Engine::open` + `ClassSourceRequest` + 文本/行为断言，参照 `tests/p3_anonymous_class_facts.rs`）：
  - **行为腿**（有可运行基线者）：原 class 的运行输出钉为 golden；能重编者加重编运行对照，不能重编者钉"当前不可编译"这一事实（**不得**把不可编译当成通过）。
  - **呈现腿**（全部 8 个）：关键文本锚（如 `anonymous-super-dispatch` 的捕获写入序早于 `super()`、`visibleDuringSuper` 相关成员完整呈现）钉为断言，使任何改变该形的呈现改动都在 CI 可见。
- 对无 README 基线的 fixture（`enum-arity`、`package-info-basic`、`short-circuit-left-false`）先补记录（`java -Xverify:all` 输出与 SHA），再写断言——不得凭猜测钉基线。
- 建立**登记纪律**（写入 handoff.md）：新增冻结行为 fixture 时 MUST 同时加一个引用它的 CI 测试；`run.sh` 是复现工具而非守卫。

## Capabilities

### New Capabilities

无（本片只补验证覆盖，不改恢复能力）。

### Modified Capabilities

- `java8-recovery`：其验收 fixture 的行为基线由 CI 断言守卫，而非仅手动脚本。

## Impact

`tests/`（新增或扩展守卫测试文件）、`tests/fixtures/proved-java-structure/{enum-arity,package-info-basic,short-circuit-left-false}/`（补 README 基线记录）、`handoff.md`（登记纪律）；`crates/` 与 `src/` **无生产代码改动**。与 `recover-ctor-reorder-dispatch-guard` 有重叠面（该片的 (a) 项也断言 `anonymous-super-dispatch` 不重排）——**实施顺序应在 guard 之后**，本片把该断言推广到其余 7 个 fixture 并补行为腿，不重复其判据测试。
