# 固定 `TestTryCatchFinally12.runTest` 的 switch/try 所有权债务

固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally12.TestCls.runTest(II)String` 是一层 `try { switch (testNumber) { case 1/2/3: testN(excType); } } catch (IllegalArgumentException e) { ... }`，然后在保护范围外执行 `return sb.toString()`。它本身没有 `finally`；清理发生在被调用的 `test1/2/3` 中。

Java 8 class 的具名异常行是 `[11,61) → 64 IllegalArgumentException`。BCI 12 为 switch dispatch，case 入口为 BCI 40、48、56，共同正常汇合点 BCI 61 位于保护范围外，handler 从 BCI 64 开始，后续 return 从 BCI 79 开始。主线 `61a167b2` 的 Jarde 报 `jre_region_ownership_overlap`：canonical BCI 40 在完成的 Region tree 中有多个 owner，故整方法安全引用，完整固定类不能重编。这个拒绝与本轮 `test1/2/3` 的 `jre_guard_finally_copy` 是独立缺口。

主线 `9440dd30` 的隔离诊断进一步拿到完成树：`ROOT[0] Region::Switch` 的 case 1 arm 是 BCI 40 的 `Region::Fallback(ExceptionEdge, handler ordinal 0)`；`ROOT[1]` 又是同一个 BCI 40 的 sibling fallback。case 2/3 的 BCI 48/56 也各被 arm 和 sibling 重复认领。形成点是 `region.rs::switch_region` 将 `arm_run` 收作 case arm，又把 `entered` 中的同一 fallback 过滤后追加到 switch 后的 root run（约 8955–8993 行）；这里的 sibling 并非未由 arm 持有的尾部。`overlapping_owner` 拒绝正确地阻止双 owner。

同一隔离副本的临时诊断 instrumentation 已找到外层 `Region::Try` 缺失的确切原因。`runTest` 的 BCI 0 canonical block 含指令 `[0,1,4,5,8,11,12]`；具名 row 为 ordinal 0、`[11,61) → 64`、catch type `IllegalArgumentException`。`starts_catch` 为真，`guard::catches` 在 `guard.rs::catches` 先把该 named row 读成单一 range `[11,61)`，随后在 `guard.rs:4311` 调 `guarded()`。该调用返回 `Some(Refused { pass: twr@1, code: jre_guard_resource_init, at: 8 })`，拒绝信息是“resource's own initialisation is not one statement of this block whose value lands in a slot”。

拒绝来自 TWR 假候选：`guard.rs::resources`（约 5062–5070 行）按 entry block 中被异常行覆盖的指令收集候选，BCI 11 使该 row 入选。保护范围起点前一条指令是 BCI 8 `putfield sb`（固定 `javap` 字节码），而不是资源声明所需的 local `Store`。当前候选路径在 `resources`（约 5176–5181、5220–5242 行）没有把这个 `FieldPut` 识别为不可能的资源头，令 `copy=None`、`header=true`，然后尝试 `twr()`。`twr` 的 `initialisation(end=11,floor=0)`（约 5305–5314 行）回看 BCI 8；`guard.rs::initialisation`（1047–1057 行）要求该位置是 `Operation::Store`，因此以 `ResourceInit` 锚定 BCI 8。`resources` 将它保留为 `twr@1` refusal（约 5242–5263 行）。

`guard.rs::catches` 把 `guarded()` 的任何 `Some(Verdict)` 都视为 guarded rule 已拥有该形状，并在 `Some(_) => return Ok(None)`（4329 行）退出；所以这个 `Refused` 阻止 named catch 继续解析。`region.rs::try_region`（3619–3632 行）收到 `None` 后返回 `None`，外层 `region_at` 因此没有构造 `Region::Try`。这条执行路径没有调用 `clause_sites` 或 `join_after(end=61)`；不能把本例归因于 join 推导失败。

BCI 8 是进入 protected range 前对 `this.sb` 的普通字段初始化，不是 TWR resource 声明。诊断边界是不能全局忽略 `catches` 收到的 `Some(Refused)`；当局部 `Store`、构造/调用值或 Java 9 resource copy 使资源头确实可能成立时，真实 TWR refusal 仍须先于 named catch 并保持拒绝。临时实验结果见下段；这些窄 patch 只用于区分这份 class 的形成路径，不是正式实现。

两阶段临时实验均在 `/private/tmp` 的 `9440dd30` 隔离副本中进行。Stage A 只跳过固定输入的候选组合（current BCI 0、row 0、range start 11、前一指令 BCI 8 且为 `Field Write`）。此时 `guarded()` 返回 `None`；named row 成功得到 `CatchSite(type_indices=[37], handler=64, parameter=3)`，`join_after(61)=Some(79)`。完成树为 `ROOT[0] Try { lead=(0,11), body=Switch(branch block BCI 0 / switch BCI 12, arms Straight[40]/Straight[48]/Straight[56], join=61), catches=[handler 64, body Straight[64]] }`，其 owner BCI 为 `[0,40,48,56,64]`；后续为 `ROOT[1] Straight[79]` 和 `ROOT[2] Fallback(UncoveredBlocks [61])`。报告不再是 ownership overlap，而是 `jre_region_uncovered_blocks`，消息指出唯一未覆盖 live block 为 BCI 61。

Stage B 在 Stage A 基础上，再临时过滤已经由 built switch arms 持有的 fallback sibling。此时三个 arm 都是 `Straight`，原先的 arm fallback 和 sibling fallback 均不存在，去重条件没有触发；Stage A 与 Stage B 的完整 Java 输出逐字节相同。因此这份固定 class 的证据是：TWR 假候选被窄跳过后，外层 Try frame 已让 case blocks 成为普通 arm，原 BCI 40/48/56 双 owner 随之消失；独立 sibling 去重对这个新树没有效果。这不证明其它 switch 形状无需独立 owner 约束。

Stage B 的完整输出在用临时 no-op `JadxAssertions` 依赖 stub 后通过 `javac --release 8`；再由 `java -Xverify:all` harness 成功加载 class。该编译与加载只验证这份输出可解析并通过 JVM 校验，不验证运行结果。`runTest` 仍带 BCI 61 fallback；同一完整类的 `test1/test2` 仍是原有 `jre_guard_finally_copy` explanation-only 空方法，故不能把这次结果当成完整类语义验收。

### BCI 61 的精确丢失点与临时归属试验

Stage A 的唯一 uncovered BCI 61 并非 switch arm 漏走，而是 Try 边界计算与 body walk 的两个合法位置不同：`guard::join_after(end=61)` 将 BCI 61 的 `Transfer` 沿唯一普通后继推导为 79；`region.rs::try_level` 的无 inner 分支以 `shape.join=79` 设 protected boundary，调用 `region_at(start=0, ...)` 后收到 `body_next=61`，却在 `let (body, _)` 丢弃这个 next，并把 Try boundary 保持为 79。switch 本身记录 `join=61`，于是从 body walk 返回时 BCI 61 没有 owner；Try 之后的 root walk 从 79 继续，最后 uncovered 扫描报 BCI 61。对应主线位置为 `region.rs::try_level` 约 3720 行，`guard.rs::join_after` 约 4488 行；行号随源文件变动，应以函数名为准。

隔离副本的 BCI 61 CFG instrumentation 给出：canonical block span `[61,64)`，完整指令列表只有 `(61, Transfer)`；normal edges 是 `0→61`、`40→61`、`48→61`、`56→61`、`61→79`，无其它块入边，唯一 normal successor 为 79，walk 前 `visited=false`。它位于异常表 `[11,61)` 的 exclusive end 之外。因此在这个固定输入里，`body_next=61` 正是由 switch 的四个出口汇合而来的 try 外单 transfer bridge，而非 protected body 的新语句。

在 Stage A 的 TWR 假候选窄跳过基础上，临时让 `try_level` 对 `body_next=61` 再调用一次现有 `region_at(61, 同一 protected frame)`，所得 run 是 `Straight[61]`、next 为 `None`。将它接在原 body run 后，现有 `split` 保留首个 Try body `Switch`，并把尾部 `Straight[61]` 作为紧邻 Try 的 sibling 发出；后续仍是 `Straight[79]`。完成树的 owner 变为 Try `[0,40,48,56,64]`、sibling `[61]`、后续 `[79]`，BCI 61 来源得到保留且只认领一次。`runTest` report 为 `quality=structured`、`fallbacks=[]`，concat split 仍有单独 warning；生成 Java 与 Stage A 输出相同，transfer 在 Java 控制流中由紧接的 lexical continuation 表示。这个 A 方案沿用 `region_at` 和 `split`，不需要新 Region 节点，也不声称 BCI 61 是 Try 的物理 body owner。

另一个临时 B 方案把 `[Switch, Straight[61]]` 包为单个 `Sequence` 并保留在 Try body。它得到 `Try.body=Sequence(Switch, Straight[61])`，Try 的 `blocks()` owner 包含 `[0,40,48,56,61,64]`；`runTest` 报告同样无 fallback，生成 Java 也相同。B 证明把 transfer 显式放进 Try body 在当前生成链可行，但需要阻止 `split` 将尾项发成 sibling，超出 A 的最小路径。A/B 都只在 `/private/tmp` 副本做诊断，没有进入仓库产品代码。

若为未来修复评估 A 路径，当前 fixture 仅支持以下五项候选窄前提，必须分别以有效负例检验：

1. body walk 明确返回 `body_next`，且它恰为异常保护范围 exclusive end 对应的 canonical block；
2. 该 block 的完整指令序列恰有一条指令，解码为 `Operation::Transfer`；
3. 当前 normal-flow view 中该 block 恰有一个普通后继，且该 successor 等于 `join_after(end)` / Try lexical boundary；
4. transfer block 在 protected range 外，且不作为任何 handler entry；
5. 该 block 的所有普通前驱都已由这个 Try 的 protected body 所有权覆盖。

这些条件是本例可观察事实整理出的边界候选，不是已经证明的通用充分条件；尤其不能仅因一个 block 未覆盖就沿普通边把它并入 Try 或忽略 uncovered。A 方案至少还需要有效反例覆盖非 transfer end、多个 successor、前驱不全属于 body、handler entry/异常边，以及 transfer 去往非 lexical boundary 等情况。临时输出能编译或被 JVM 加载也只证明语法/验证层，不替代运行语义验收。

完整正例的候选边界是单条具名 catch 完整覆盖 switch dispatch 和各 case 正文、唯一正常 join 在范围外；固定类行从 BCI 11 起，而 switch opcode 在 BCI 12、canonical block 从 BCI 0 起，必须明确处理同块 lead。保护范围只覆盖部分 arm、切入 case body、handler 交叉、arm 间跳转、非唯一 join、handler 回流到 try 正文应拒绝。已有 verifier 有效的 `PartialSwitchCatch` 临时负例只保护 case 1 调用，Jarde 没有把它扩成整 switch 外层 catch；它仍有其它局部 fallback，不能声称完整恢复。不能因现有 `TestSwitchWithTryCatch`（switch 外层、每个 case 内层 try）通过或失败，就推定此反向嵌套形态的结果。

### 主线修复状态

上述形成路径已由 [OpenSpec](../../changes/recover-switch-inside-named-catch/) 的受证字段前缀与单 transfer 出口闭合。root 在 `4c6301ad` 上完成[独立验收](../java-syntax-2026-09-28/cf16-switch-catch/after/root-acceptance.md)：固定类与同布局最小完整类的原/JADX/Jarde 完整 Java 8 源码各九路径重编、验证运行逐字一致，全部物理 BCI 有来源，BCI 61 单独拥有；四个 verifier 有效近邻与受控额外边未被误认。上文历史诊断保留为形成路径，不再描述当前主线的失败状态。
