# 同一短路决策图的消费者矩阵（Java 8）

已验收的消费者与数组元素的冻结正例证明：图前半部分可以相同，恢复边界主要随**唯一消费指令**变化。不能把“JADX 能写短路语法”笼统地当作一项能力；每种消费者还要证明自己的类型与求值顺序。

| 消费点 | 冻结样例与事实 | 当前裁决 |
| --- | --- | --- |
| `putstatic Z` | [混合字段写入](mixed-short-circuit-field/analysis.md)：`andOr/orAnd` 两方法，原/JADX/当前 Jarde 完整类 16/16 行值与 RHS 次数一致 | 已在 `recover-mixed-short-circuit-field-values` 独立验收；复用闭合图、SSA/Phi 与字段身份 |
| `ireturn`，方法描述符 `Z` | [直接返回](mixed-short-circuit-return/analysis.md)：原/JADX/当前 Jarde 八行值与 RHS 次数一致；`(Z)I` 同形负例保留完整引用 | `recover-short-circuit-return-values` 已独立验收；只增加直接返回消费证明 |
| `invokestatic (Z)V` | [单实参静态调用](mixed-short-circuit-argument/analysis.md)：原/JADX/当前 Jarde 八行值与 b/c/sink 调用次数一致；多参数/实例近例仍整图引用 | `recover-short-circuit-invocation-arguments` 已独立验收；仅开放真实 Methodref、唯一 Phi 参数绑定 |
| `istore` 写入 boolean local | [局部初始化](mixed-short-circuit-local/analysis.md)：BCI 21 的唯一 Phi consumer 是 `istore_1`，后续 BCI 22/26 是两次 local load；当前原/JADX/Jarde 八行结果、字段值与 b/c 次数一致，Jarde 一次声明并复用 local 名，14 个 BCI 全可追 | `recover-short-circuit-local-values` 的正向能力已独立验收；同一 RegionPath 与显式 Boolean 消费闭合才定型，任务 1.2 的无名/各门独立负例仍待补齐（7/8） |
| `putfield Z` | [实例字段写入](mixed-short-circuit-instance-field/analysis.md)：稳定 CLI 快照因 BCI 20 Region owner overlap 整体引用；[最新独立验收](../../changes/recover-mixed-short-circuit-instance-field-values/verification.md)的原/JADX/Jarde 整类 Java 8 重编与 `-Xverify:all` 16 路完全一致，receiver 只调用一次，null 时 RHS 后才 NPE，13 个 BCI 全可追 | 已在闭合短路图中证明 Phi value 与 receiver 两操作数、单一 receiver owner、field identity 与延迟 null fault；没有放宽 Region owner 检查。外部 owner overlap 的隔离负例仍是任务 2.4 |
| `[Z` 的 `bastore` | [数组元素写入](mixed-short-circuit-array/analysis.md)：旧 Jarde 整体引用，24/24 行不等价；[最新独立验收](../../changes/recover-mixed-short-circuit-array-values/verification-root.md)中原/JADX/Jarde 完整类 Java 8 重编与验证执行 24 路一致，15 个 BCI 全可追 | 已在原闭合图中加真实 `ArrayStore` 锚点，并证明 Phi 为第三操作数、array/index 独立唯一来源及物理顺序、`[Z` 与 `bastore`、RHS 后 null/bounds fault；`[B`、未知组件、额外 use/边拒绝 |

局部 `istore`、实例 `putfield Z` 与 `[Z` `bastore` 均有永久可复核基线，三者正向能力均已独立验收；局部与实例字段仍有各自尚未闭合的隔离负例任务。实例字段和数组两个旧正例的首因均是消费者锚点漏掉相应写入，修复后 Builder 仍各自完成操作数证明。`if (condition)` 这种直接控制转移没有 `1/0` Phi，不属于这张值消费者矩阵。
