# 复合赋值与后缀旧值巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/CM.java](fixture/CM.java)、[CM2.java](fixture/CM2.java)（`--release 8`）：
- **复合赋值链**（`+= -= *= /= %=`）恢复为展开形（`x = x + 5`——javac8 本就无复合 opcode，忠实）；**表达式内复合赋值**（`y = (x += 5)`）恢复（旧值捕获=+5 后新值，正确）；**静态字段复合**、**前缀**（`++i`/`++CM.a`——无旧值快照）、**后缀拆两语句**（`i++; j = i;` 无快照）全部恢复；
- `incField` 前缀字段双形恢复。

## 发现：后缀旧值快照（第 9 个新证缺口，响亮拒绝）

javac 为 `i++`（值被消费时）发射快照 dance：`iload i(旧); iinc i; …消费(旧)`。判别（四形）：

| 形 | 结果 |
| --- | --- |
| `int j = i++;`（赋值捕获旧值）/ `return i++ + 10;`（表达式内消费旧值） | **拒**（"the value at BCI N is the value local 0 held at BCI M, and the slot does not hold it at BCI N" 时间引注 + "reads `local1`, no statement declared that local"） |
| `++i`（前缀，无快照）/ `i++; j = i;`（拆语句，无快照） | 恢复 |
| 复合赋值的旧值捕获（`y = (x += 5)`——捕获的是**新**值） | 恢复 |

**jadx 有解**：`i++` 直接还原（jadx incDec 输出 `int i2 = (i - 1) - 1;` 的旧值算术等价形）或语句级 `i++`。缺的是 **load-旧值在 iinc 之前入栈、跨 iinc 被消费**的时间建模（快照值是 distinct SSA 值）。

## 处置

第 9 个新证窄缺口（呈现域：后缀自增的旧值快照——load-before-iinc 的时间序消费）。登记 + 立窄片。
