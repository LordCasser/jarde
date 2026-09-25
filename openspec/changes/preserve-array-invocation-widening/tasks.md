## 1. 固定数组关系与目标选择输入

- [x] 1.1 将 `array-reference-conversion/` 的 core 收成最小永久 Java 8 fixture，记录1632B/16Code、28项原/JADX相同、jarde三引用/整类 javac 失败；保留37项全样本中的数组写入独立缺口。root统一 census/fingerprint。
- [x] 1.2 固定 `ArrayReferenceOverloadTarget` 的无checkcast局部上溯与更具体重载；`int[]↛Object[]` 用纯数组关系单元判据，未知类继承与 null/零长/副作用用合法 class 对照，逐个验证 Methodref 和三方阶段，不构造验证器拒绝的伪反例。追加 `int[][]→Cloneable[]`、`String[][]→Serializable[]`、`int[]→Cloneable` 的 source-only 合法类对照，内置关系之外仍拒绝。

## 2. 在现有调用参数处证明封闭数组上溯

- [x] 2.1 用现有 Type/descriptor 事实定义有界数组组件关系，仅对结构可证明的 Object/Cloneable/Serializable 内置数组超型及同型准入；不读取外部层级、不放宽一般 `meeting_position`。纯判据测试 `int[][]→Object[]`、`int[][]→Cloneable[]`、`int[]→Cloneable` 与 `int[]↛Object[]/Cloneable[]`，合法class测试 `String[][]→Object[][]/Serializable[]` 和未知继承。
- [x] 2.2 对所有准入的非同型数组实参复用 `cast_argument` 固定目标 descriptor 静态类型；目标选择样本必须恢复 `Object[]` 重载且不改局部推导、对象身份、异常、求值次数。
- [x] 2.3 保持原实参/调用来源、已有预算/取消与 fallback；默认/all及replay正文稳定，未知关系保留参数生产者的真实BCI。

## 3. 完整类与 root 验收

- [x] 3.1 永久 fixture 与 core 的原 class/jarde 完整源码实际编译执行逐项相同且零引用；目标选择样本必须执行一致，JADX只有成功编译后才算执行 oracle。
- [x] 3.2 root 独立审查封闭子类型判据与实际 Methodref 重载目标，重放所有输入；复跑调用参数、普通数组、cast、泛型函数式目标与deferred顺序相邻回归，外部层级债务另案。
- [x] 3.3 root 统一 census/fingerprint、fmt、适当 Cargo 回归与 OpenSpec strict；完成证据后才勾选任务。
