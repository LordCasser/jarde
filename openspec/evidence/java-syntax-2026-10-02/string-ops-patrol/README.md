# 字符串操作域巡查（2026-10-02）

EM-05 字符串域扩验（主线 `4b782a68`）。固定转录 [fixture](fixture/)（S1：charAt/substring/indexOf/length/toUpperCase/valueOf/format/拼接/replace/equals/引用相等；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| charAt/substring/indexOf/length/toUpperCase/valueOf/format/拼接/replace/equals 于 append 链 | 全部恢复（含 `(Object)` cast 呈现与 varargs 数组），行为一致 |
| **`t == t.intern()` 引用相等结果传 `append(Z)`** | 语句被引："the parameter 0 … declared `boolean` presents `int` and this layer has no proven conversion to boolean"（BCI 156）→ 重编后该语句缺失（行为差） |

## 根因

引用相等 `==`（refs）降低为 `if_acmpne 155 / iconst_1 / goto 156 / iconst_0`——分支选择 0/1 的 int 进入 `append:(Z)` 实参位；该位的 int→boolean 转换无证据通道（布尔上下文的既有证明覆盖赋值/return/条件位，未覆盖"比较结果直接作实参"——`equals` 前一语句同链恢复是因为它的值是 invoke 返回 boolean）。

## 处置方向

`recover-ref-eq-boolean-argument`（窄切片）：`if_acmpXX` + iconst_0/1 模式的引用相等值在**调用实参位**按 boolean 呈现（`t == t.intern()` 直译），复用既有布尔分支值的 SSA 证明形态（布尔上下文切片已建），落点在实参位转换判定而非新证明机制。同模式数值相等（`if_icmpXX`）若同病一并覆盖并在报告区分。

**已实现**（见 [repeq-variants/](repeq-variants/README.md)：判定点定位、变体前后、三方对照与 SHA）：S1 完整恢复、行为一致；序分支/零测试边界保持拒绝。

原 class 为行为基准。
