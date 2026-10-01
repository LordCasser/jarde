# EM-19/CF-20 位运算巡查：核心健康 + 短路值拼接消费缺口（2026-10-01）

位运算域（[EM-19](../../../jadx-feature-inventory-2026-09-27/expressions-misc.md) 多种位运算、[CF-20](../../../jadx-feature-inventory-2026-09-27/control-flow.md) 位掩码条件）的扩验收证（主线 `03552a2e`）。固定转录 [fixture](fixture/)（B1–B5，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| B1.bitOps：`&`/`\|`/`^`/`<<`/`>>`/`>>>`/`~`、位掩码条件（含 `& getMask()` 非常量掩码）、表达式与拼接混合 | 完整恢复，行为逐字一致（`ho:111:165:35`/`:13:252:1`） |
| B2.longOps：long 复合位赋值（`^=`/`\|=`/`&=`/`>>>=`）与移位组合 | 完整恢复 |
| B3：`(v&1)!=0 && (v&2)!=0` 等**直接 return** 短路链（含三层 `\|\|`+`&&` 混合、位掩码与普通比较混排） | 全部完整恢复 |
| B5.s3：短路链**存布尔局部 + return 局部** | 完整恢复（`boolean local1 = (arg0 & 1) != 0 && …; return local1;`） |
| **B5.s1/s2、B4.v1–v3、B2.compound：短路链存局部后由拼接消费**（`hasA + ":"`） | 整方法退化："the short-circuit chain from BCI 3 through 9 reaches a shared value consumer at BCI 17, but this slice has no SSA proof for that value" |

## 根因

既有 `recover-short-circuit-local-values` 切片（7 项已验收）覆盖"短路值存局部 + 简单消费（return）"；当消费方是**拼接链实参**（`append(Z)` 位）时，短路值切片的 SSA 值证明不覆盖——错误信息明确指向该切片自身边界（"this slice has no SSA proof for that value"）。判别链完整：直接 return ✓ → 存+return ✓ → 存+拼接 ✗（唯一变量 = 消费方种类）。B2.compound（复合位赋值前缀 + 双短路 + 拼接）同因，非独立根因。

## 处置方向

`recover-scv-concat-consumers`：短路值切片的消费方集合扩展到拼接链 `append(Z)`/`append(Object)` 实参位——值经 append（Z） 或装箱 append（java/lang/Object）消费时按既有布尔呈现。判别链 B3/s3 为逐字不变锚；负例（值流断裂/共享消费者未证）保持整方法退化。属既有切片消费方集合扩展，无新机制。

原 class 为行为基准。
