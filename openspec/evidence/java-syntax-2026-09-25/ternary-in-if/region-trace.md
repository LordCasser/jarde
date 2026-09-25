# BCI 62 的 Region owner trace

## 输入与只读诊断方式

此追踪针对 `TernaryInIfProbe.bothMatch(Ljava/lang/String;)Z` 的冻结 class（见同目录 `javap.txt`）。没有改动共享生产代码：将当前工作树复制到 `/tmp/jarde-region-trace-20260925`，只在副本 `crates/jarde-java/src/region.rs` 的 completed-tree ownership 检查前打印 `Region` Debug 树和所有与 BCI 62 相邻的 canonical edge；另用已有的 `JRE_JOIN_PROBE` 打印每个条件分支的 join 选择。副本以默认私有 `target/` 构建，运行后清理了该 target 和临时副本。追踪不改变 Region 决策或输出。

## 物理 CFG

BCI 62 是含 `iconst_1; ireturn` 的终结基本块，canonical identity 为 `(62, [])`。该块的全部普通前驱是：

| 前驱 | 边 | 来自字节码 |
|---|---|---|
| `(45, [])` | `45 -> 62` | `goto 62` |
| `(48, [])` | `48 -> 62` | `ifeq 64` 的 fall-through |

BCI 62 没有普通后继，因为它执行 `ireturn`。另一个相关终结块 `(64, [])` 是 `iconst_0; ireturn`，从 BCI 38 的条件分支和 BCI 48 的条件分支都可达。BCI 45/48 是分别通向公共真返回 62 的两条路径；分支 38 与 48 的另一结果通向假返回 64。`javap.txt` 保留了完整指令与目标。

## completed Region tree 中的两个 owner

诊断打印的 `Region` 树中，第一次和第二次声明 `(62, [])` 的路径分别为：

1. `method[0] If(branch@0).then If(branch@7).then If(branch@31).then If(branch@38).then Straight[45, 62]`。
2. `method[0] If(branch@0).then If(branch@7).then If(branch@31).else If(branch@48).then Fallback[62]`。

第二处 fallback 的局部 reason 是 `Loop { block_bci: 62 }`：这不是物理循环，而是 branch 48 的 then arm 遇到了 walker 已访问过的 block。第一处把 62 收在从 45 进入的 straight run；随后 branch 48 的 arm 又尝试收同一个返回块。completed-tree 的 `overlapping_owner` 发现重访后，把整个 `bothMatch` Region 替换为 `OwnershipOverlap` quote。这解释了报告为何只显示全方法 fallback，而 builder 无法继续恢复。

## join 选择为何导致重访

`region_at_inner` 对该方法六个条件分支（BCI 0、7、31、38、48、17）的诊断都是 `ipdom=None`，且两个方向的 `forward_join` 都为 false。原因是各终结返回块在 `NormalFlowView` 的反向支配图中都通向虚拟 exit；算法会过滤虚拟 exit，因此这些分支没有一个**实际基本块**可当作唯一的 immediate post-dominator。两个返回结点 62、64 都是可能结果，不能选其中一个作单一 join。`frame.arm(join=None, ...)` 因而没有在 62/64 上设置边界，递归 arm walk 可一路进入各自终结块。

导致此具体重访的是 branch 38 与 branch 48：branch 38 的 `then` 路径 `45 -> 62`，而它的 `else` 路径进入 block 48；block 48 再分流到 62 或 64。当前 Region 以一条 `If` 的单个 `join` 和单个边界描述分支。对 branch 38，真实控制流是一个有两个终结出口的 DAG，并不存在满足现有 join 前提的单一块。把 62 或 64 任选为 join 会错误地将另一出口移出分支；只把 branch 48 的 fall-through 62 当 join 也不能让 branch 38 的 then arm 和 else arm 都在该点相遇。

因此单改现有 `region_at` 的 join tie-break 或边界规则不足以安全修复。可落地的最小边界是一个窄的、先证明再消费的条件图折叠：在 Region 所有权提交前识别由相同两个终结结果（本例 62/64）组成的两条判定路径，将判定表达式合为一个布尔条件，让共享的两个返回块各有一个 owner；证明失败时保留整个原有 quote/fallback。它需要表达“两个出口共享”的有界节点集合或一个明确的合并结果，不能将任意重访块从 owner 列表中去重。若不想增加 Region 形状，可由专门判定直接生成一个 owner 唯一的既有 `If`，但要在生成前证明被折叠的分支、两种极性、所有中间块和求值顺序；这不是把 `join` 指向 62/64 的局部修补。

## 与 JADX 的判定及拒绝边界

JADX 的 `IfRegionMaker.checkForTernaryInCondition`（本机源码 `jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/IfRegionMaker.java:479`）先分别找 then/else 路径上的下一 `If`，要求两者 `firstIfBlock` 的 dominance frontier 完全相同，再递归处理，并只在两组结果分支目标相同或严格互换时构造 `IfCondition.ternary`。相邻 `mergeIfInfo` 也比较分支路径，并对返回块有 `isEqualReturnBlocks` 特例。本冻结探针上的 JADX 输出没有 ternary：它保留早退式嵌套条件，Java 8 重编和八条 `-Xverify:all` 路径都与原 class 一致。由此没有证据支持仅凭两个 arms 各自可达相同返回块就照搬其合并；匹配 dominance frontier 是候选证据，还必须证明完整条件路径与出口的成对映射、每条边的极性以及没有中间副作用被移动或丢弃。

最小拒绝负例建议至少覆盖：

- 两条候选路径不共享完全相同的 true/false 终结目标（例如其中一支改为返回另一常量或落入第三个返回块）；
- 任一路径在分支间含有可能抛异常的调用、字段写入或其他可观察语句，合并会改变调用/求值次数或顺序；
- 两分支的出口对应关系相反或不能由 branch target 与 fall-through 精确证明，防止错误应用 De Morgan/反相；
- 返回块具有不同栈/返回类型，或入口/局部 SSA 状态无法证明一致；
- 存在异常边、循环回边、外部入口/额外 predecessor，或某个中间块被 CFG 外部路径共享；
- 分析预算耗尽或取消，必须回退到同一原始字节码 quote，不能保留半折叠 Region。

这是一项 Region 分支所有权/条件折叠候选，不是 Boolean conditional Phi 简化。对当前探针也不能说 JADX 有行为错误；它的等价早退输出在此例正确，只是没有恢复原始 `?:` 形式。
