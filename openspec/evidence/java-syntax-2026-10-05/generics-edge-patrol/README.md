# 泛型边角巡查（2026-10-05 root）

## 健康面（负结果）

- **多重界泛型方法本体**：`both(Serializable & Comparable)` raw 降级呈现（第一界作擦除类型）+ **跨界面 cast**（`((Comparable) arg0).compareTo` —— 交叉界方法调用如实）；
- **通配符读/写**：`readUp(List<? extends Number>)` → raw + `(Number) get(0)` cast；`writeDown(List<? super Integer>)` → raw + `(Object) valueOf(1)`——通配符方向语义经 cast 保真；
- `downCheck`（下界写入后读回 cast）恢复。

## 发现：String→Serializable 实参扩宽（平台扩宽族第 3 员）

多重界泛型方法调用点 `both("a","b")` 拒——"parameter 0 … declared `java.io.Serializable` presents `java.lang.String`"。与前两员（CharSequence/Comparable）**完全同因**：`DIRECT_EDGES` 只覆盖 java.util。Serializable 的实现者封闭集需 javadoc 核（String + 8 装箱型 + 数组类型的 Serializable 性质留实现判——**数组也实现 Serializable**，但呈现类型是数组时走 snapshot 通道的可能性需插桩定）。jadx 恢复（[results/jadx-GE.java](results/jadx-GE.java)）。

另：`up()`（Arrays.asList(1, 2L)）命中**异构数组**已知缺口（#3 片）——非新发现。

## 处置

Serializable 扩宽登记并入平台扩宽姊妹族（CharSequence/Comparable/Serializable 三员同机制同落点——**可合并派发**为一个任务的三张表）；不另立独立片（并入 Comparable 片的表族扩展——root 随后在两片 proposal 补互指）。

## 处置（2026-10-06，三表合并落地后重渲染）

多重界调用点恢复（`refusals = 0`）：[`results/jarde-GE-after-serializable-argument-widening.txt`](results/jarde-GE-after-serializable-argument-widening.txt)
的 `useBoth` 写出 `both((java.io.Serializable) "a", (java.io.Serializable) "b")`——擦除首界位由 Serializable 的
java.lang 九行表命中（String + 八装箱；`String`/`Character`/`Boolean` 自声明，六个 `Number` 子类经
`java.lang.Number` 的 header 到达，javadoc 的 implemented-interface 列表读法与 java.util 表的
`Properties → java.util.Map` 同规）。`up()` 的异构数组拒绝（BCI 23）逐字不变——那是
`recover-heterogeneous-array-init` 的地面，非本表族；本表**不**声称 `java.math`/`java.util.Date` 等
java.lang 之外的 JDK 实现者（负例实测见 change 的 verification）。
