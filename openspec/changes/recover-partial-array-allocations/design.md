## Context

Java 8 javac 对 `new int[n][]` 产生 `anewarray [I`，常量池条目写的是**组件**数组类型；指令再增加一个外层维度。`new int[a][b][]` 则产生 `multianewarray [[[I, 2`，常量池写完整结果类型，立即数 `2` 只说明本次分配前两维。当前 `Operation::NewArray` 只含 `element` 和分配的 `dimensions`，`ExprKind::NewArray` 只有相同数量的 `lengths`；`presented_of`、`array_of_value` 和 `emit` 均以此数量充当总维数。`decode::array_creation` 因此主动拒绝数组组件的 `anewarray` 和维数不相等的 `multianewarray`。

根从源码编译的 `core-no-overload/` 完整类为 777 B/10 Code，SHA-256 `487389b770dbfdd226ae9ae3ff87c9d6be9df4e7df01781505f5d2a55e10e8f2`。69 项含 `-1/0/2` 尺寸、二维组合、尺寸函数按顺序抛错、局部/字段/长度消费者与完整分配对照。原 class 与 JADX 完整源码均可编译执行且逐项一致；冻结 CLI `ecab8244…` 有 18 处引用且整类 javac 失败。最初 72 项类另外含 `pick(Object[])` 重载，不能把其独立调用参数数组协变缺口强加本案。

## Goals / Non-Goals

**Goals:** 恢复已证明的基本元素类型、数组总维数和分配前缀维数，写出 `new T[d1]…[dk][]…[]`，同时保持所有已分配尺寸表达式与异常语义。

**Non-Goals:** 不推导未知数组引用的形状；不扩展重载调用参数的数组协变、数组初始化器、数据流合并或自动补全缺失尺寸；不处理非法 descriptor 或指令。

## Decisions

1. 给现有 `Operation::NewArray` 与 `ExprKind::NewArray` 增加一个**数组总维数**事实。继续用现有 `dimensions`/`lengths` 表示实际分配维数，要求 `1 ≤ allocated ≤ total ≤ 255`。这只是指令已给出的形状，不建第二种数组 AST 或数组求解层。
2. `newarray` 的总维数为 1；普通类组件的 `anewarray` 为 1；数组组件的 `anewarray` 为组件 descriptor 维数加 1。`multianewarray` 由数组 descriptor 给总维数，立即数给已分配维数；必须检验 `count ≥ 1` 且不超过总维数。基本元素类型仍经现有 descriptor/type 映射，无法合法拼写的类型继续来源完整拒绝。
3. 构造表达式仍按现有 stack operands 顺序渲染 `lengths`，`emit` 先写有尺寸的 `[expr]`，再写 `total - lengths.len()` 个空 `[]`。`presented_of` 和 `array_of_value` 都用总维数，不再把空维度误判成较低秩。数组下标、字段写入、局部声明及 `.length` 继续使用已有类型/消费者合同。
4. 新事实、表达式和空括号沿既有深度、预算、取消及 `OriginSet` 传播；常量池引用、尺寸生产者和最终创建 BCI 均保持可追溯。若延迟尺寸生产者尚未由共享顺序机制安全呈现，本项拒绝对应方法，不能为了通过测试重复调用或把其语句提前。
5. 不新增依赖。JVMS 已给出全部需要的 rank/count，现有 descriptor 解析、AST 与 emitter 足够。长期成本集中在保持一个形状不变量与相邻测试，不引入额外解析/类型框架。

依据：[JVMS `anewarray`](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.anewarray)、[`multianewarray`](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.multianewarray)、[数组类常量池名称](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-4.html#jvms-4.4.1)。

## Risks / Trade-offs

- 把组件 rank 当结果 rank 会少一个 `[]`；分别固定 `anewarray [I`、`anewarray [Ljava/lang/String;` 和 `multianewarray [[[I,2`，并保留完整维度对照。
- 只修正文字符串而遗漏类型事实会令局部、字段或下标误恢复；同一 rank 在 AST 呈现与 `array_of_value` 必须一致。
- 负尺寸与效果函数的先后顺序是运行语义；比对异常类别/身份、trace 与零尺寸形状，不以 javac 通过代替执行对照。
