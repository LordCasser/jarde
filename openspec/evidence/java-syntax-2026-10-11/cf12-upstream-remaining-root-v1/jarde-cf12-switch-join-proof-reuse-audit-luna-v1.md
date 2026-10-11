# Switch2 join 发现能否复用 switch fallthrough DAG 证明

## 结论

可以复用现有 `prove_switch_fallthroughs` 的 DAG 遍历和大部分 arm 闭合/ownership 证书，不需要再实现第二套三色 DFS，也不需要新增 `Region`。但当前函数**不能原样当作完整 join 证书**：它把指定 join 当作闭合边界，返回的只有 case-to-case `fallthroughs`，没有返回“至少一个 arm 真正到达 join”“join 的全部 incoming rows 来自 switch dispatch/已证明 arms”或“闭合中确有内部 return”的证据。最小路线是在原遍历中保留这些 outcome 证据，并在 join 候选选择层进行额外核验。

对 Switch2，候选 164 应从 target map 移除并作为已知 `join` 传入：`groups` 仍包含目标为 164 的合并 case group（case 3/4/default），而 `targets` 仅包含 48、56、71、137。随后要求原证明完整成功且 `fallthroughs` 为 empty；即 `Some(empty)` 表示扫描闭合且没有 case-entry 之间的落入。此例的 case 2/5 内部 branch 都是正常比较，最终分别可到 continuation 164 或到显式 return 136/163，正是该 helper 当前 DFS 可走的 DAG 与 terminal-leaf 形状。

## 现有契约实际证明了什么

`Walker::switch_region` 先把 decode 命名的 keys/default 按 BCI target 合并成 group。候选 target 164 若作为 `join_bci`，它对应的 group 会自然从 `targets` map 中排除；`prove_switch_fallthroughs` 自己要求：不等于 `join_bci` 的实际 groups 数必须等于 `targets.len()`。这使“join 也是一个直接 case target”的输入形态与现有 map 契约兼容。外层 `switch_region` 在调用前也已核对每个 decode target 都是 branch 的真实 successor，故不能绕开该 target/decode 对齐检查。

helper 随后：

- 从完整 canonical edge stream 构造 incoming/outgoing rows；逐 row 计 `AnalysisSteps` 并 poll 取消。
- 核对 dispatch 的 canonical outgoing rows 与 NormalFlow successors 完全一致、没有重复边，且只能是 Normal edge。
- 对 map 中每个非 join target 做三色有限图扫描。到已知 join 时停止；到其他 case entry 时记录 case exit；环、非 Normal outgoing、canonical/NormalFlow successor 多重集不符、非比较多路分支、未以 Return/Throw 结束的死叶都拒绝。每次 node/edge probe 都继续向共享 `Budget` 计费/响应取消。
- 收集每个 case closure，并拒绝 closure block 多 owner；核对 case-entry 的 incoming 只能来自 dispatch 或一个顺序更早、且其完整 exit 集确实落在此 entry 的 owner；再核对 closure 内各节点 incoming 的完整 canonical rows。
- 最后返回 `Some(fallthroughs)` 或拒绝 `None`。`Some(empty)` 是完整扫描成功但无 case-to-case fallthrough；它不是“目标没有后继”的空结果。

这比 `switch_forward_join` 现有的线性 route 扫描更接近 Switch2 的实际控制流：它已接受比较节点的分叉，也已接受显式 Return/Throw 终端叶，且不在 `visited` 上做破坏性试走。

## 现有契约还缺的候选证据

当前 `prove_switch_fallthroughs` 在每个 node 的 DFS 中先检查 `Some(node) == join` 并立即停止。因此：

1. 到达 164 的 closure edge 不进入 `case_exits`。这是把 164 视作 switch 后 continuation 的正确语义，不应把它误判为普通 case fallthrough；但函数也就没有说明哪些 case closure 是 164 的实际 incoming source。
2. candidate join 本身不在 `entries` 或任何 closure 中，后段对 closure 和 case entries 的 incoming 检查不会检查 `incoming[164]`。Canonical dispatch 与 normal-view 对齐能说明 dispatch 有一条至 164 的边，却不能说明非 dispatch 入边无外来 owner、无 Exception/其他 edge kind、没有重复/视图遗漏。
3. 无后继的 DFS 叶会验证是 Return/Throw，但 terminal 类型/是否出现内部终止叶不进入返回值。`Some(empty)` 本身因此不足以证明“有一个 join continuation 和至少一条 early-return path”。本例要求的 Return 叶在 BCI 136、163，必须有显式 witness。
4. helper 只判断调用者给定的 join；它不会证明候选唯一。若调用者逐个测试直接 target 候选，两个不同候选都闭合时，不能任取 BCI 较小者或第一个。

`forward_join_predecessors(branch, candidate)` 是可复用的 normal-view 边界检查：拒绝 loop header，要求有非 dispatch predecessor，并要求每个 normal predecessor 由 dispatch 支配且不受 join 支配，从而拒绝典型外部入边/回边。它不能替代上述 canonical incoming 精确核验，因为它只看 `NormalFlowView` 的 predecessors，不对被视图省略的 canonical edge kind 或行完整性作证明。

## 最小复用方案

优先只增强现有 helper 内部的一次扫描，不复制 DFS：

1. 对 branch 的直接 successor target set 预筛 join 候选。候选必须是实际 direct case target，且 `view.predecessors(candidate)` 有 dispatch 以外的 predecessor；这避免对所有只被 dispatch 进入的普通 case 起点重复扫描。候选 target 必须与恰一个已分组 case target BCI 对应。最终仍逐个候选验证，并要求唯一通过；如无或多于一个，拒绝。
2. 对一个候选，把它从 `targets` map 排除，保留 groups 完整，传 `join=Some(candidate)` 与 `join_bci=Some(candidate.bci())` 调用同一 `prove_switch_fallthroughs`。其余 case target/labels、dispatch 完整性、case fallthrough 顺序、每 arm DAG/闭包 ownership 都走现有实现。Switch2 要求结果是空 fallthrough map；以后若明确支持 case-to-case fallthrough，也只能接受 helper 实际证明的 map，并由现有相邻 label ordering 检查接受。
3. 在现有遍历中为每个 closure 记录两项轻证据：`has_terminal_return`（或 Return/Throw outcome bit）与“本 closure 中哪些 source 的 canonical Normal successor 是 join”。这些是 DFS 已访问结果，不需要再扫 closure。完整 incoming map 已在函数开头收集；在 helper 返回前按完整 rows 核对 join：rows 必须全为 Normal、无重复，且 source 集恰等于 `{dispatch} ∪ join_sources_from_closures`。另要求 `join_sources_from_closures` 非空，证明至少一个 case arm 的 continuation witness 真的到达 join；要求有 Return/Throw terminal witness（此 fixture 可进一步要求 `Return` witness）。candidate 必须也在 dispatch’s exact outgoing rows 内。所有这些数据在 proof 现有 map 与 traversal 中已可取得，边验证额外计费并 poll。
4. 将私有 helper 结果从裸 map 扩充为最小证书，例如 `{ fallthroughs, has_terminal_return, join_sources, join_rows }`，或等价地让 helper 内部在候选模式下完成 2/3 并继续只返回 map。不要新增通用图框架或可随处配置的 `Region`；唯一需要的是让候选调用者获得它目前丢掉的 outcome/incoming 事实。偏好把 join ownership 核验的开关/用途表达到具体私有候选路径中，避免悄悄加严现有所有 switch 调用的 join 规则，因为已有调用的 join 可能是 enclosing branch 共享的 boundary。
5. `switch_forward_join` 保留原调用点，但其候选寻找改为上述 target 候选与复用证书；选择成功的唯一 candidate 后，再执行现有 `switch_region` arm walk 与 `claimed` 不相交核验。join 164 不归任何 case closure 所有；case target==join 的 group 继续由现有 switch builder 表达为空 arm；BCI 164–181 continuation 留给外层 walker 正常恢复，不被当作 case body 或删除。

这里唯一建议的新私有结果信息是证明器已有 DFS 的产物，不是第二套遍历或新产品概念。若想完全不扩展返回值，也可让现有 helper 在候选调用中直接验证 join rows/terminal witnesses并仅返回 map；但当前函数面向所有 switch 调用，不能无条件加入这些更严格条件，因此需要一个精确的 candidate-use distinction。

## Switch2 上的手工契约匹配

保存的反汇编 `/private/tmp/jarde-cf12-remaining-render-root-v1/TestSwitch2.test/TestSwitch2$TestCls/javap.stdout.raw` 显示 switch BCI 5 的 direct targets 为 48、56、71、164、137。BCI 164 是 default 与 case 3/4 的共同目标；它同时是空 case group 与共享 continuation。case 48 的 goto 164、case 56 的 if/transfer 到 164、case 71 与 137 的比较 DAG 都能到 164；case 71 内部还有 terminal return 136，case 137 内部有 terminal return 163。所有 5 个 targets 中候选 map 应移除 164，remaining groups 四个（注意 56 代表 keys 1/6 合组），所以 group-count 与 target-map length 一致。

该候选的证书预期是：empty fallthrough map；non-empty `join_sources`（由多个 case-owned blocks 进入 164；具体 canonical block identities 应由运行时证书记录，不在这里静态硬编码）；canonical join rows 包含 switch dispatch 直接入边以及由闭合 arms 到 164 的 Normal edges；至少一条明确 Return terminal path（136 和 163）；候选 164 唯一通过。上述 BCI 集合是依据保存的 javap 边形状推导的 fixture sanity check，不作为生产条件。

若 candidate 164 有 `Exception`/`Subroutine` row、重复 canonical row、来自非 owner 的 edge，证明必须拒绝。若 case2/5 的分支不以 comparison 结束、return block 还有 normal successor、任一 arm 有 loop/backedge、unknown side exit、跨到非相邻 case target，沿用现有拒绝，不以 candidate join 为借口放松。若另一个 direct target 也满足 complete closure + join ownership，拒绝多 candidate。

## 结语与边界

复用是可行的：在传入已知 candidate join 的前提下，现有 `prove_switch_fallthroughs` 已证明大部分复杂 DAG 与逐 arm closure，足以替代重写三色扫描。它当前只输出 fallthrough map，且把 join incoming 与 terminal outcome 留在局部或未统计，所以要补 join source/rows 和 terminal witness，再以候选唯一性收口。扩展应保持同一 `Budget`、同一 cancellation polling、同一精确 NormalFlow/canonical 对齐与所有权规则；预算不够时照旧 Stop，不放大限额。

此为 private read-only 架构核查，没有修改 OpenSpec、源码或运行 Cargo/JDK/JADX/CLI/Git。
