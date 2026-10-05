# 实例字段复合赋值健全性巡查（2026-10-05 root）——critical 族泛化（第 5/6 锚）

## 发现：可编译且行为完全不同（第一不变量违反，整类 setter 静默空操作）

- `BF`（位标志状态机）：`enable`/`disable`（`flags |= 1 << bit` / `flags &= ~(1 << bit)`，**void 实例复合**）渲染体仅剩 `return;`——**剥注释后 javac exit 0、行为 `false/false/false/true/0` vs 原 `true/false/true/false/3`**（全部置位操作静默丢弃）；
- `BG.ienable2`（实例复合+值消费）：复合语句被引注吞掉，剩余 `return this.flags;` 同样可编译且丢副作用。

## 判别矩阵

| 形 | 结果 |
|---|---|
| `staticFlags |= 1 << bit`（静态复合——无 receiver dup） | 恢复 |
| `(flags & (1 << bit)) != 0`（读测试） | 恢复 |
| **`this.flags |= …`（void 实例复合）** | **拒 + 可编译错文本** |
| **`this.flags |= …; return flags;`（值消费）** | **拒 + 可编译错文本** |

机制：javac 发 `aload_0; dup@1; getfield; …; ior; putfield`——**receiver dup 跨 getfield+putfield**；诊断两行（"instruction at BCI 1 belongs to no shape" + "the copy at BCI 1 has no proved local assignment"）属 copy 家族文本。**jadx 正确**（`this.flags |= 1 << i;` 直排）。

## 与既有 critical 的关系

与 postfix 自赋值四锚**同缺陷类、不同触发**：postfix 族触发于 old-value store 证明失败（"the value at BCI N is the value local X held…"）；本族触发于 copy/receiver-dup 证明失败（"the copy at BCI N has no proved local assignment"）。**泛化结论**：任何引注 fallback 剩余文本不得可编译出不同行为——健全性守卫须覆盖两诊断族。恢复侧：dup 跨 getfield+putfield 的复合 RMW（与在队 chained-field-assignment 的 dup-跨-putfield 相邻，实现时核实是否同门）。

## 边界锐化（root 复核，修正提取 bug 后的权威矩阵）

| 形 | 结果 |
|---|---|
| 静态复合全类型（String/long/double，`getstatic…putstatic` 无 receiver dup） | **恢复**（SB 链/`+` 折回） |
| 实例 `this.i++` 语句位（iinc 语义等价路径） | **恢复**（`this.i++;`） |
| 实例复合赋值（任何类型：int `\|=`、String `+=` 简单/复杂 RHS、long `+=` dup2_x1） | **QUOTED**（void 单语句形剩 `return;` → 可编译错） |

失败判据因此精确为：**实例 receiver dup/dup2_x1 参与的复合 RMW**；静态与语句位自增不在族内。**边界再锐化（2026-10-05 晚，root 双二进制矩阵）**：同形状 `aload_0; dup; getfield; <op>; iload_1; putfield` 下，**仅 iadd/isub 恢复**（累积器惯用法既有规则覆盖——FA.add/addAll/循环累积全部恢复，双二进制一致），`ior/ixor/iand/imul/idiv/ishl` 全 QUOTED（[DV fixture](fixture/DV.java)，新二进制结果归档 jarde-DV-newbinary.txt）。**#84 矩阵"实例复合全 QUOTED"系过度泛化**——SG 探针恰好无 iadd 形；权威失败面 = 实例 receiver-dup 复合 **且** 运算符 ∉ {add,sub} **或** RHS 复杂。critical 锚计数不变（锚 5/6/11 均非 add/sub 简单形；诊断族守卫不受影响）。

root 自查：本轮曾因 `
    ` 前缀匹配 8 空格行的提取 bug 得出"SG 全恢复"的假反转，打印方法体后修正——教训同 handoff 假零族（提取器必须在证据 README 附原始渲染切片）。

## 处置

扩展 `preserve-postfix-fallback-soundness`（改名语义见其 proposal——触发族泛化为 copy/old-value 两族）；账本 critical 行更新；chained-field-assignment 片补锚。
