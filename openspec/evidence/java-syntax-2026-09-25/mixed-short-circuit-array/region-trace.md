# `bastore` 短路消费者的首个失败点

根代理在实例字段修复后的工作树独立重放冻结的 `MixedArrayValue.class`（SHA-256 `f6856cc228d8efebe60aecfa2f903290a10e9d3f14ed4197ea3fa0b8a30c9351`）。原源码用 `javac --release 8 -g:none -Xlint:-options` 重编得到逐字节相同的类；原类与仅去除 JADX 虚构 `package defpackage;` 的完整类均在 `java -Xverify:all` 下输出与既存基线相同的 24 行。最新 Jarde 整类可以重编，但 `one` 仍整体引用，24 行与原类不等价；本轮 [完整报告](jarde-after-instance-field-root-report.json)、[文本](jarde-after-instance-field-root-MixedArrayValue.java)、[执行输出](jarde-after-instance-field-root-run.txt)及[逐行差异](jarde-after-instance-field-root-vs-original.diff)另存，未覆盖基线。

为判断 `jre_region_ownership_overlap` 的首因，根代理将当时工作树复制到 `/tmp/jarde-array-region-trace-src`，仅在该临时副本的 `region.rs::short_circuit_value` 和完成树所有权检查前加只读 `eprintln!` 检查点；生产工作树未加入诊断代码。临时 CLI 对同一 class 的 [`region-trace.log`](region-trace.log) 记录：外层 BCI 9 候选在图遍历、1/0 producer、精确前驱和 `consumer_block` 读取后到达消费者锚点。消费者是 BCI 29；SSA 的真实 `bastore` `0x54` 读取三个输入：`Stack(2)=ValueId(18)`、`Stack(1)=ValueId(22)`、`Stack(0)=ValueId(20)`，随后 BCI 30 `return`。`short_circuit_value` 当时的锚点只接纳 Field write、直接 `ireturn`、Invoke 和局部 Store，没有 `Operation::ArrayStore`，因此在这个 `find_map` 返回 `None`。不能把下游的 overlap 当作所有权检查本身的错误，也不能声称已进入 builder 的数组消费者证明。

规范边为 0→12/18、12→24/18、18→24/28、24/28→29。候选退出后，通用 Region 树实际包含两个 BCI 24 owner：[日志](region-trace.log)中的第一个外层 `If(9)` 的内层 `If(15)` `else_arm` 为 `Straight [24,29]`；第二个 `If(21)` 的 `then_arm` 为 `Fallback [24]`，BCI 29 又另有 `Fallback [29]`。这与 `overlapping_owner` 拒绝相符，故不可通过跳过或放宽该 validator 修复。

最小架构路径是在现有 `short_circuit_value` 的闭合图中识别真实 `ArrayStore` 锚点，然后由 builder 独立证明 `Stack(2)` 的唯一 1/0 Phi、`Stack(0/1)` 的数组和下标来源、`[Z` 组件与 `bastore`、以及数组→下标→RHS→store 的求值和异常顺序；证明失败仍由单一候选引用完整图。与实例 `putfield` 一样，锚点仅解决 Region 认领，不能把三操作数证明省略。现有 [设计](../../changes/recover-mixed-short-circuit-array-values/design.md)的 owner 防线维持不变。
