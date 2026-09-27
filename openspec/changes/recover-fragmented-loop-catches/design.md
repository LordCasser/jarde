## Context

见 [proposal.md](proposal.md) 与 [CF-18 架构复核](../../evidence/java-syntax-2026-09-27/cf18-handler-region-triage/architecture.md)。缩小类的正常环是 `4→9→32→51→4`；内层 `NumberFormatException` handler 19 和外层 `IllegalStateException` handler 38 均以普通边到更新点 51，但 handler 只经异常边进入。异常表 `9..16→19`、`9..29→38`、`32..35→38`，其中内层 handler 19 位于外层首行的物理区间。完整类的外层 handler 72 由三条分段行共享。当前 `proved_loop_catch_joins` 对声明行全部块要求普通循环头支配，使异常根被误判为额外普通循环入口；`guard::nests` 随后又拒绝同 handler 多行。固定 JADX 的区域放置在完整类上改变行为，不能照搬其支配前沿包裹算法。

## Goals / Non-Goals

**Goals:** 在同一候选内证明异常表、正常 CFG、SSA、词法 try/handler 所有权与完整类行为；以固定完整类及缩小类验收；保护原子回退。

**Non-Goals:** 建立通用异常图重写、修复所有不可约循环、合并 finally/TWR、推断来源未知的编译器方言，或在没有证据时猜测 handler 对应的 Java 词法层。

## Decisions

1. **新证书只表示本受限形态，不新建通用 IR。** 现有 `(handler, join)` 集合只能解释 SCC 例外，不能把同 handler 的分段行与嵌套范围交给后续 Guard/Region 共同所有。需要一个有界、只读的候选证明记录：原异常行 ordinal/start/end/type/handler、每行实际 protected BCI、同 handler 组、内外词法边界、异常根到唯一循环更新点的关系，以及已验证的 block owner。它以现有 Canonical CFG/Code/SSA 为事实源，供不可约例外、catch 候选和 Region 归属复用；不复制 JADX 的可变块重排或设置全局 `DONT_GENERATE`。若已有 Guard Plan 能承载这些事实，应扩充该 Plan；否则新增私有证书，不引入公共模型。
2. **先证明真实异常派发，再允许循环例外。** 对每个可抛 BCI 按异常表原 ordinal 顺序比较候选内外 catch 的覆盖和优先级，同时核 Canonical CFG 的相应异常边；某段表行即使没有实际可抛指令也不能省略，仍须证明物理范围、正常路径和词法所有权。每个 handler 根须无普通前驱，所有有限正常完成路径须汇入同一个受证循环更新点，不能逃到其它未归属的环内块；缩小类的 `19/38→51` 是单后继子形态，完整类的内层 handler BCI 38 经 BCI 48 条件分支，一路 `52→85`、一路 `55..69→85`，外层 handler BCI 72 也到 85。内层 handler 必须在外层被保护体中且无外部入口；声明范围与 Canonical 行须完整映射，不得切过未处理的物理块或越出受证循环。只在这些证据闭合时才把异常根的回接排除出普通不可约判定。简单删边或整体放宽支配会掩盖真正的多入口循环，故不采用。
3. **分段同 handler 是一个外层 catch 的物理组。** 仅当各行相同类型及准确 handler、区间不交叉、表优先级不与内层 handler 互换、保护区与一个连续词法 try 的全部可抛操作等价，才将它们视为一条外层 clause；内层 handler 的正常正文仍是外层 try 的一部分。需要证明外层 catch 置于完整正常分支外围，尤其完整 fixture 的 `work(value)` 位于负值分支之后且仍受外层行保护。不能依据某一 row 的首尾把 catch 放进该分支。
4. **一次性认领 handler 与汇合。** Region 继续使用既有 Try/Loop/If AST，但根据同一证书让两个异常根分别归属唯一 CatchClause、循环更新点归属唯一续接 AST/source 节点；更新点可以有受证的多条普通前驱，不能重复输出。SSA 必须核 handler `astore` 的 caught value、累计值及循环索引从每个入口到后继读取的 reaching definitions。先在固定两类上保存 SSA/来源图，然后实施；若发现缺失值或独立局部作用域债务，先记具体阻碍并修订任务，不放宽局部声明为 `Object` 或输出不能编译的源码。源代码只在完整证明后于现有 Builder checkpoint 原子发布。
5. **验收原 class 优先。** 缩小样本原/JADX 皆 `4:110`，完整样本原 `124:115` 而 JADX 外层 catch 错位导致非零退出；Jarde 必须匹配原 class。使用 Java 8 源码构造正向变体，必要的真实多入口/错归属负例可由测试夹具的 JDK ASM 生成并以 `-Xverify:all` 先验证，不把无效字节码的拒绝算成果。无生产依赖。

## Risks / Trade-offs

- [同 handler 多行并不必然是同一个 Java catch] → 必须同时比较每个可抛指令的表顺序、类型及闭合词法体；未证明就拒绝。
- [handler 回接例外掩盖真实不可约普通 CFG] → 只豁免无普通前驱的受证异常根到唯一更新点，负例保留其它普通入口。
- [所有权通过但 SSA 局部声明失败] → 在阶段一固定实际 SSA 定义/使用，再局部修正现有声明计划；若所需机制超出本候选则停止并独立登记债务。
- [完整 fixture 比缩小样本多条件和 continue] → 两者都作为发布门槛，不以缩小样本的绿色结果替代完整测试。
