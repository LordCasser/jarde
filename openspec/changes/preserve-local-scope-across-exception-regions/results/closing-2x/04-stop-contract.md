# 2.4：作用域规划 / 校验 / 拒绝闭包的定向停止测试（closing-2x）

日期 2026-10-08。测试：`tests/preserve_local_scope_refusals.rs` 的
`the_scope_planning_and_validation_stops_commit_no_partial_body`。

## 1. 停止点为什么是“定向”的

声明规划的每一次 `AnalysisSteps` 计入都以**该局部访问的 BCI** 为锚
（`crates/jarde-java/src/build.rs` 的 `declarations` → `validate_declaration_placements` 均以
`use_.bci` 记账）。因此把 run 的 `analysis_steps` 上界压到“某成员完成前的最后一次记账”时，
`StopReason::Budget { dimension: AnalysisSteps, at }` 的 `at` 就是该成员**第一个局部访问**的 BCI：
停止确实落在作用域规划/校验的计入点上，而不是别处的巧合。

测试对正例类 `ScopePlan` 的五个成员逐一定位该边界（成员 → 锚 BCI）：

| 成员 | 锚（第一个局部访问） | 语义 |
| --- | --- | --- |
| `catchOnly` | 9 | catch 绑定 store（`astore_1`） |
| `assignedAcrossTry` | 7 | 受保护区写入（`istore_1`） |
| `assignedAcrossIf` | 5 | then 臂写入（`istore_1`） |
| `nestedHandlerOnly` | 19 | 内层 handler 体 |
| `nestedAcross` | 8 | 最内层写入（`istore_1`） |

本提交实测的绝对边界（仅记录；测试自己扫出边界，不钉绝对步数）：`catchOnly` 262/263、
`assignedAcrossTry` 524/525、`assignedAcrossIf` 753/754、`nestedHandlerOnly` 1016/1017、
`nestedAcross` 1389/1390（“停止/完成”的界），整类 3065/3066。

## 2. 停止契约（逐点断言）

* `RecoveryOutcome::Stopped(StopReason::Budget { dimension: AnalysisSteps, at: Some(anchor) })`；
* `RecoveryContent::NotProduced`、`text == ""`、`source_map.len() == 0`、`regions`/`fallbacks` 空；
* 边界值 +1 时该成员整段呈现（与 ample 预算的文本相同）——停止只在预算不足时发生；
* 整类扫描（两腿，1..=完成步数）：每个成员的文本要么等于参考文本，要么为空且 `NotProduced`
  且无 segment table；出现任何停止时类级 `execution` 非 `Complete`。**没有任何一次出现“半个成员”**。

## 3. 拒绝闭包的 output 计入

拒绝文本（`// @bytecode` 引注行 + 拒绝句）由 emitter 写出并按 `OutputBytes` 记账，故
`output_bytes` 上界扫描（`ScopeRefusals`，1..=完成字节数）同样只出现两种结果：

* 成员文本为空 + `NotProduced` + 无 source map（其中至少一次是
  `StopReason::Budget { dimension: OutputBytes, .. }`——拒绝闭包自身的写入点）；
* 成员文本与参考文本逐字节相同（引注整段提交或完全不提交）。

上界低于类信封自身写入时，请求级回答 `OperationOutcome::Incomplete`（
`TerminationReason::BudgetExceeded { dimension: OutputBytes }`），同样没有成员文本可被部分提交。

## 4. 取消

取消令牌在 `Engine::class_source` 上对 `ScopePlan` 与 `ScopeRefusals` 两输入都回答
`Incomplete`（`ExecutionReport::Cancelled`），或在 `Performed` 时每个成员文本为空——
与预算停止同点、同契约。取消与预算是同一组 `poll`/`charge` 调用，故第 1–3 节的边界扫描即覆盖
同一组取消点。

## 5. 运行

```
$ cargo test --test preserve_local_scope_refusals --all-features --locked
test the_scope_planning_and_validation_stops_commit_no_partial_body ... ok
test result: ok. 4 passed; 0 failed; 1 ignored
```
