## Context

冻结 `MixedBooleanArgument.call(Z)V` 的 BCI 1/7/13 是同一闭合前向测试图，16/20 生产 `1/0`，21 `invokestatic sink:(Z)V` 唯一读取栈 Phi，24 `return`。当前方法级所有权校验整段引用，故 Jarde 的完整类没有调用 sink 的语义。已有 `ShortCircuitValue` 已对字段消费者恢复、对直接 `ireturn Z` 另案扩展；调用方差别是 Methodref 参数绑定和调用指令本身的效果。

## Goals / Non-Goals

**Goals:** 最小的单参数静态 void 调用消费；保持短路 RHS、目标调用各自准确次数和顺序；唯一 owner、完整来源与预算/取消；既有字段/返回恢复不回退。

**Non-Goals:** 多参数求值重排、实例接收者、动态调用、重载猜测、跨方法内联或通用值图重写。将来扩到这些形状须逐个给真实 Methodref/SSA/异常边证据，不由本正例自动开放。

## Decisions

1. **沿原图只扩消费者。** Region 候选仍从测试真实 taken/fallthrough 收集，逐节点查精确正常前驱、异常边及独立效果；producer 的唯一正常后继必须同指一个调用 consumer。单参数静态 void Methodref 是这个 change 唯一新的消费锚点；已有字段/返回锚点不变。
2. **Methodref 与 SSA 精确绑定。** 调用 opcode 为 `invokestatic`，真实目标描述符恰为 `(Z)V`，无接收者和其它参数；consumer 第一条指令恰读同槽 Phi 一次，Phi 只有这个 use，两个 producer 精确为 `1/0`。`invokevirtual`、额外参数、未知目标、引用传入或返回值被消费的路径均拒绝，不从 Java 目标名字推断重载。
3. **复用惰性值和调用发射。** 沿测试边建立既有 `Conditional`，在真实 `Z` 参数位置经现有布尔适配，然后发射一次 `Call` 表达式语句；调用后缀 `return` 保持原位置。测试、producer、goto、call、return 的 source map 均保留。共享后继的互斥文本复制仍受深度/展开和输出预算。
4. **拒绝不得变伪成功。** 完整回退保持额外入口、异常边、双消费者、独立效果与预算停止；Jarde baseline 八行都没有 sink 调用，不能只要求输出可编译。字段 16 路径、直接返回八路径及纯两/三测试均须同跑。

## Risks / Trade-offs

- [调用参数类型或顺序改变真实 Methodref] → 限定唯一 `(Z)V` 静态参数，并查 SSA 唯一 use 和绑定；以后扩多参数另证。
- [复制共享后继引起额外 RHS 调用] → 互斥路径与图闭合证明，再用 b/c/sink 三个计数逐路径复验。
- [调用 consumer 还有后缀效果] → 只在第一条 call 证明成功时消费其值，后续由已有顺序写出；未证明则整段引用。
