# CF12 两项剩余控制流缺口：只读设计稿

## 结论和证据等级

本稿只讨论 `TestSwitchWithFallThroughCase2.test(IZZ)Ljava/lang/String;` 的双层 join 延续，以及 `TestSwitch2.test(I)V` 含早退 case 的共享 continuation 发现。两者都仍是 Jarde 的安全拒绝；本文提出的是后续独立工作的最窄设计候选，不是当前未关闭 conditional 的实施内容。

父任务转来的真实重放结果是：完整 runtime replay 32 条命令、15 个比较项；JADX 的 `TestSwitchWithFallThroughCase2` 完整反编译类可编译，原 `check()` 和额外 68 个输入值矩阵均与原类相同，现存问题只是重复代码 warning。Jarde default/all 已完成的三整个类重放（Simple、Switch3、Switch4）与原类相同，包含此前讨论的 Switch4。保存的执行记录在 `/private/tmp/jarde-cf12-remaining-full-replay-root-v1/execution.json`，状态写作 `observed_complete_source_comparison_pending_independent_acceptance`；因此这是已观测运行证据，尚不是独立验收记录。`TestSwitch2` 和 `TestSwitchWithFallThroughCase2` 的 Jarde 输出仍带有未恢复标记，不能声称 Jarde 的这两个目标已运行通过。

下面的 BCI、JADX/Jarde 结果来自已保存 class/Javap 与完整源码产物。函数行为是对当前 `region.rs` 的静态阅读结论；由于本任务未加内部探针、未编译或运行，具体哪一个运行时拒绝点先触发，仍需 root 在后续独立验证中观测。

## 1. FallThroughCase2：switch join 175 之后的外层 continuation 197

### 字节码形状

留存的 class 反汇编为 `/private/tmp/jarde-cf12-remaining-render-root-v1/TestSwitchWithFallThroughCase2.test/TestSwitchWithFallThroughCase2$TestCls/javap.stdout.raw`。`test` 的主要控制点是：

- BCI 4 的 `ifle 197` 是外层 `if (a > 0)` 的条件，外层 join 为 BCI 197。
- BCI 11 是 `tableswitch`。case 1/2/3/default 分别进入 36/121/150/153。
- BCI 36 的 case 1：先追加 `>`。条件 `(a == 5 && b)` 成立后，`c` 的两条路径在 BCI 175 前分别追加 `1` 或 `!c`，随后离开 switch；条件不成立则落到 case 2 的 BCI 121。
- case 2、3、default 的路径也在 BCI 175 汇合。BCI 175–195 追加 `+`，然后落入外层 join BCI 197。BCI 197–225 是外层 if 之后基于 `b && c` 追加 `-` 的 continuation，最后 BCI 227–229 返回。

因此这里有两个不同的 join：switch 内部 join 175，以及包含“switch 后 `+`”的外层 if join 197。Java 的预期结构是 `if (a > 0) { switch (...) { ... } str += "+"; }`，不能把 175 当作外层 if 的终点，也不能把 175 之后的语句复制进每个 case。

### 当前 Walker 的静态路径推论

保存的 Jarde 输出 `/private/tmp/jarde-cf12-remaining-render-root-v1/TestSwitchWithFallThroughCase2.test/TestSwitchWithFallThroughCase2$TestCls/jarde-default.java` 在整个 `test` 方法上安全拒绝，理由为 `the arms of the branch in block 0 do not meet at one join`。当前 `Walker::region_at_inner` 遇到 switch 会直接返回 `switch_region(...)`，后者返回 `Region::Switch { join: Some(175) }` 和 `next=175`，不在该次调用中继续沿 175 走到外层 frame boundary 197。

外层 if 的 arm 处理看到的于是是 `arm_next=175`，而非它要求的 197。现有两个续接候选不能接上这个形状：`continue_early_return_arm` 只接一棵两臂均为 `Straight`、汇合到直接 `ireturn` 的 boolean-return `If`；`continue_prefixed_loop_arm` 只接单个 `Straight` 前缀后进入新鲜自然循环头、并证明循环唯一退出到当前 arm join 的结构。两者都没有“单个完整 Switch region 后接 switch join 的直线 continuation”的分支。若它们都返回 false，当前外层 if 路径就设置 `unclosed_tail_at` 并报告 `ArmsDoNotMeet`。

这是强源码证据，说明当前显式续接器没有覆盖 switch-to-outer-join 这一组合；但不能称为已实测的唯一责任。内部 switch 也可能先在 switch-region 的 arm ownership 或 fallthrough 证书处失败。后续第一步应仅加一次临时观测：确认 BCI 11 的 `switch_region` 是否成功返回 join 175，以及外层 if 的 arm 是否确实收到 `next=175`。不需要改 production API，也不应为了定因保留观测代码。

### 最窄扩展

优先沿用现有 `Region::Switch`、`Region::Straight` 和 `sequence_region`，不增加 Region 变体。一个小型 `continue_switch_tail_arm`（或将 `continue_switch_arm` 收窄后泛化）只接受如下形状：

1. 当前 arm run 恰为一个已结构化的 `Region::Switch`，其 `join` 等于返回的 `next`；该 join 不等于外层 if 的 boundary 197，且不落在外层 frame 的 stop/case-entry/scope 之外。
2. switch 的 case arm 已通过原有 fallthrough 证明，case block 集合互不重叠。加入 continuation 前，验证 canonical incoming rows：175 的每条进入路径都由该 switch 的已拥有 case closure 产生、均为 Normal，且不存在外部前驱；case 分支以 `SwitchBreak` 离开 175 的边仍由现有精确源/目标校验处理。
3. 在**外层 arm frame**（保留 boundary 197，不沿用 switch arm frame）从 175 再调用 `region_at`。针对首个最小闭环，可只接受一个 `Straight` tail，且 walk 恰好返回 `next=197`；tail 的新 visited 集必须与其 Region block 集完全一致、非空、不与 switch blocks 相交。tail 与 switch 的 canonical incoming/outgoing rows 都要满足路径和 frame 边界。这样产出的既有 sequence 是 `Switch(…, join=175) ; Straight(175..195)`，外层 if 再与空 else arm 汇合在 197。
4. 只有所有检查成功才发布组合。预检应先验证确定的边界与 edge ownership，避免 walk 后失败留下半消费的 `visited`。若 walk 本身 Stop，应保留原 Stop/BCI；若证明拒绝，必须恢复或完整引用已经 walk 的 tail 块，保持拒绝内容覆盖与现有“每块只有一个 owner”的不变量。

此任务不需要动 `build.rs::switch_join` 的表达式改写。该函数只消去“join block 是唯一 `return`”的 switch-expression 尾部，并要求各 arm 不 fall through、arms 是 join 的唯一入口。这里的 175 是普通语句 continuation，应该保留并在外层 if arm 中写一次；不能把它设为 `settled` 或消除它。现有 `SwitchBreak` 是无物理块的控制叶，builder 还会核对活跃 switch 身份、join、嵌套深度；新续接不应弱化这项验证。

### 反例边界

若 175 另有外部入口，或某个 case closure 的边进入另一个 case 却不符合现有相邻 fallthrough 证明，必须继续拒绝。若 175 到 197 的尾部有分叉、return、循环或另一段 switch，第一版直线-tail 规则应拒绝，避免把外层控制结构塞进 switch arm。若 walk 的 visited delta 含 tail 以外的块，也拒绝。尤其不能仅按 BCI 相邻、`immediate_post_dominator` 或源码看起来像 `str += "+"` 就认定二者是唯一 join。

## 2. Switch2：含内部 return case 的共享 continuation 164

### 字节码形状

正确 BCI 以留存的 `/private/tmp/jarde-cf12-remaining-render-root-v1/TestSwitch2.test/TestSwitch2$TestCls/javap.stdout.raw` 为准。`test(I)V` 在 BCI 5 执行 `tableswitch`，不同目标是 48、56、71、137 和 164；3/4/default 直接共享目标 164。case 0 的 48 更新字段后 goto 164；case 1/6 的 56 条件分支最终到 164；case 2 的 71 有多个条件分支，有些路径在 BCI 136 return，有些到 164；case 5 的 137 条件分支，一路到 164，另一路更新字段后在 BCI 163 return。BCI 164–181 是共享 continuation：若 `isScrolling && action == 1` 则更新字段，然后在 181 return。

`TestSwitch2` 的 Jarde 文本 `/private/tmp/jarde-cf12-remaining-render-root-v1/TestSwitch2.test/TestSwitch2$TestCls/jarde-default.java` 拒绝整个 `test(I)V`，指出 switch BCI 0 的两臂重复拥有 block（实际表 dispatch 在 BCI 5），并另列 17 个只可由 normal-flow 视图之外边抵达的 live blocks。这里不能因视图外 blocks 被列出就忽略 canonical exception/其他边；它们本身正是保守闭合检查要保留的信号。

### `switch_forward_join` 的精确适用边界

当前 `switch_forward_join` 在 immediate post-dominator 缺失时尝试补找 join。其注释和实现要求：至少三个不同 successor；直接 successor 若不是 terminal，就必须沿**单 successor 直线**走到候选；直接 terminal target 才计入 terminal 数；不允许 route 中途遇见另一个 switch target/case entry；通过所有线性 route 的首次公共节点作为候选；最后用 `forward_join_predecessors` 排除回边和外部 normal predecessor。

本例与这组边界直接冲突：

- 表中有 5 个不同目标，但 164 自己也在目标集合中。case 48 的单 successor 是 164，代码在 `targets.contains(next) && next != start` 处就拒绝该 route；candidate 搜索也排除所有 `targets` 中的节点。因此 164 不可能被当前“各线性路径公共内部节点”办法选中。
- 56、71、137 的首个 block 含条件分支，不是单 successor 线性路由；case 2/5 的 return 136/163 是内部 branch leaf，不是 switch 的直接 terminal target。当前 `terminal_targets` 统计不到它们，且分支 route 会因 `successors.len() != 1` 被拒绝。
- 即使其他 arm 的直线后缀共享到 164，也无法跨过上述两个硬拒绝条件。因此当前 helper 不适用于“以一个直接 case target 为公共 continuation，且其他 case 的 DAG 可在该点继续或提前 return”的形状。

此项适用边界由当前代码直接可证；但本次没有插桩取得 helper 的实际返回值。Jarde 的整方法拒绝也可能还涉及之后的 ownership/exception coverage。因此报告为“`switch_forward_join` 源码条件确定无法选中此候选；内部实际首个拒绝阶段待观测”，不把全文失败唯一归因于此函数。

### 最小扩展候选

不增加 Region 实体。保留 `Region::Switch` 的已有 arm、join 字段和现有 `Return` 语句构造；让 join 发现阶段可以证明一个有限 DAG，其每条完整路径只会：到达同一个候选 continuation，或终止于本 arm 明确的 `Return`/`Throw`。为保持第一版边界，候选只从 switch 的直接 successor/case target 中枚举；允许候选本身是一个空 case target（此例的 164），其直接 dispatch edge 是合法 join 入口。要求候选唯一；不按 BCI 大小或最短路径猜选。

证书需覆盖以下事实：

- dispatch 的 canonical outgoing rows 与解码的 keys/default、normal-flow successor 多重集完全一致；不接受重复或非 Normal dispatch edge。
- 每个非 join target 的 closure 是同一路径、无环 DAG。普通内部节点的所有 canonical rows 必须为 Normal，且和视图 successors 完全相等；每个分支 terminal 是经 `Operation::comparison()` 证明的条件比较；每个终端叶是显式 `Return`/`Throw`，或候选 join。禁止其他无后继、异常出边、未知跳转、回边、嵌套 switch 和未证明的跨 case entry。
- 各 arm closure 两两不相交；case-entry 交叉仅允许走现有 `switch_fallthroughs` 证出的单一相邻落入。每个 closure node 的 canonical incoming 只能来自本 arm；join 的 incoming 只能来自 dispatch 的直接 join target 与已证明 arm closure。保留 `forward_join_predecessors` 对 loop header、外部前驱与回边的拒绝，并额外比对 canonical edge kind，不能只看 normal-flow 投影。
- 至少存在一个继续到 join 的路径和一个显式终止路径，否则这不是“post-dominator 缺失下补找 join”的候选 shape。若多候选通过、路径无法封闭或所有路径都 return，拒绝。

实现上有两种轻量落点。首选在现有 `switch_forward_join` 中将线性 route 改成其局部的三色 DAG outcome 扫描，仍只返回 `Option<join>`；之后现有 `switch_region` 按该 join 建 arm frame，复用 canonical `switch_fallthroughs`、arm ownership overlap 检查和 `Region::Switch` 输出。不要直接复用 `prove_switch_fallthroughs` 而不改其契约：它要求 join 已知，并将 join 当作 stop，不会把到 join 的 predecessor 记入 `case_exits`；这里需要先枚举 join 候选，并允许 candidate 自身是 case entry。避免为此抽象成可配置的通用图框架。若局部扫描与 fallthrough proof 会重复大量边读取，再考虑抽出一个私有的只读 outcome helper，并由两个消费者复用同一完整证书，而不是新增产品 API。

### 计费、取消、visited 和 AST 写出不变量

`Walker` 与证明器共享同一个 `&mut Budget`。扩展需在访问每条 canonical row、每个节点/后继、入边 ownership 检查和候选交叉检查前按实际上界收取 `AnalysisSteps`，每段循环检查取消，并将 Stop 的 `at` 固定在 switch dispatch BCI（此例 5）。复杂度必须被实际候选数、canonical edge/node 数界定；如果预算不足，返回原 Stop，不提高预算，不先扫描未计费的大型 closure。

Join 和 arm 的所有权继续按物理 block，而非“最终输出看起来只写一次”判断。证明阶段不改 `visited`；证书完整后，现有 `region_at` 按每个 case arm walk，并继续执行 `claimed` disjointness 检查。候选 join 不进入任何 case arm：到达 join 的 path 由 arm 正常结束，join 的语句由外层后续 walk 写一次。早退 return 块属于各自 case arm，写在对应 `If` 分支中。任何 Arm 路径重叠、外部 predecessor、或者新 walk 的 block delta 与证书 closure 不吻合，都保留整 switch 拒绝并引用实际覆盖块，不把“visited 已走过”当作成功 ownership。

statement switch 的 join 164 有后续 if 和 return，不能套用 builder 的 switch-expression return-folding：`switch_join_return` 只识别恰有一条 `return` 指令的 join block；`switch_join` 还要求每 arm 留一个返回值、无 fallthrough 且 arms 是 join 唯一入口。本例必须将 164–181 留作 switch 之后的普通 region。这样不会因为折叠或 settled 清理而漏掉外层 continuation。

## 后续顺序和必要观测

这两项应在当前 conditional gate 关闭后拆成独立工作，先验证真实 Walker 状态，再写最小测试：

1. **先观察 FallThroughCase2。** 仅对真实 frozen class 临时记录 BCI 11 的 `switch_region` 结果（join、fallthrough map、每 arm block owner）、外层 if branch 对其收到的 `next`、两个现有 continuation helper 的入参/返回值，以及最终 fallback code/BCI。若 first switch 本身就拒绝，不实施 outer-tail 续接；若确认为 `next=175` 且 helper 无匹配，则按前述直线-tail 方案做独立最小补丁。
2. **再观察 Switch2。** 记录 `immediate_post_dominator`、`switch_forward_join` 的每个候选/拒绝条件和后续 arm overlap 位置。若确定卡在 helper，可先只扩展候选164的有限 DAG 形状，不引入任意嵌套结构。以实际全类 default/all 两次编译、原类/JADX/Jarde `-Xverify:all` 与 value-state 对照验证。
3. 两个正例均需保留计费/取消测试路径；负例至少覆盖候选有外部入边、两个 arm 共用非 join block、case 交叉不构成相邻 fallthrough、内部有循环/非比较多出边、canonical exception edge，以及外层边界被误当作 switch join。测试只针对已出现的证明边界，不建设通用图测试框架。

截至本稿，只读 workspace 与留存产物；没有运行 Cargo/JDK/JADX/CLI/Git，没有改 workspace、formal OpenSpec 或生产源码。
