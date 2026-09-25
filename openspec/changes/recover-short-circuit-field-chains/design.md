## Context

见 [proposal.md](proposal.md) 和 [冻结三方执行](../../evidence/java-syntax-2026-09-25/short-circuit-chain-shared-true/analysis.md)。`ChainOrField.assign(ZZ)V` 的正常路径为 BCI 1、5 两次 `ifne 14`，BCI 11 `ifeq 18`，BCI 14/18 供给 1/0，BCI 19 唯一 `putstatic`。BCI 14 有三个入边。现有 `Region::ShortCircuitValue` 固定 `outer_branch`/`inner_branch`，因此在第三次测试前退回普通嵌套 `If`；BCI 19 被重访并误诊为循环，BCI 14/15/18 不在完整 source map。JADX 的 `IfRegionMaker.mergeIfInfo` 按条件继续路径合并 OR/AND，是候选遍历顺序的线索；它没有替代 Jarde 对物理边、栈 Phi 与唯一写入的证明。

## Goals / Non-Goals

**Goals:** 对同极性、正常控制流、两生产者汇入一次静态 `Z` 字段写入的短路测试链，一次取得物理所有权；证明时保留逐级惰性求值与最低位字段存储，不可证明时完整引用所有 BCI。两次测试的现有 `&&`/`||` 成功路径和异常边拒绝必须继续成立。

**Non-Goals:** 不识别唯一原始源码拼写，不发射 `&&`/`||` 新 AST，不把混合极性、异常保护内的字段写入、实例字段、多消费 Phi 或任意布尔 CFG 归入正例。三次以上测试不因超出表达式深度/预算就默许不完整普通 `If`；若不能整体引用须明确停止整个恢复。

## Decisions

1. **改造已有短路 Region，而非并列新节点。** 用按执行顺序排列的测试块与分支 BCI 列表替代固定外/内两个字段，保留两个生产者、消费块、前缀与拒绝理由。Region 构造从第一测试的顺序边逐个跟进：之前每个测试的跳转边必须到同一个共享生产者，最后一个测试的两个出口恰到共享与另一生产者；两个生产者均唯一进入一个静态写入消费块。每个测试的前驱只能是前一个测试的顺序边，共享生产者的前驱集合必须等于各短路出口，消费块只能来自两个生产者。任一额外正常入口、回边、异常边或作用域越界都不用于正例证明。对局部探测逐块 poll/charge，维护已见节点，避免引入单独的 CFG 重写 pass。旧固定两次测试的 Region 和 proof 正好是列表长度 2 的特例。
2. **所有权早于值证明，且拒绝必须原子。** 有限共享生产者图先在普通 `If` 前整体认领。生产者即使不是精确 1/0、消费块含 `dup` 或其它字段写入、测试有额外效果，也先保留这整个局部供 builder 检验；不让失败形状回到会重访消费点的嵌套 `If`。Region 的引用按每块 SSA 指令枚举全部 BCI，不只列块起点；`putstatic` 与同块后缀必须在引用和 source map。诊断记录首尾测试及消费点，完整测试列表保留在 Region。异常保护 frame 沿用已有限定：只在同一可证保护区且走到边界时取得拒绝所有权，不能因为链扩展越过 handler 或 body tail。
3. **泛化同一 SSA/字段证明。** 从解码 target 和 canonical 正常边推得共享生产者的布尔极性，不按 opcode 名称或地址猜测。按列表逐个核对测试指令、局部依赖与无独立效果；核对 producer 恰为 1/0、相同栈深、两个前驱出口对应唯一栈 Phi、Phi 仅由目标 `putstatic Z` 读取，并拒绝成员上的任意真实异常边。预收费覆盖所读边、指令与 Phi，采用现有 `MAX_VALUE_DEPTH` 限制最终表达式嵌套；超过可呈现深度仍整体引用，不发布部分计划。证明对象只保留测试 BCI 列表、共享极性、两个值、Phi、消费和字段身份。
4. **反向折叠已有 `Conditional`，保持延迟求值。** 从最后一次测试的 `fallthrough ? 1 : 0` 开始，按逆序将前层测试包成 `fallthrough ? inner : sharedConstant`。OR 的短路出口是 1，AND 是 0；`test_expr(bci,false)` 已表示顺序边条件。字段消费处沿用现有整数到 `Z` 的最低位转换，赋值仅在原消费点发射一次；来源合并所有测试、生产者、`putstatic`，后缀原顺序继续。建表达式或字段语句任一步失败，丢弃整个 staged plan 并整体引用。
5. **以完整类和负例验证算法，而非以 JADX 文本裁决。** 冻结类原/JADX/Jarde 各自 Java 8 重编，JVM 验证下对比四条值/调用路径；检验 source map 的 BCI 0/1/4/5/8/11/14/15/18/19/22。另以不规则 producer、第二消费者、独立效果、额外入口及真实异常处理区约束拒绝；两次测试 Base、shared-true、左假和非规范 `Z` 控制复跑。JADX 此正例输出 `z || z2 || rhs()` 且执行正确，但已知同方法双分配点及跨类匿名内联的合法反例改变类身份，因此只参考其合并顺序，不照搬启发式。

## Risks / Trade-offs

- [多测试共享生产者的前驱被宽松接受] → 按物理 predecessor 集合逐一比对，任何未知入口或重复出口拒绝。
- [链的前序 RHS 被提前执行或重复执行] → 以条件顺序边逐层嵌套 `Conditional`，四条 JVM 路径逐项检查 `calls`。
- [探测超过预算或表达式深度后留下半个 Region] → 探测前/逐步收费并只在全图闭合后提交所有权；建值计划失败时统一引用整条链。
- [改动双测试内部表示导致既有正例/异常拒绝回退] → 专项回归覆盖 shared false、shared true、protected exception 和 complete quote，并重跑 class-source/CLI 邻近测试。
