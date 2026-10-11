# 循环局部引用类型错误收窄

外层赋值foreach控制基线v1的 noCastAssign 明确声明 `Object target="seed"`，循环中首句 `target=iterator.next()`，末尾return target。javac8/23真实class的LVT/StackMap保存Object；旧产品993e9f8/CLI8bdb868…四profile输出却声明String，使Object next结果赋值非法。四combined及四isolated重编共8次失败，没有Jarde运行结果；原/JADX六完整类运行腿在12 case相同。证据见[实际基线](../../changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/README.md)。

这是类型决策跨循环写入的独立问题，不是foreach绑定门；当前窄片不得混修。受控输入只改初始化为Runner.seed():Object后，旧Jarde十条三方运行腿一致，并不证明一般类型推定已修复。后续先从现有LocalVariable/复用/声明类型选择与全部写入约束分析；LVT/StackMap作为物理事实，不能仅靠字符串字面量推定目标生命周期均String。诊断决定是否扩既有合并机制，不预定新服务。

只读架构事实已核：[Luna分析](local-reference-type-narrowing-loop-debt-analysis.md)。现有decide_types以最早write决定reference type，后续仅ref/primitive冲突会Unknown；普通LVT descriptor不参与该次决策。SlotEvidence仅用于名字，不能当类型证据。Atlas结构可用，dataflow未建立；尚未在同次Program/Builder发布该local各writer及loop-Phi准确Type，因此不得猜Phi类型。

root架构约束：先用v1原class做同IR诊断，确认全部write/local identity、SSA/Phi类型和现有source-type证明如何拒绝或被绕过，再决定最小修复。LVT是物理debug事实，不能仅因有Object descriptor就直接信任；需闭合当前reuse lifetime、初始化范围以及所有写和读处类型要求。若现有SSA合流/引用类型约束已足够，优先直接复用；无debug同形状也应列为控制。不要把StackMap粗Verifier join或简单相邻BCI当Java声明类型/identity证明。当前foreach片不添加这些机制。

root 后续静态审阅：`names.rs` 的 `DebugLocal::descriptor()` 文档仅保证“unique matched generic signature”时存在，`with_type_metadata` 同时保存 erasure 与 LVTT。因此原 class 的 ordinary LVT Object 已有 raw/javap 证据，不等于 `decide_types` 当前输入已有该 descriptor；same-IR 日志必须如实记录 None。后续需追 reader→facade→DebugLocal 的实际元数据传播，再决定既有类型/SSA约束是否够用，不能直接按调试表修复。私有观测 v2 尚未应用/编译/运行。
