# 实现者复核：引用槽生命周期

## 现有链路与本片职责

`report::recover` 在方法 Code、帧、SSA、canonical CFG 与 Region 准备后调用 `reuse::plan`。`Plan` 持有每槽的 `SlotEvidence` 与按 BCI 的 `variable_at` 映射；`NameTable` 为 `Split` 生成独立 `LocalVariable` 名称，声明规划和 `build::decide_types` 再消费各段的写入事实。原问题发生在身份规划：`array_retype_split` 要求每次写入都能读成数组形状，因而无法把 `int[]` 后面的 `ArrayList` 作为同槽的第二身份。

新规则沿用数组路径的每写定义 owner、trivial replacement 代表值、每读唯一 owner、旧定义跨界 use 拒绝和 ranges/`variable_at` 映射。数组路径继续优先运行；普通引用规则只处理无 LVT、非参数/资源、完整且无 clone/unreachable 的正常 CFG，并要求每个写入有明确可拼写的引用类型以及至少两种不同类型。新增 CFG 走查对每个段界从后段访问块的后继开始，若可到达前段访问块则拒绝；同一块中先后访问不会因块 ID 相同被拒绝，但该块的后继若真正回到前段访问仍会拒绝。全部新增循环在每次处理时 poll 并计入 `AnalysisSteps`。未知帧的 `Object` 兜底和 `null` 均不参与类型分类。

## root 真实 SSA 复核

独立 `analyze_method_ir` 直接读取两份冻结 javac23 class 的 SSA/CFG，而不是从渲染 JSON 反推。实际编译 argv、rlib/source hash、exit 与转储在 `root-ir-audit-v3-run.json` 和 `root-ir-audit-v3-run.stdout`。这是一份独立方法分析的证据，不冒充 class-source 内部那次运行的序列化事实。

- ArrayThenList.run([I)I：槽2写 BCI3 为 ValueId(10)，frame明确 `[I`；写44为 ValueId(33)，明确 ArrayList。读4唯一归属3，读45/54/64/75唯一归属44。旧定义直接uses为读4以及块13的phi输入；无BCI的phi use以块BCI13定位，早于44。正常边为0→13、13→20、13→37、20→13，后段37没有回到前段访问的路径。
- ListThenMap.run(I)I：槽2写7为 ValueId(5) ArrayList，写40为 ValueId(23) HashMap。读8/17/28唯一归属7，读41/52/56唯一归属40；前值全部直接uses小于40，整个方法在一个普通块内，没有回边。

旧源码把后段列表写给 `int[] local2`、后段映射写给 ArrayList，完整重编日志确认类型错误；不是泛型调用、构造器或字符串拼接失败。

### 直接uses不足以证明旧引用死亡

root另用真实 SSA 查出初候选CLI1的HeldUse元数据多发布一个别名，虽然拒绝源码完全不变。store9的ValueId(5)直接use仅load10；load10产生Stack0 ValueId(6)，调用20在store19之后消费它。故仅扫描store代表值的uses会误分段。该失败及候选原字节均保留，CLI1不作为最终验收版本。

最终reference-only检查沿既有Load/Store/Duplicate/CheckCast的SSA身份转发继续检查uses，仅已有replacement证明的平凡phi可以继续；其它栈复制/非平凡phi保守拒绝。普通调用和运算的结果不视作输入引用的别名。SSA栈编号是绝对位置，不能把Stack0当成所有指令的栈顶：新增双JDK NonzeroHeldUse的checkcast13读取Stack1 ValueId(7)、写Stack1 ValueId(8)，store24后调用25仍读ValueId(8)，必须不分段。实际转储与命令见 `root-held-use-audit-v1-*`、`root-nonzero-held-audit-v1-*`。所有新增扫描有计费/轮询；同类型写在已计费类型循环内判定后退出，不进入无必要的身份闭包。

## 基线边界

归档 patrol 的能力统计按冻结矩阵分别为 16/16 无调试腿失败、8/8 不同名 LVT 腿成功、8/8 同名 LVT 腿失败；JADX 对 32 腿完整重编并运行一致。首轮错误源文件名 harness 失败仍保留，但不计入能力数字。本实现不改写这些历史结果，也不把本片局部生命周期规则外推为完整 LG 或 EM-20 已完成。

聚焦测试中的 8 个不同名 LVT 正例和 8 个同名 LVT 失败例分别取自 patrol 两批 manifest 的 `flavors.baseline.sources`；class fixture 从对应原始 JAR 提取，baseline source 逐字冻结。负例 fixture 按 `/private/tmp/jarde-ref-ref-slot-negative-20261009/replay-cli9-v3/manifest.json` 的八个 case dict 项保存原始 class 与 `jarde_source`：same-type、cross-phi、cfg-loop-phi、handler、parameter-header、unknown-null、held-use、cfg-backedge-disjoint。测试逐字比较新库完整 `ClassSourceReport.text` 与这些实际 baseline source，不依据用例名推测预期文本。held-use 的 `javap` 明确展示 BCI 10 读取旧引用、BCI 19 写入新 `ArrayList`、BCI 20 调用 `pick` 保留旧引用；其精确 SSA 代表值/uses 已按上述独立审计确认。
