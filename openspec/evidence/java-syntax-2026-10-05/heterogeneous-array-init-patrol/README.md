# instanceof/cast 与异构数组初始化巡查（2026-10-05 root）

## 健康面（负结果）

- **instanceof + cast 模式**完整恢复（`if (o instanceof String){ String s = (String) o; …}` 含 else）；
- **三元内双重 cast**（`instanceof ? ((String) o).trim() : "?"`）完整恢复；
- **装箱上转型赋值**（`Object l = Arrays.asList("a","b")`）恢复；
- **混合装箱三元** `true ? Integer : Double` 的数值提升展开（`Double.valueOf((double) Integer.valueOf(3).intValue())`）是 **javac 8 自身 codegen 的忠实呈现**（JLS 15.25 binary numeric promotion；常量条件折叠正确）——非缺口。

## 发现：异构数组初始化器（响亮拒绝，已证缺口）

`static Number cov(){ List<? extends Number> ln = Arrays.asList(1, 2L); … }` 整方法拒绝：**"the array initializer element at BCI 23 is presented as `java.lang.Integer`, while the array component …"**。

字节码（javap）：`anewarray Number` + 元素 0 为 `Integer.valueOf(1)`、元素 1 为 `Long.valueOf(2L)`——**元素静态类型异构**（都是 `Number` 子类、擦除后同组件 `Number`），呈现层要求元素呈现类型与组件类型一致而拒。jadx 呈现见 [results/jadx-CT.java](results/jadx-CT.java)（其对 `Arrays.asList(1, 2L)` 有独立处理）。

**判别**：同构装箱数组（`Arrays.asList("a","b")` String 元素）恢复（`up()` 实证）——缺口仅在**元素呈现类型互异且均需上转型到组件类型**的初始化器。响亮拒绝（非静默）。

**处置**：登记窄缺口（呈现域：异构元素上转型一致性证明——把 `Integer`/`Long` 元素呈现为对组件 `Number` 的合法赋值即可，无需新机制）。待窄片立项。
