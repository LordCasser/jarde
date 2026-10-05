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

## 处置

扩展 `preserve-postfix-fallback-soundness`（改名语义见其 proposal——触发族泛化为 copy/old-value 两族）；账本 critical 行更新；chained-field-assignment 片补锚。
