# Switch2 后续 condition 表示：本地 JADX 源码核对

> 只读架构笔记。对象是本地生产 checkout `/Users/lordcasser/workspace/testzone/jadx` 中的 `IfRegionMaker.java`，不是官方当前 upstream。读取时 HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，`git status --short` 只有 `?? .atlas/`。任务给出的冻结 SHA `2fb1b16386941660fda07e9017285aec40fcb37e` 在此 checkout 无法解析，所以这版只能按实际本地文件作源码事实引用，不能声称已与 `...c37e` 逐行比较。

Switch2 的 direct-target join 候选证明已经通过；完整恢复随后在 `ArmsDoNotMeet@78` 停止。这一步不应再回到“继续找一个 join”上。现在可见的困难是：candidate proof 与普通 region/条件合并各自持有局部结构事实，遇到后续四个测试共享 payload 和物理 return 时，没有一个统一的 condition/ownership 事实对象供两边继续闭合。

## 0x00 当前 trace

实际诊断记录给出的关键分支与边为：

```text
branch98:  ipdom=136, then=111, else=118, forward_else=true
branch111: ipdom=136, then=118, else=131, forward_then=true, frame_boundary=118

98  -> 111 / 118
111 -> 118 / 131
118 -> 124 / 136
124 -> 131 / 136
131 -> 136
```

四个测试节点最终收敛到同一个 payload `131` 和物理 return `136`。这里有两层共享：`118` 是 `98` 与 `111` 的共同目标；`131` 与 `136` 又分别是多条后续路径的共同输出。共享目标不等于重复执行，打印成条件表达式时也不能复制 payload 或 return 语句来假装每条路径都有独立 owner。

从纯控制条件看，这张图可以写成一个带短路语义的条件树。按 `then/else` 的实际真假含义调整谓词方向后，成功路径大致是 `98.then && 111.else` 或 `98.else && 118.then && 124.then`；其余路径落到 `136`。这只是解释图的等价条件，不是建议把 BCI 或推测出来的布尔式写死到实现里。具体谓词翻转和求值顺序仍须以 decoded `IfCondition` 为准。

## 0x01 本地 `IfRegionMaker` 能做什么

本地 [`IfRegionMaker.java`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/maker/IfRegionMaker.java) 的处理顺序是：`process` 先调用 `mergeNestedIfNodes`，再由 `restructureIf` 定 out block，随后 `confirmMerge` 标记已归属块，最后创建一个 `IfRegion` 并分别调用 `makeRegion` 构造 then/else 两个子 region（60–109 行）。所以 `IfRegion` 的最终形状仍是树：一个 condition、then 子 region、else 子 region 和一个 out block；它不会原生保存两条分支共同引用一个 region 节点的 DAG 边。

不过，“树形 region”不等于“不能打印复合短路条件”。`mergeNestedIfNodes` 可以沿当前 then 或 else 路径找下一条件，按需要反转，并在复杂路径上递归合并；成功时由 `mergeIfInfo` 用 AND/OR 组合 `IfCondition`（398–477、528–545 行）。它还会要求未沿用分支的输出路径对齐：follow-then 比较两个 else，follow-else 比较两个 then（516–522 行）。`getBranchBlock` 对相同 block、等价 return、重复 block path、path-cross 和 empty path 有专门处理；最后 `confirmMerge` 把合并测试及跳过块标为已加入 region（547–596 行）。

在这张图上，局部合并并非显然不可能：`98` 可沿 then 到 `111`，二者反向对齐后共同输出 `131/118`；接着 `118` 可沿 then 到 `124`，`118` 与 `124` 的 else 都是 `136`，合并后输出 `131/136`。此时外层分支的 then 已是 `131`，与子条件的 then 对齐，满足 `canMerge` 的对应侧条件。`canSelectNext` 也允许读取有多个前驱的 `118`，但要求这些前驱全部已经进入 `mergedBlocks`（605–609 行）。因此不能只凭四个节点或共享汇合就断言现有算法必定失败；能否完成仍受方向反转、frame/scope 边界、路径清洁、block 标记时序和真实 decoded 节点影响。

这也解释了 `ArmsDoNotMeet@78` 的证据边界：candidate join proof 通过，只说明 switch continuation 的目标及其 incoming ownership 检查闭合；完整恢复在后续 region 构造遇到 arms 不会合。trace 证明失败位置在 `78`，却没有单独证明原因就是缺少 DAG。需要把“本地嵌套条件折叠是否成功”和“折叠后的条件/输出归属是否能交给 switch 恢复继续消费”拆开看。

## 0x02 现有专项形状的边界

`IfRegion` 加 `IfCondition` 足以表达普通短路树及两个输出 region；若 `131` 作为 then，`136` 作为 else，条件树成功形成，那么 payload 与 return 可以各在一个物理位置拥有，而不是为四个测试复制四份语句。真正需要验证的是当前 pipeline 能否让该唯一 owner 穿过 `78` 的 switch arms 证明边界。

现有专项表示并不能替代这个共同事实：

- `SharedTailEarlyReturn` 是双测试、一个物理 early return 加外置 shared tail 的窄形状；四测试和中间共享节点 `118/131` 不由这个形状完整描述。
- `TwoExitReturn` 约束在 boolean 方法的两个 terminal return outcome；当前 Switch2 是 `I)V`，图里是共享 payload 后到一个物理 return，形状不匹配。
- `ShortCircuitValue` / builder 处理有界的测试、gateway、`1/0` 生产者、唯一栈深 SSA phi、boolean consumer 与可选 tail continuation。Switch2 是普通语句 payload 的控制流，不能把 value-producing 条件图当作它的 ownership 证明。

以上边界说明需要统一 facts，但不能据此宣称“必须新建一套通用 condition DAG”已经是唯一或最小方案。当前 `IfCondition` 本身已经是条件树；局部折叠可能足够。缺口更准确地说是缺少跨 continuation proof 与 region recovery 共享的、可核验的 condition facts/representation：它至少要保留 canonical test identity、每条已证明边的方向、condition 的短路求值顺序、输出 arm 的唯一物理 owner，以及 scope/frame 限制。有没有必要把这组 facts 具体实现为显式 DAG，还是可扩充现有 `IfInfo`/`IfCondition` proof contract，要由独立设计和真实 trace 验证决定。

## 0x03 下一步设计边界

若独立 OpenSpec 继续做此问题，建议先定义 proof facts，而不是先选容器或引入另一轮 join 搜索：

1. 输入只来自已 decode、已 canonicalize 的 block/test 及其实际 successor；不以 BCI 排序制造路径。
2. 每个测试的两个输出保留原方向与短路顺序；任何条件反转都需显式记录，避免重排带副作用的表达式。
3. 共享节点可以有多个已证明 incoming edge，但语句/terminal 仍只有一个 physical owner；region emitter 不得按 arm 复制。
4. continuation candidate proof、condition folding 和 `Region` ownership 对同一 canonical identity 集合工作；若 scope/frame boundary 阻止合并，就保留边界并拒绝整个可选重建，不输出 partial tree。
5. 用真实 Switch2 做两组实测：本地 `mergeNestedIfNodes` 是否能从 `98` 收到 `131/136`；即使能，`ArmsDoNotMeet@78` 是否仍存在。前者失败说明条件合并限制，后者仍失败说明跨阶段 proof/ownership contract 不足。

待完善：拿到可解析的冻结 `...c37e` SHA 后，与当前 HEAD 的同一文件作精确差异对照；再结合失败 `@78` 周围的调用者与 region ownership 检查，确认缺口是 facts 没有传递还是现有 facts 本身不够。现在不据单条诊断把“统一 representation”收窄成已证实的最小代码改动。
