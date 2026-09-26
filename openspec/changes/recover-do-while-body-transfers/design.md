## Context

已有 `Region::Loop { form: DoWhile }`、`StmtKind::DoWhile` 与统一 emitter。`region::latch_tested_loop` 要求一个闩锁，但当前仅因头块末尾有比较就把体内 `if` 判为第二测试；`withContinue` 因此报 `LoopShape`。`withBreak` 的头块另有经 `goto` 到出口的路径，天然环集合不含该单向路径；`header_tested_loop` 先把头块误判为头测循环并因头块中的 `iinc` 拒绝，体框架也会丢掉向环外走的分支。这里需要补足**区域归属**，不能改写字节码或放宽任意 `goto`。

冻结输入见 `../../evidence/java-syntax-2026-09-22/do-while/`：498 B `DoWhileCore` 的7项原 class/JADX一致，jarde完整源码两处缺返回；702 B混合类的9项同样只有 `withContinue` 与 `withBreak` 编译失败；532 B正面类普通循环及带调用的闩锁条件三方编译执行一致。root 用同一冻结 CLI 独立重放脚本，状态和哈希一致。JADX 在本案的完整源码已编译执行，可作额外对照；原 class 始终是运行判据。

## Goals / Non-Goals

**Goals:** 只证明本层、单闩锁 `do-while` 的体内汇合与到唯一出口的提前退出；让本案完整类可运行，保存精确条件次数与来源。

**Non-Goals:** 不恢复原源码是否用 `continue` 的词面选择，不推断 `for`，不处理多层标记 `break`/`continue`、`switch` 内的 `break`、异常/不可约边或任意环外目标。父变更 `present-proved-java-structure` 的宽合同仍按独立任务推进。

## JADX 算法对照

本地 JADX `LoopRegionMaker.process` 先从 loop 的 exit node 挑 IF 条件，按 `isConditionAtEnd` 决定是否从 loop start 重建体区域，再扫描其它 exit edge 插入 `break`；`insertContinue` 则从 loop end 的 synthetic 前驱补 `continue`。这说明闩锁/体内分支应先按控制边角色区分，且体区域必须包含提前退出的边。Jarde 已有 `Region::Loop`、`LoopBreak/LoopContinue`、`Frame::loop_body` 和来源锚点，可以在现有区域证明中处理，无须移植 JADX 的指令改写流程。

JADX 的 `insertLoopBreak` 会沿空路径和路径交叉点寻找插入位置，再把新 `BREAK` 指令附在边上；其 `canInsertContinue` 主要检查 synthetic 前驱和支配关系。它们适合作为候选边搜索的参考，但单凭这些条件不足以证明目标是本层唯一出口、桥无效果、来源没有重复归属，也不足以区分嵌套 `switch` 截获的无标记 `break`。本变更以原 CFG/SSA、路径所有权和真实 BCI 完整覆盖为准，不能只因为 JADX 生成了可编译文本就放宽准入。参见本地 `jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/LoopRegionMaker.java` 的 `process`、`makeLoopRegion`、`insertLoopBreak`、`insertContinue`。

## Decisions

1. 区分循环测试与体内比较靠图上的**边角色**，不靠“头块含比较”这一语法迹象。只有头块的比较本身确实分流到循环外时才是头测候选；两条路径均留在体内、最终汇到唯一闩锁时，它就是体内 `if`。复用当前 `Region::If`、`Frame` 的边界/汇合和 `DoWhile` 语句；到闩锁的无效果转移可由结构化单臂 `if` 表达，无须为了还原 `continue` 字眼增加 AST。
2. 到出口的早退只承认显式的普通转移桥：它由体内分支独占，终点恰是本层 loop 的已证明唯一出口，桥上没有未认领的效果、异常边或外部入边。把这个桥作为**当前 loop 的臂**认领，不让天然环集合外的节点落入未覆盖扫描，也不把它当成另一个循环的出口。因为该路径真正跳过闩锁及余下语句，现有 `if` 或 `switch` 的空臂不能表达它：为这一个已证明转移增加最小的区域转移事实和 `break` 语句形状，继续复用现有 emitter/来源。若代码审读发现已有等价转移事实可复用，以复用为先。
3. 先判定唯一闩锁、唯一出口和体内桥的所有权，再决定 `header_tested_loop`/`latch_tested_loop`，避免头块中的 `iinc` 被错误送入测试纯度检查。各分支、桥、闩锁和出口各认领一次；`break` 锚实际 `goto`，`if` 锚实际比较，原值生产者经现有来源并入。失败时用现有 loop fallback 名出所有活块，不能用空方法伪装成功。
4. AST 构造只消费已证明的区域事实；不从反编译文本或 JADX 推断控制边。已有预算、递归深度、取消与默认/all 产物合同覆盖新节点；转移桥的遍历和来源计费与既有区域同级。JVM 图、SSA、reader 不增新机制或外部依赖。

## Risks / Trade-offs

- 把到闩锁的路径写成无条件 `continue` 可能跳过原本仍要执行的体语句；先证明两路径的汇合及相对顺序，允许写成倒置的单臂 `if`。
- 天然环不含退出桥，若只修改打印器会漏认领桥或把出口语句写两次；以区域所有权与实际 BCI 覆盖作为测试，而非只查关键词。
- 在 `switch` 臂中写无标记 `break` 会被 `switch` 截获；本项明确拒绝这种嵌套，父变更的带标记控制任务处理。
- 带调用的闩锁正例已健康；任何纯度放宽都可能移动调用。保持现有测试证明，仅改头块比较的边角色判定。
