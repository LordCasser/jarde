# 1.2/1.3 切片：任务落点与测试证据

本切片交付：1.2 与 1.3 所要求的“三分类 + 提升覆盖校验”的**行为验收面**（在 CLI 调用的同一入口
`Engine::class_source_with_evidence` 上），以及两个锚的 gating/边界证据。**没有改动任何恢复代码**：
`git diff` 对 `crates/`、`src/` 为空（gating 补丁只存在于 `patches/`，是测量用的临时补丁，未提交）。

## 1.2 的三分类（定向测试）

`tests/preserve_local_scope_plan.rs::the_three_answers_are_the_rendered_class`（两个 leg 各跑一次）：

* **词法 owner**：`catchOnly(Z)I` 的 `int local2 = 2;` 与 `return local2;` 都在 catch 子句内，子句闭合后
  正文不再出现 `local2`；`nestedHandlerOnly` 的内层子句局部同样只在内层子句内（外层子句只用自己的
  绑定，两者同槽不复用同一声明）。
* **可提升**：`assignedAcrossTry(Z)I` 在 `try` 之上写 `int local1;`，两条路径各一次赋值，汇合后读取；
  `assignedAcrossIf(Z)I` 在 `if` 之上声明；`nestedAcross(Z)I` 在**外层** `try` 之上声明并覆盖内层
  try、内层子句、外层子句三处写入与汇合读取。
* **类型/DA 一致**：正例类**每个成员**都被自己的 run 判为 `ContainsStatements`（无 `not recovered`
  标记），整类文本在两条 leg 上逐字节相同，且可重编执行（见下）。

## 1.3 的覆盖校验、子作用域与阻断

`tests/preserve_local_scope_plan.rs::the_lifted_declaration_carries_the_origin_of_every_region_it_covers`
把“提升声明覆盖已知读写”钉在 **source map 的 origin** 上（两 leg 相同）：

| 成员 | 声明段 origin | 各 region 的写入/读取 origin |
| --- | --- | --- |
| `assignedAcrossTry` | 7（= 首个写入所在 store） | 保护区 7 / handler 13 / 汇合读 15 |
| `assignedAcrossIf` | 5（then 臂写入） | then 5 / else 11 / 读 13 |
| `nestedAcross` | 8（最内层写入） | 内 try 8 / 内子句 15 / 外子句 22 / 读 24 |
| `catchOnly` | 10（catch 体内 store） | 声明与读取都在 `try` 语句自身的节点范围内；该节点以 derived 携带 handler entry BCI 8 |

`a_crossing_local_keeps_the_dependent_slice_refused`：`ScopePlanCrossing` 的 `flatFinally` 与
`resourceAcrossFinally`（锚 12 平铺形 + 锚 13 资源跨 finally 形）两 leg 都保持
`ExplanationOnly` + `Mixed`，逐字保留
`local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`
（原文见 fixture 渲染），`@bytecode` 覆盖列表分别为 `0 15 28` 与 `0 27 36 42 52`，正文里不出现
`int local1`/`return local1;` 的越界读取。

`a_bounded_or_cancelled_run_commits_no_partial_body`：`output_bytes=1` 与取消令牌都在
`Engine::class_source*` 上回答 `OperationOutcome::Incomplete`（不提交半个 try/catch、不提交 source map）。

## 正例的行为对照（fixture 自身）

`04-roundtrip.sh`（自测通过）：正例类文本剥掉 `//` 行后，分别用安装版 `javac --release 8` 与真
javac 8（Corretto 1.8.0_432）重编，二者在 `java -Xverify:all` 下与 fixture 自己的 class 逐行同答：

```
original v8: 1,2,3,4,6,5,9,10,11
original v8-javac8: 1,2,3,4,6,5,9,10,11
recovered (installed): 1,2,3,4,6,5,9,10,11
recovered (real8): 1,2,3,4,6,5,9,10,11
```

## 站立零回归控制

本切片的 fixture 与 1.1/1.4/1.5 的 fixture 都不改动；控制套件全部绿：

```
p3_exception_scope       1 passed
p3_nested_try            2 passed
p3_typed_catch           6 passed
p3_stated_rows           2 passed
p3_loop_try_handler_entry 6 passed
p3_try_local             2 passed
preserve_local_scope_plan 5 passed   （本切片新增）
```

## 未完成 / 边界

锚 12 与锚 13 的方法**没有恢复**，且 gating 证明这不是声明规划能翻的（`02-gating.md`、
`03-boundary.md`）。因此 tasks.md 的 1.2/1.3 复选框**未勾选**：它们的行为验收锚（12/13）仍在
region/guard 层未闭合，需要 re-slice；本切片给出可直接接手的定位、补丁、渲染与负例控制。
