# 1.2/1.3 回填：复核与勾选（closing-2x）

日期 2026-10-08。二进制 = 本 worktree 的 `target/debug/jarde-cli`（clean 树 + 本切片新增文件）。

## 1. 结论

1.2 与 1.3 的**行为验收面**在 guard 片落地后仍成立，两个行为锚（12 explicit-lock / 13 io-wrapping）
在 current main 上已恢复（由 `recover-lock-guard-loop-finally` 与 `recover-io-resource-finally`
的 guard 证书翻转）。据此把 1.2/1.3 勾选，勾选依据是**测试 + 锚的现时实测**，不是规划片当时的口头预期。

## 2. 三个分类答案的现时复核（`tests/preserve_local_scope_plan.rs`）

```
$ cargo test --test preserve_local_scope_plan --all-features --locked
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

两条腿（`v8` = javac 23.0.1 `--release 8 -g:none`；`v8-javac8` = 真 javac 8）逐字节同答：

| 答案 | 现时钉面 | 现时结果 |
| --- | --- | --- |
| 词法 owner | `catchOnly` 的 `int local2 = 2;`/`return local2;` 只在 catch 子句内，子句后正文不含 `local2`；`nestedHandlerOnly` 的内层子句同理 | 通过 |
| 可提升 | `assignedAcrossTry` 在 `try` 之上声明、两路径赋值、汇合读；`assignedAcrossIf` 在 `if` 之上；`nestedAcross` 在外层 `try` 之上覆盖三处写入 | 通过 |
| 证据不完整 | `ScopePlanCrossing.flatFinally` 逐字拒绝（`@bytecode 0 15 28`）；`resourceAcrossFinally` 由 io 片**显式更新**为整段呈现（该测试文件的注释记录了这一处置） | 通过 |
| 类型/DA 一致 | 正例类每个成员 `ContainsStatements`，整类文本两腿逐字节相同且可重编执行 | 通过 |
| 覆盖校验 | 提升声明的 source-map origin 逐 region 钉住（7/13/15、5/11/13、8/15/22/24、catch 10 + derived 8） | 通过 |
| 停止契约 | `output_bytes=1` 与取消令牌在 `Engine::class_source*` 上回答 `Incomplete`，不提交半个 try/catch 与 source map | 通过 |

## 3. 锚 12/13 的现时实测（`01-reverify-1.2-1.3.sh`，自测：每个渲染先断言自头部）

```
input|sha256
anchor12-patrol-jar|79b61c4db829aa7e78ff6645ef5f302ab63b719e5462a4ffcbd6a02f16873b1f
anchor12-v8|79b61c4db829aa7e78ff6645ef5f302ab63b719e5462a4ffcbd6a02f16873b1f
anchor12-v8-javac8|79b61c4db829aa7e78ff6645ef5f302ab63b719e5462a4ffcbd6a02f16873b1f
anchor13-patrol-jar|27d2b6721966d046e47b968e6f7eb57879a5251d6a69f2ba0b9c5d319b26e442
anchor13-v8|27d2b6721966d046e47b968e6f7eb57879a5251d6a69f2ba0b9c5d319b26e442
anchor13-v8-javac8|27d2b6721966d046e47b968e6f7eb57879a5251d6a69f2ba0b9c5d319b26e442
plan-positive-v8|c823e459bc27f75d3ea776a9f34dd4add5f8fa5b80eaf9c866c871ad1da758ad
plan-positive-v8-javac8|c823e459bc27f75d3ea776a9f34dd4add5f8fa5b80eaf9c866c871ad1da758ad
plan-crossing-v8|73c08f7d4fc0f897c2cc2d06ae9d50b4b4798d0c30cb73af3c99ae7ac3bb4365
plan-crossing-v8-javac8|73c08f7d4fc0f897c2cc2d06ae9d50b4b4798d0c30cb73af3c99ae7ac3bb4365
refusals-v8|10020302fa06804777cca63a01da9e5b222d0e382f00deedc139de9f47d01c32
refusals-v8-javac8|10020302fa06804777cca63a01da9e5b222d0e382f00deedc139de9f47d01c32
```

* 锚 12 的 `79b61c4db829…` 与 lock-guard 片 `01-gating.out` 的 current 摘要、io 片
  `02-gating.out` 的 `lk-patrol-jar` 一致；渲染中 `local 1 crosses` 计数为 0，三个方法各有一个
  `finally { this.lock.unlock(); }`（`renders/anchor12-patrol-jar.java`）。
* 锚 13 的 `27d2b6721966…` 与 io 片 `02-gating.out` 的 `io-patrol-jar` current 一致；`countLines`
  以 `try { … } finally { local1.close(); }` 呈现（`renders/anchor13-patrol-jar.java`）。
* 两条 fixture 腿与巡查 jar 同摘要：控制流结论与编译器降级无关。
* 规划片记录的 `local-scope-positive`（`c823e459bc27…`）与 `local-scope-crossing`（`31a5506f5b7a…`）
  对照：正例不变；crossing 面因 `resourceAcrossFinally` 翻转（io 片已显式更新该测试的对照）而移动，
  `flatFinally` 仍逐字拒绝。

## 4. 站立零回归控制

```
$ cargo test --test p3_exception_scope --all-features --locked       1 passed
$ cargo test --test p3_nested_try --all-features --locked            2 passed
$ cargo test --test p3_typed_catch --all-features --locked           6 passed
$ cargo test --test p3_stated_rows --all-features --locked           2 passed
$ cargo test --test p3_loop_try_handler_entry --all-features --locked 6 passed
$ cargo test --test p3_try_local --all-features --locked             2 passed
$ cargo test --test recover_lock_guard_loop_finally --all-features --locked 5 passed
$ cargo test --test recover_io_resource_finally --all-features --locked 4 passed / 2 ignored
$ cargo test --test recover_nested_lock_finally_bodies --all-features --locked 4 passed / 2 ignored
```

## 5. 勾选处置

* **1.2 勾选**：三分类（词法 owner / 可提升 / 证据不完整）由 `preserve_local_scope_plan` 的
  `the_three_answers_are_the_rendered_class` 在两腿上钉住，类型与 DA 事实一致（正例全成员
  `ContainsStatements` + 两腿同文本 + 可重编执行）。
* **1.3 勾选**：提升覆盖已知读写由 `the_lifted_declaration_carries_the_origin_of_every_region_it_covers`
  钉住；catch/resource 子作用域与 fallback/未归属使用阻断由 `a_crossing_local_keeps_the_dependent_slice_refused`
  与 `p3_nested_try`/`p3_typed_catch` 的负例钉住；证据不足先拒绝再建 AST（拒绝即 `ExplanationOnly` +
  `@bytecode` 覆盖，正文无越界名）；预算/取消停止不提交部分正文由
  `a_bounded_or_cancelled_run_commits_no_partial_body` 与 2.4 的新定向停止测试共同钉住。
* 行为验收锚（12/13）已恢复，见上表；planning 片当时保留未勾选的理由（锚未闭合）不再成立。
