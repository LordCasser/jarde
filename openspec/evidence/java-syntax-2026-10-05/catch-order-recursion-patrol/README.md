# catch 层级序与递归巡查（2026-10-05 root）

## 健康面（负结果）

- **自递归 + 同型三元**：`fib`（`n < 2 ? n : fib(n-1)+fib(n-2)`）完整恢复——递归调用在三元内无碍；
- **互递归（if/else 形）**：`even`/`odd` if/else 对照形完整恢复（CO2）——互递归本身不阻塞。

## 发现一：互递归的三元形=三元片变体 3（已立项域）

`even`（`n==0 ? true : odd(n-1)`）拒——诊断 "the two values joined at BCI 14 do not have a conditional Java type"（**三元异型**诊断）。判别（对照矩阵）：同型 int 三元+自递归 ✓、if/else 互递归 ✓、**布尔字面量 vs 调用返回的三元** ✗——分支类型呈现（boolean 字面量 vs boolean 方法返回）在汇合点无法统一。归 [#1 三元拆分片](../../../changes/recover-divergent-ternary-statement-split/)第 3 变体锚（异型引用/上转型汇合/短路条件/**字面量-vs-调用**）。

## 发现二：catch 链+finally（local-scope 锚家族+1）

`order`（三段 catch 子类→平级→父类序 + finally）拒——"local 1 crosses a quoted fallback region"（local-scope 同因，锚家族第 5 数据点）。catch **顺序**本身在可恢复形中已由早前巡查证实保留。

## 处置

均不新立；三元片变体 3 锚与 local-scope 数据点分别补入两 change。
