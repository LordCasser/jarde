# 多维/锯齿数组 + 初始化 dance 值位巡查（2026-10-05 root）

## 健康面（负结果——数组族主体全过）

[fixture/MD.java](fixture/MD.java)、[MD2.java](fixture/MD2.java)、[MD3.java](fixture/MD3.java)（`--release 8`）：
- 锯齿初始化器（静态字段 + 方法返回）：`new int[][]{new int[]{1}, new int[]{2, 3}, …}`——完全显式等价形；
- 规则/部分创建（`new int[2][3]`/`new int[2][]`）、数组元素存取（`reg[1][2] = 9`）、嵌套 for（索引外层+增强内层）全恢复；
- **初始化 dance 的四个合格位**：局部赋值（`int[] x = new int[]{1,2}` ✓）、字段赋值（putfield ✓）、方法实参（✓）、外层初始化器的元素（✓）；裸数组立即消费（`new int[2].length` ✓）、裸数组返回（✓）。

## 发现：初始化 dance 值的非法位（第 8 个新证缺口，响亮拒绝）

javac 为 `new int[]{7}` 发射 dance：`newarray; dup; iconst_0; bipush 7; iastore`——dup 引用**跨内层 iastore 存活**后交给消费方。判别（六位）：

| 位置 | 结果 |
| --- | --- |
| 局部赋值 / 字段赋值 / 实参 / 外层初始化器元素 | **恢复** |
| `partial[0] = new int[]{7};`（**预存数组的元素存储**，aastore 值位） | **拒**（"copy at BCI 7 has no proved local assignment"） |
| `return new int[]{9}[0];`（**立即下标**，arrayload 接收者位） | **拒**（"copy at BCI 3 has no proved local assignment"×2） |

**jadx 两形皆完整恢复**（[results/jadx-MD.java](results/jadx-MD.java)）。对照证伪两个候选机制：非"dance 本身"（四位合格）、非"立即消费"（`new int[2].length` 裸形过）——是 **dance 的 dup 值作为任意表达式值**（非具名存储目标）时的消费方建模缺失。

## 处置

第 8 个新证窄缺口（呈现域：初始化 dance 值的通用消费方呈现）。登记 + 立窄片 `recover-array-initializer-value-positions`。
