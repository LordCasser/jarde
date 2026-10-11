# Switch2 condition DAG：最小实现路线（只读）

> 本地源码 pin：`/Users/lordcasser/workspace/projects/jarde` 使用的生产 JADX checkout 为 `/Users/lordcasser/workspace/testzone/jadx`，本次读取到 HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`。前一轮 `...c37e` 是 hash 笔误；当前 HEAD 就是应核对的冻结版本。此笔记只分析，不改实现。

当前缺口比 v2 更具体：`Region::If` 只保存一个 `branch/branch_bci`，而 loop chain 才保存多条 test 与统一 `&&` 或 `||` operator。已有 `TwoExitReturn` 和 `ShortCircuitValue` 已证明一类 test DAG 能由 `CanonicalBlockId` 驱动 bottom-up fold；可以复用这条路，但不能把现有 `Expr` 克隆行为直接当成共享条件实现，因为共享测试可能被复制到多棵表达式子树。

## 0x00 Switch2 这张图怎么折叠

诊断里的图是：

```text
98  -> 111 / 118
111 -> 118 / 131
118 -> 124 / 136
124 -> 131 / 136
131 -> 136
```

给每个条件记它沿着指定 outgoing edge 取真时为 `t98/t111/t118/t124`。edge 的真假方向必须来自 decoder 给出的 taken/fallthrough 关系，不能从 successor 排列或 BCI 大小猜。以诊断中的 then/else 方向归一化后，抵达 payload `131` 的条件可以因共享后缀折叠成：

```text
(t98 && !t111) || (t118 && t124)
```

执行顺序与原图相同：先测 `98`；只有 `98` 的路径要求时才测 `111`；若 `98 && !111` 已到 `131`，就不测 `118/124`；否则到共享节点 `118` 后测 `118` 和必要时的 `124`。表达式里每个 canonical test 只出现一次，所以 `118` 的调用/字段读取等副作用不会因为它有两个前驱而被静态展开两份。`131` 的 payload 和 `136` 的 return 仍分别由 then/else arm 唯一拥有，各写一次。

关键是保留中间条件图的共享身份到因式分解结束。若像当前 builder 一样先把每个节点递归展开为 `ExprKind::Conditional` 并 `.cloned()` successor 表达式，`118` 会出现在 `98` 的 then/else 两侧表达式副本中。每次运行仍可能只经过其中一个副本，但生成的 AST/source 不再是单 owner，origin 也会重复；对有副作用测试，这不是可以悄悄接受的“同一条件”。所以 fold 应用 canonical identity memoize；只有成功降成无重复 test 的短路表达式后才生成普通 `Expr`。如果某图无法在不重复 test 的前提下降成表达式，当前窄版本应整体拒绝，不能复制测试或重排求值来挽救。

这个具体布尔式成立依赖已证 edge 与端点分类。对任意 DAG 不应套用同一公式模板。`tests` 列表也不能按 BCI 排序后就称为拓扑证明；builder 的 bottom-up 顺序应从经过闭合校验的 canonical normal edges 计算，且验证每个测试节点恰有其解码声明的两个 successor、无环、单 entry、两个 terminal outcome 完整覆盖。

## 0x01 可复用的代码，不应复用的假设

### 可复用的折叠部件

`build_two_exit_return` 在 [`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs) 15735–15962 行已经完成了主要的 DAG→condition 工作：

- 检查 boolean return 类型和两个 terminal constant return，然后把两个 terminal seed 成 `true/false` expression；
- 按 test 的两条已记录 outgoing edge 取两侧 expression，从 `test_expr(bci, false)` 得到解码 test，再建 `ExprKind::Conditional`；
- 检查 test 所依赖的 SSA values 与同块 instructions，没有遗漏独立 effect；最后返回一个 boolean expression，并把闭包 instruction BCI 放入 origin。

`build_short_circuit_statement`（16567–16700 行）做相近的拓扑 fold，但输入是 `true_producer/false_producer/consumer`；它用 `test_edges`、gateway map 逐层得到 conditional value，后面再调用 `short_circuit_boolean_expression` 将 1/0 conditional 转为逻辑表达式。该转换在 29489–29562 行，仅能按 boolean 常量叶子将 ternary 化成 `&&`/`||`/`!`；它没有识别重复的 canonical test 子图，也没有保证 DAG 展开后每个 test 的单一表达式 occurrence。

合适的抽取点是一个窄的内部 `fold_condition_dag`：接收已经证明闭合的 root、canonical test IDs 与 test BCI、每个 test 的 decoded edge sense、纯 gateway edge，以及 true/false terminal IDs；以 terminal Boolean seed 和 `test_expr` 递归/拓扑 fold。不要把 `TwoExitReturn` 的 terminal boolean-return 检查、SSA Phi/consumer 约束或 `ShortCircuitValue` 的 producer/consumer 约束塞进通用 fold。两者各自的证明仍在调用处完成。

折叠节点需保留 canonical test identity，等价于内部 formula DAG，而不是立即 clone 成 `Expr` 树。做短路因式分解时沿被证明的 edge 保持测试求值顺序；再检查输出表达式中每个有副作用的 canonical test 至多有一个 occurrence。符合条件后才用现有 `logical_expression`、`short_circuit_not` 和 `Expr` origin 构造器降成最终 Java expression。`condition(op, operands, builder, branch_bci, taken)`（32567 行起）负责单个 branch 的解码 sense 和 Java operand 类型，适合作为 `test_expr` 下游依据；它不负责路径/共享证明。

### Region 端的最小合同

不需要新建第二套 Region 树，也不需要让普通 `If` 的两侧 child 共享一个 region 对象。保留现有 `then_arm/else_arm/join`，让 `If` 持有一份条件 DAG facts；可用一个小的 `ConditionDag` 字段承载，以便与 `TwoExitReturn`/`ShortCircuitValue` 共享 edge/test 形状。候选合同至少包含：

1. root identity；每个 canonical test identity 与该 block 中的 branch BCI；每个 test 按 decoder 方向区分的两个 successor；纯直达 transfer gateway 及其唯一 successor。
2. 两个终端 canonical IDs，分别映射到 then/else arm 的 entry；terminal block 不得再被 condition graph 当作 test owner。
3. proof 已证一个 entry、exact normal predecessor/successor closure、acyclic test/gateway graph、frame/path/scope 边界一致、无不支持的 exception/subroutine/return 边、两个 terminal 覆盖所有路径；不能按 BCI 建立这些性质。
4. graph fold 的 expression 降低保留原短路顺序；测试 expression 不得被重新求值，两个 terminal arm 保持各自唯一 Region owner。超出表达式树可安全表达的 DAG 形状就拒绝，不能静默复制条件或 arm。

这里不需要把所有 `Region` 都变成 DAG。DAG 仅描述两条 statement arm 之前的 test/control facts；真实 payload/return 子树仍是普通 region，按现有 blocks owner 规则各归属一次。Switch2 的 `131` 和 `136` 正好是两片物理子树的入口。

## 0x02 最小修改路线

1. 把 `two_exit_return` 已有的闭合 DAG walk抽成适用于 statement arm 的候选证明；仅把两个 terminal 从“精确 boolean constant return”改成经过 region walker 证明的两个 arm entry。继续使用 exact incoming 和 outgoing identity 集合、frame boundary、预算/cancel、atomic commit 这些已有规则。候选内 test 数上限先复用现有 24；不要在本 change 内做无界通用 CFG structuring。
2. 让 `Region::If` 表示 root test DAG + 两个普通 arm + join；现有单 branch 是 DAG 的单节点特例。普通单 branch 的构造不变成额外 region kind。对于已有 loop chain，仅共享 test/edge 结构或 fold helper，不将 loop 的同质 AND/OR 约束误用在混合 DAG 上。
3. builder 先用 canonical edges 构建 memoized condition DAG，按 decoded edge sense 得到 Boolean 分支表达式；对 shared successor 做保身份的共同子图因式分解。只有得到每 test 单 occurrence 的短路表达式时，才生成一个 `StmtKind::If`，then/else arms 各 emission 一次。Switch2 目标表达式就是 `(t98 && !t111) || (t118 && t124)`；测试应明确各 expression 来源 BCI 的 occurrence 计数及执行顺序。
4. candidate failure、预算 Stop 或表达式无法安全降级时整块拒绝，不先提交部分测试 Region；已有 switch continuation 的成功结果不能因此丢失或部分消费。

“一个 shared test identity→一个表达式 occurrence”是当前建议的安全窄合同，不等价于证明所有有共享节点的布尔 DAG 都可表示为 Java 短路表达式。若后续要支持只有引入临时局部变量、label 或更一般 statement-DAG 才能表达的形状，应另开设计，不在本次 continuation 变更里扩大。

## 0x03 需要重新审阅的 exhaustive consumers

新增 `If` condition facts 会改变它宣称拥有的 test blocks；只更新 emitter 不够。至少要过以下集合，并让未支持的 owner/shape 检查显式拒绝，而不是默认漏掉新 tests：

- [`region.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs) 的 `Region` 汇总方法：`elided_cleanup_bcis`、`blocks`、`is_structured`、`fallbacks`、`rule`（648–983 行）；以及 `project_string_switches` 的递归 visitor（1037–1089 行）。`blocks()` 应列 root 与所有 test/gateway identity 一次，再列 arms；测试/terminal 交叠应由 ownership closure 拒绝。
- `region.rs` 中检查嵌套 Region 可接受形状的 `monitor_body_supported`、`finally_body_supported`、`shared_join_body_supported`（2426–2477 行），以及 `If` 专形态的 loop/latch/cleanup 识别器。它们要么读取 new condition tests 并证明入口语义，要么拒绝非单 branch `If`，不能只把新字段忽略。
- [`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs) 的递归 Region walkers 和 owner/accounting：`collect_guards`（约 3390–3510）、`guard_return_ownership`（3518 起）、`collect_paths`（约 4300–4450）、`own_blocks`（4519 起）、`unaccounted_region_bcis`（4621 起）。条件 blocks 需要属于 If condition 的 source path 一次，两个 arms 保留独立子路径。
- builder 的 `prepare_conditional_regions`（15965–16400）和最终 region emission（201xx 附近）；前者要在 suppression/deferred binding 前形成完整 expression + statement，后者只发一个 condition 与两 arm。两类失败仍分别走既有 refusal/atomic fallback 路径。
- 用 `Region::If` 做精确形状匹配的专用 walker也要审查：loop-end/latch 识别（region.rs 10315 起）、`implicit_tail_latch_origin`（10403 起）、guard/cleanup 模式、CF12 switch-break 的递归/包装 helper（build.rs 37806–37994）。若其证明只适用于单测试，就明确加 `condition.tests.len()==1` 的边界，而不是让 DAG If 混入旧形状。
- 覆盖 `Region::If` 结构假设的架构测试需要加两类断言：所有 condition test identity/BCI 只拥有一次，所有 arm blocks 只拥有一次；改变一个 shared edge、复制一个 test occurrence、污染 terminal predecessor 或越过 boundary 都要拒绝且不产生 partial text/map。

这里列的是 exhaustive/形状消费者类别，不是要求当前 change 改写每一个 pattern match 的语义。编译器的 exhaustive match 会指出漏点；真正容易漏的是 `if let Region::If` 或 `matches!(Region::If { .. })` 后假定只有一个 branch 的 helper，因此还要全文搜 `Region::If` 并逐处判定。

## 0x04 判断

能抽取现有 DAG→Expr 折叠给普通 statement condition 使用，但最小可接受路线不是“把 `build_two_exit_return` 原样复制到 If”。抽出共享的 test DAG proof/fold 边界，保留 canonical identity 做短路共同子图折叠，再把结果交给一个正常 If 和两个单 owner arms。现有两种 builder 的从后往前求值与 test/terminal 验证可复用；`TwoExitReturn` 的 boolean terminal return 前提、`ShortCircuitValue` 的 Phi consumer 前提及其现有树形 clone 语义不可复用为新的证明。

Switch2 展示了为什么要分开这两层：test 图共享 `118`，condition expression 需要因式分解才不会复制它；输出 `131/136` 则已经适合普通 If 的两个唯一 arm owner。继续找 join 不会补上这条表达式/ownership 边界。
