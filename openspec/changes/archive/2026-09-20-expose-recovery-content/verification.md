# 验证记录

固定提交 `7d095ce`（实现），归档提交见仓库历史。门禁在本机跑在实现提交上，随后由该提交的 CI 复核。

## 契约实现

- `RecoveryReport.content: RecoveryContent`，闭合三值 `not_produced` / `explanation_only` / `contains_statements`（`crates/jarde-java/src/report.rs`，serde `snake_case`，随 `jarde-java` 与根门面 re-export；CLI 直接序列化库类型，`crates/jarde-cli/src/main.rs` 未改）。
- 事实来源在发射层：`emit.rs::Emitter::stmt` 只把非 `StmtKind::Fallback` 的语句计入 `Emitted::statements`，该计数仅在 `Emitter::finish` 里随文本一起发布，所以发射中途停止不会留下半个计数。`report.rs::content_of` 只读这个计数；`report.rs::stopped` 直接写 `NotProduced`，不询问构建器是否已经产生 AST。
- 不使用 `program.statements`（它把 fallback 计为语句），不剥离注释、不数 token、不二次解析文本；`produced()` 语义未变。

## 逐例证据（公开入口 `Engine::recover_method`）

| 输入 | outcome | representation/quality | content |
| --- | --- | --- | --- |
| `RefusedCast.fieldCast()` / `instanceCast` / `chainCast` | Produced | Mixed/Fallback | `explanation_only`（产物只有包装、理由与 `// @bytecode` 引用，逐行断言无语句行） |
| `NestedEval.nestedPlain(I)I`、`RefusedCast.leftRead()` | Produced | Java/Structured | `contains_statements` |
| `NestedEval.nestedLocal(I)I` | Produced | Mixed/Fallback | `contains_statements`（保留 `arg0 = arg0 + 1;` 与 quote 并存） |
| 手建 `nothing()V` = `return;` | Produced | Java/Structured | `contains_statements` |
| 手建 `emptyArm(ZI)I` = `if (arg0) { }` | Produced | Java/Structured | `contains_statements`（quality 未因此提高） |
| 手建 `literalA`/`literalB`（`"a//b"`、`"a/*b*/"`） | Produced | Java/Structured | `contains_statements`（分类不随字面量或注释措辞改变） |
| 手建 `castA`/`castB`（同一形状、不同 refusal 措辞与 BCI） | Produced | Mixed/Fallback | `explanation_only` |
| `nestedLocal` + 输出预算停止 | Stopped(`Budget{OutputBytes}`) | Bytecode/Fallback | `not_produced`，`text == ""`、段表为空 |
| `nestedLocal` + 预取消 token | 分析面 Cancelled、呈现面 Stopped(`IrTableMissing{canonical}`) | Bytecode/Fallback | `not_produced`，`text == ""` |
| `nestedLocal` + `stages=[Frame]`（成功响应内的停止） | Stopped(`IrTableMissing{ssa}`) | Bytecode/Fallback | `not_produced`，`text == ""` |

同一个 `nestedLocal` 在足额预算下是 `contains_statements` 且文本非空——于是「AST 里已有语句、报告仍为 `not_produced`」是被直接对照过的，而不是默认。

## 判别性

- 测试先写、后实现：实现前该文件无法编译（`E0425 cannot find type RecoveryContent`、`E0609 no field content`），失败输出留在 `/tmp/pre-change-failure.txt`。
- 变异 A（设计明令禁止的来源）：用 `program.statements == 0` 判定 → explanation-only 用例与手建形态用例**变红**（`fieldCast` 左 `ContainsStatements`、右 `ExplanationOnly`），因为该计数含 fallback。
- 变异 B（调用方式猜测）：用 `emitted.text.is_empty()` 判定 → 同两例变红。
- 两条变异均已恢复，最终树只含预期改动。

## 库/CLI 一致

`crates/jarde-cli/tests/json_cli.rs` 对同一请求走两个入口做**整份文档**逐字段比较（只剔除 `elapsed_millis`），并单独断言 `content` 的三种取值：`contains_statements`（`nestedPlain`/`nestedLocal`/`leftRead`）、`explanation_only`（`fieldCast`）、`not_produced`（无 SSA 停止）。JSON 拼写另由 `tests/p3_product_vocabulary.rs` 按 `snake_case` 与闭合集合（读取 `report.rs` 的枚举定义，顺序与成员都断言）验证。

## 评测汇总（任务 2.1）

汇总落在 ignored 对照用例 `tests/p3_execution_comparison.rs`：每行带 `content`，每个样本表后给出可对账的计数块（declared / with `Code` / without `Code` / requests / produced / `contains_statements` / `explanation_only` / stopped / not requested），并由 `run_sample` **断言**恒等式：`declared = with + without Code`、`declared = requested + not-requestable`、`requested = produced + stopped`、`produced = statements + explanation-only`、`not_produced == stopped`。无 `Code` 的成员改为以适用性理由跳过，不再作为请求计入语句覆盖。本次记录数量：12 个样本、83 个声明成员、83 个带 `Code`、70 次请求 = 70 produced = 58 `contains_statements` + 12 `explanation_only`、0 stopped、13 个 initializer 未请求。

历史 token 启发式（`compare2.py` 去注释判 token：25,853 / 99.4% / ≈83.4% / 16.0%）作为**它自己的定义**记在 `tests/fixtures/p3-corpus/README.md` 的新节里，并标明新版本分类与它不是同一口径；旧数字没有被改写。文本/调用/字符串相似度的有效 pair 与排除原因在仓库内没有对应仪器（那是 benchmark 的脚本），因此本次以文档区分口径，不新建相似度 harness。

## 门禁（`7d095ce`，本机）

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test p3_content --locked` | 4 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1127 passed / 0 failed / 5 ignored** |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（20.4 s） |
| `cargo test -p jarde-cli --test json_cli --locked` | 16 passed / 0 failed |
| `openspec validate --all --strict --no-interactive` | 22 passed / 0 failed（归档前，含本 change） |

## 边界

- `recovery-validation` 中「文本/调用/字符串相似度各自有效 pair 与排除原因」的 scenario 没有仓库内载体；本次记录口径而不是发明仪器。
- 受控 `return;` 与空分支形态放在内存生成器 `tests/p3_content.rs`（仓库内 `.class` 总体被 `jarde-reader` 的 fixture 普查按精确数量断言，本 change 不改该 crate）。
- `content` 只描述产物内容；representation/quality/syntax/coverage/execution/verification 一律未动，R8/R9 的回归继续独立成立。
- 顺带观察到、本次未改的既有缺陷：`tests/fixtures/p3-corpus/README.md` 有一处标题缺少换行（`## What this corpus does not cover* **A second javac generation**`）。
