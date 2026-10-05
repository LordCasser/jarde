# 布尔实例复合 + 循环早退巡查（2026-10-05 root）——critical 第 15 锚（copy×多消费者交汇，循环乘积）

## 发现：`ok &= x > 0` 循环内复合 + 早退——可编译错行为

`for(x : xs){ ok &= x > 0; if(!ok){ return false; } } return true;`（**验证循环最常见形状**：逐元素置布尔+失败早退）——循环体内复合与条件**全部被引注吞掉**，幸存 `for(...){ if(!this.ok){ return false; } } return true;`——**隔离编译 exit 0、渲染 `true` vs 原 `false`**（布尔复合静默丢失）。诊断三族交汇：copy（BCI 21）+ 多消费者（saved producer 3 consumers，BCI 22）+ 级联。

双二进制一致（旧/新同拒）——非陈旧。族计数 **15 锚 / 4 诊断族**。

## 边界矩阵补格（双二进制）

| 形 | 结果 |
|---|---|
| 实例布尔 `&=`/`\|=`（void 位） | QUOTED（与 int `&=` 同失败面） |
| 实例布尔 `&=` 值消费位（`ok &= v; return ok;`） | QUOTED |
| 静态/局部布尔 `r &= b` | RECOVERED（#91 结论在新二进制备验成立） |
| 循环内实例布尔复合 + 早退 | **QUOTED 可编译错**（本锚） |

## 处置

四族 Requirement 已覆盖；账本 15 锚。
