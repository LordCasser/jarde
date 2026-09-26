## Context

见 [proposal.md](proposal.md)。冻结 Java 8 样本的 `nested(I)I` 在 BCI 3/9 测试，BCI 12/16/20 各推一个整数，BCI 21 汇合后存入局部；`nestedEffects` 还区分测试与叶子效果和异常。原 class、JADX 输出重编后的 runner 完全一致，当前 jarde 将嵌套 `if` 的值留为未解析栈入口，质量为 fallback、语法为 not_java。物理证据在 `../../evidence/java-syntax-2026-09-26/nested-conditional-value/`。

当前 `Region::If` 可嵌套；`build::prove_conditional_value_with_forward` 只接受两个非空 `Region::Straight` 臂、两个入边和两个 Phi 输入，`prepare_conditional_region` 因而无法折叠内层 `If` 与外层共享的三叶子汇合。`ExprKind::Conditional`、类型/调用目标处理、来源集合、预算及停止渠道已存在。解析 class、Java 8 方言/descriptor 校验、运行时选择与验证均不负责源语法投影；此 change 仅在已验证输入的恢复层增加证明。

## Goals / Non-Goals

**Goals:** 对共同最终 join 的、由现有 `If`/非空直线叶子构成的有界条件树，整体证明路径、栈值、消费与表达式覆盖；复用已存在的条件值 AST、类型及发射规则。覆盖外层真臂或假臂继续分叉，并在预算允许的深度内继续递归。

**Non-Goals:** 不用源码行号决定语义，不改 SSA/CFG 或 Region，不加入通用 Phi 求解器；不把中途先汇合、再运算、再流向另一个汇合的树归入本证明（这需要独立的多汇合值组合分析）；不在本项处理循环、switch、异常处理器、外部跳入、不能折叠的独立语句或未证实的类型提升。

## Decisions

1. **复用现有事实和语法节点。** 从现有 `Region::If` 根递归遍历两个臂，每个内部节点保留自身 `branch`/`branch_bci` 的真/假边，叶子只接受 `Straight`。读取 canonical CFG 闭包与真实 branch target，确认内部边、叶子到共同 join 的每个正常入边一一对应，没有外部/异常入边、重复所有权或回边。拒绝不在这个形状内的 `Sequence`、`Fallback`、`Try` 等。所需的是恢复层私有的证明结果，不加新 Region、AST 或公开 IR。备选是把多前驱 Phi 直接按 BCI 排序配叶子；它不能证明分支极性和外部入口，故不用。
2. **只读、整树的 SSA 值证明。** 在共同 join 找到一个非平凡栈 Phi；按每条叶子的 exit slot 值与 Phi 输入一一对应，而不是假设输入列表顺序等于源码顺序。确认 Phi 无替换、没有第二个变化栈值、每个叶子值定义位于自己的叶子、只作为该 Phi 的输入使用，且 Phi 被 join 的一个真实指令唯一消费。沿用两臂 proof 对身份传递、消费者位置及同值 Phi 的保守界限。预算与取消在图、Phi、值遍历每步计费；任何不确定即拒绝，整个证明不改 SSA。备选是直接移植 JADX 的逐层 Phi 输入消去：本地 JADX `TernaryMod.makeTernaryInsn` 确认两臂结果唯一用于同一 Phi，遇到超过两输入时移除其中一个并把内部三元结果写回，后续 visitor 迭代外层。这启发了“内层值和外层汇合须同证”，但会修改 JADX 的可变 IR；jarde 的只读 SSA、物理来源与原子回退要求更适合整体证明。
3. **表达式与计划原子发布。** 从叶到根构造已有 `ExprKind::Conditional`；每个测试复用 `test_expr`、`test_effects` 与现有来源锚，每个叶值复用 `conditional_dependencies`/`render_value`，检查其 `Straight` 块内除被表达式覆盖的依赖与结构性转移外没有独立指令。逐层复用现有条件值类型、消费处转换与调用 Methodref/cast 规则，不额外推断类型。只有整树建成且来源完整时，才写入最终 Phi 的 `conditional_values` 和根分支折叠计划；根折叠后跳过对子节点的独立发布与语句发射。整树值构造失败时根的完整 `region_quote` 保留全部真实 BCI，已准备好的局部结果不得泄漏为可执行半成品；若控制流形状本身不构成候选树，仍由现有区域规则独立判断子区域是否可证明。备选是沿旧流程先折叠内层再试外层，容易留下不完整的空 `if` 与无定义消费者。
4. **不用生产期外部依赖。** 本地 JADX 是 Apache-2.0 项目，但其 Java visitor 绑定 JADX 的可变 Insn/Phi 模型、line hints 与遍历调度；直接复制或引入依赖会增加维护和集成成本，且 line hints 并非本产品的充分语义证据。借鉴其共同 Phi/自内向外思路，在现有 Rust 层写最小证明，无新增 crate。先用冻结 fixture 对比源码、JADX 和 jarde，再用拒绝样本检查输出质量与来源。

## Risks / Trade-offs

- [分支极性或叶子到 Phi 配对错误] → 用 decoded taken target、canonical exit slot 和唯一匹配双向核对，正例覆盖真臂/假臂再分叉。
- [测试或生产者被重复执行，异常顺序变化] → 对每个节点检查独立指令与使用次数，整树一次发布；runner 区分外/内测试、三个叶子与各自异常。
- [Java `?:` 的类型/重载解析偏离字节码] → 仅放行现有类型门可证明的表达式及真实消费目标；复杂数值提升、引用合并、多个消费者拒绝。
- [深树导致扫描或表达式递归失控] → 每步沿现有预算计费，遵守值深度限制并轮询取消；不得把限制简单放大。
- [拒绝时来源丢失] → 对根完整区域引用原始 BCI，核对默认/完整证据正文和 source map；不能把 `quality` 或 `execution` 的完成误读为 Java 可编译。

## Migration Plan

不改变持久格式或公开 API。实现和回归通过后按现有构建发布；回退该恢复层修改即可重新使用原保守回退。
