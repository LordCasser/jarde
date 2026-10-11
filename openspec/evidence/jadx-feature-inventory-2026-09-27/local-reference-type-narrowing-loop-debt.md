# 循环局部引用类型错误收窄

外层赋值foreach控制基线v1的 noCastAssign 明确声明 `Object target="seed"`，循环中首句 `target=iterator.next()`，末尾return target。javac8/23真实class的LVT/StackMap保存Object；旧产品993e9f8/CLI8bdb868…四profile输出却声明String，使Object next结果赋值非法。四combined及四isolated重编共8次失败，没有Jarde运行结果；原/JADX六完整类运行腿在12 case相同。证据见[实际基线](../../changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/README.md)。

这是类型决策跨循环写入的独立问题，不是foreach绑定门；当前窄片不得混修。受控输入只改初始化为Runner.seed():Object后，旧Jarde十条三方运行腿一致，并不证明一般类型推定已修复。后续先从现有LocalVariable/复用/声明类型选择与全部写入约束分析；LVT/StackMap作为物理事实，不能仅靠字符串字面量推定目标生命周期均String。诊断决定是否扩既有合并机制，不预定新服务。

只读架构事实已核：[Luna分析](local-reference-type-narrowing-loop-debt-analysis.md)。现有decide_types以最早write决定reference type，后续仅ref/primitive冲突会Unknown；普通LVT descriptor不参与该次决策。SlotEvidence仅用于名字，不能当类型证据。Atlas结构可用，dataflow未建立；尚未在同次Program/Builder发布该local各writer及loop-Phi准确Type，因此不得猜Phi类型。

root架构约束：先用v1原class做同IR诊断，确认全部write/local identity、SSA/Phi类型和现有source-type证明如何拒绝或被绕过，再决定最小修复。LVT是物理debug事实，不能仅因有Object descriptor就直接信任；需闭合当前reuse lifetime、初始化范围以及所有写和读处类型要求。若现有SSA合流/引用类型约束已足够，优先直接复用；无debug同形状也应列为控制。不要把StackMap粗Verifier join或简单相邻BCI当Java声明类型/identity证明。当前foreach片不添加这些机制。

root 后续静态审阅：`names.rs` 的 `DebugLocal::descriptor()` 文档仅保证“unique matched generic signature”时存在，`with_type_metadata` 同时保存 erasure 与 LVTT。因此原 class 的 ordinary LVT Object 已有 raw/javap 证据，不等于 `decide_types` 当前输入已有该 descriptor；same-IR 日志必须如实记录 None。reader 保留普通 descriptor，facade 仅在唯一匹配 LVTT 的分支传递它；是否扩大现有字段传播仍需源码类型证明。

观测 v2 已由 root 实际执行：精确测试1/0/0，但结束时 free5352644608低于5GiB，守卫判定失败，runner/invocation exit1；不接受为正式诊断通过。finally clean exit0，两文件原字节恢复，4744→4745→4744，target不存在。raw显示同一local1/index0的初始String常量写、后续Object next写和Unknown loop Phi，但声明最终选String。证据与元数据/方法泛型的独立只读分析见[本轮诊断](cf11-type-analysis-root-v1/README.md)。后续要核准全部写入兼容性及读取约束，不能无条件Object化；资源满足后才重新独立验收。

空间恢复后 observation-v3 实际 exit0、guard未触发；新增Load29的Stack0/Value14完整消费链唯一到Return30，方法descriptor返回Object。独立accept-v5 actualexit0已接受同IR观察，source4744→4745→4744按原字节恢复、clean/target absent；详[实际诊断验收](cf11-type-diagnostics-accepted-root-v1/README.md)。可以据此设计最小逐写兼容规则，但尚未类型修复，no-debug/窄receiver/数组/参数/复用仍须控制，旧v2资源失败不被覆盖。
