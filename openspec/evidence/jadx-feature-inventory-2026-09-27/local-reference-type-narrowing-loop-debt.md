# 循环局部引用类型错误收窄

外层赋值foreach控制基线v1的 noCastAssign 明确声明 `Object target="seed"`，循环中首句 `target=iterator.next()`，末尾return target。javac8/23真实class的LVT/StackMap保存Object；旧产品993e9f8/CLI8bdb868…四profile输出却声明String，使Object next结果赋值非法。四combined及四isolated重编共8次失败，没有Jarde运行结果；原/JADX六完整类运行腿在12 case相同。证据见[实际基线](../../changes/recover-proved-iterator-assignment-foreach/results/controlled-baseline-root-v1/README.md)。

这是类型决策跨循环写入的独立问题，不是foreach绑定门；当前窄片不得混修。受控输入只改初始化为Runner.seed():Object后，旧Jarde十条三方运行腿一致，并不证明一般类型推定已修复。后续先从现有LocalVariable/复用/声明类型选择与全部写入约束分析；LVT/StackMap作为物理事实，不能仅靠字符串字面量推定目标生命周期均String。诊断决定是否扩既有合并机制，不预定新服务。
