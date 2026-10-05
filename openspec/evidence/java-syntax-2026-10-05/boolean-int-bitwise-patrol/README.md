# 位运算与布尔混合巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/BW.java](fixture/BW.java)（`--release 8`）——**纯型位运算与位技巧全过**：
- `boolean ^ boolean` 与 `int ^ int`（同型）分别恢复；
- **移位量超宽**（`x << 33`，JVM 对 int 移位 &31）如实呈现 `<< 33`（呈现写 33、语义由 JVM 定义——编译源同形即等价）；
- **常量移位折叠**（`1 << 31` → `-2147483648`）忠实（javac 折叠产物）；
- **Kernighan 位计数循环**（`x &= x-1`）完整恢复（位技巧循环无碍）。

## 发现：boolean–int 混合位运算（响亮拒绝，第 6 个新证缺口）

两形被拒，拒绝文本同为 **"the bitwise operator `^`/`&` at BCI N has operands presented as `int` and `boolean`, which no Java integer…"**：

| 形 | 源 | 拒因 |
| --- | --- | --- |
| `r ^= x`（boolean 累积 xor，javac 编为 int 计数器 `local1 = local1 ^ (x?1:0)` 再 `% 2 != 0`） | `mix` | 循环内 xor 的一侧是 int 计数器、一侧是 boolean（来自数组元素 load） |
| `a & !b`（javac 编为 boolean & int(0/1)） | `andNot` | `!b` 编为 `xor 1` 产生 int |

**jadx 两形皆有解**（[results/jadx-BW.java](results/jadx-BW.java)）：`mix` 直接推断 boolean 累积变量 `z ^= z2`；`andNot` 呈现 `z & (!z2)`。即字节码里的 int-化布尔值可被一致地还原为 boolean（前提：该 int 值的**全部**生产/消费都在布尔位运算内——`% 2 != 0` 的出口是既有 boolean-from-int 呈现域）。

## 处置

第 6 个新证窄缺口（呈现域：boolean–int 混合位运算的布尔一致还原——int 化布尔操作数在纯布尔位运算上下文中回投为 boolean）。登记 summary + 随后立项窄片。
