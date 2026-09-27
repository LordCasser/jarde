# 固定 `TestTryCatchFinally12.runTest` 的 switch/try 所有权债务

固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally12.TestCls.runTest(II)String` 是一层 `try { switch (testNumber) { case 1/2/3: testN(excType); } } catch (IllegalArgumentException e) { ... }`，然后在保护范围外执行 `return sb.toString()`。它本身没有 `finally`；清理发生在被调用的 `test1/2/3` 中。

Java 8 class 的具名异常行是 `[11,61) → 64 IllegalArgumentException`。BCI 12 为 switch dispatch，case 入口为 BCI 40、48、56，共同正常汇合点 BCI 61 位于保护范围外，handler 从 BCI 64 开始，后续 return 从 BCI 79 开始。主线 `61a167b2` 的 Jarde 报 `jre_region_ownership_overlap`：canonical BCI 40 在完成的 Region tree 中有多个 owner，故整方法安全引用，完整固定类不能重编。这个拒绝与本轮 `test1/2/3` 的 `jre_guard_finally_copy` 是独立缺口。

主线 `9440dd30` 的隔离诊断进一步拿到完成树：`ROOT[0] Region::Switch` 的 case 1 arm 是 BCI 40 的 `Region::Fallback(ExceptionEdge, handler ordinal 0)`；`ROOT[1]` 又是同一个 BCI 40 的 sibling fallback。case 2/3 的 BCI 48/56 也各被 arm 和 sibling 重复认领。形成点是 `region.rs::switch_region` 将 `arm_run` 收作 case arm，又把 `entered` 中的同一 fallback 过滤后追加到 switch 后的 root run（约 8955–8993 行）；这里的 sibling 并非未由 arm 持有的尾部。`overlapping_owner` 拒绝正确地阻止双 owner。

同一隔离副本的临时诊断 instrumentation 已找到外层 `Region::Try` 缺失的确切原因。`runTest` 的 BCI 0 canonical block 含指令 `[0,1,4,5,8,11,12]`；具名 row 为 ordinal 0、`[11,61) → 64`、catch type `IllegalArgumentException`。`starts_catch` 为真，`guard::catches` 在 `guard.rs::catches` 先把该 named row 读成单一 range `[11,61)`，随后在 `guard.rs:4311` 调 `guarded()`。该调用返回 `Some(Refused { pass: twr@1, code: jre_guard_resource_init, at: 8 })`，拒绝信息是“resource's own initialisation is not one statement of this block whose value lands in a slot”。

拒绝来自 TWR 假候选：`guard.rs::resources`（约 5062–5070 行）按 entry block 中被异常行覆盖的指令收集候选，BCI 11 使该 row 入选。保护范围起点前一条指令是 BCI 8 `putfield sb`（固定 `javap` 字节码），而不是资源声明所需的 local `Store`。当前候选路径在 `resources`（约 5176–5181、5220–5242 行）没有把这个 `FieldPut` 识别为不可能的资源头，令 `copy=None`、`header=true`，然后尝试 `twr()`。`twr` 的 `initialisation(end=11,floor=0)`（约 5305–5314 行）回看 BCI 8；`guard.rs::initialisation`（1047–1057 行）要求该位置是 `Operation::Store`，因此以 `ResourceInit` 锚定 BCI 8。`resources` 将它保留为 `twr@1` refusal（约 5242–5263 行）。

`guard.rs::catches` 把 `guarded()` 的任何 `Some(Verdict)` 都视为 guarded rule 已拥有该形状，并在 `Some(_) => return Ok(None)`（4329 行）退出；所以这个 `Refused` 阻止 named catch 继续解析。`region.rs::try_region`（3619–3632 行）收到 `None` 后返回 `None`，外层 `region_at` 因此没有构造 `Region::Try`。这条执行路径没有调用 `clause_sites` 或 `join_after(end=61)`；不能把本例归因于 join 推导失败。

BCI 8 是进入 protected range 前对 `this.sb` 的普通字段初始化，不是 TWR resource 声明。最小修复边界是收窄 `resources` 的候选前提：这种非 local-`Store` 的前驱不能单独把 named-catch range 变成 TWR 候选，让它回到普通 named-catch 解析。不要在 `catches` 中全局忽略 `Some(Refused)` 或降低 guarded rule 的优先级；当局部 `Store`、构造/调用值或 Java 9 resource copy 使资源头确实可能成立时，真实 TWR refusal 仍须先于 named catch 并保持拒绝。只修这个 TWR 假候选也不会消除 Switch arm 与 sibling 对 BCI 40 的重复 owner，两个边界应分开处理。

完整正例的候选边界是单条具名 catch 完整覆盖 switch dispatch 和各 case 正文、唯一正常 join 在范围外；固定类行从 BCI 11 起，而 switch opcode 在 BCI 12、canonical block 从 BCI 0 起，必须明确处理同块 lead。保护范围只覆盖部分 arm、切入 case body、handler 交叉、arm 间跳转、非唯一 join、handler 回流到 try 正文应拒绝。已有 verifier 有效的 `PartialSwitchCatch` 临时负例只保护 case 1 调用，Jarde 没有把它扩成整 switch 外层 catch；它仍有其它局部 fallback，不能声称完整恢复。不能因现有 `TestSwitchWithTryCatch`（switch 外层、每个 case 内层 try）通过或失败，就推定此反向嵌套形态的结果。
