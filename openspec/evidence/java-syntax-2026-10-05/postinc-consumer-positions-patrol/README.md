# 后缀自增消费位全景巡查（2026-10-05 root）——critical 第 8/9 锚（消费位扩展，三族封闭确认）

## 发现：两个新消费位的可编译错文本

- **第 8 锚（链实参位）**：`sb.append("n").append(t++)`——渲染 `saved0 = sb.append("n"); local0 = local0 + 1; return sb.toString() + ":" + local0;` **隔离编译 exit 0、`n:6` vs 原 `n5:6`**（旧值 5 被吞、iinc 呈现保留）；诊断属**旧值族**（"value local 0 held at BCI 10"）；
- **第 9 锚（方法实参位）**：`list.set(idx++, "X")`——渲染含 `return PC.idx;` **隔离编译 exit 0、`0` vs 原 `1`**（idx++ 与 set 全吞）；诊断属 **copy 族**（"copy at BCI 28 has no proved local assignment"）。

## 安全面（负结果）

- `return counter++`（**返回位**）——**整方法响亮拒绝**（"not recovered"）= 安全；
- 类级 fixture 因 viaReturn 空/缺 return 不可编译（碰巧掩蔽 8/9 锚——与 compoundSelf 同一掩蔽模式，**单方法隔离是必要工序**）。

## critical 族定格：9 锚 / 3 诊断族 / 5 消费位

| 族 | 锚 |
|---|---|
| 旧值 store | `i=i++`/`i=i--`/`a[i]=i++`/`i+=i+++1`/**`sb.append(t++)` 链实参** |
| copy | `flags\|=` void/值消费/**`list.set(idx++,v)` 方法实参** |
| 依赖链 | `elems[size++] = t` |

消费位谱系：自赋 / 数组存 RHS / 复合 / 链实参 / 方法实参（返回位=安全拒绝）。**无第 4 族**——泛化按"语句效果被引注吞掉后剩余文本碰巧可编译"的结构性质覆盖即可，无需按族枚举。

## 处置

soundness spec 补第 4 scenario（消费位泛化）；账本 critical 行更新至 9 锚。
