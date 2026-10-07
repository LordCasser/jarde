# Tasks

> 纪律：门控实验先行（A/B 各自独立门控：A 单独翻 nestedLocks、B 单独翻 interruptibly、LK/IO 不动）；行号按锚点名重验。

- [x] 1.1 插桩复核两形的四件套拒绝链（finally 体单调用判据/行前可抛判据的发出处）；门控实验转录存证据。→ [results/01-refusal-points.md](results/01-refusal-points.md)（两拒绝点：`lock_guard_copy`/`lock_guard_copies` 的单调用副本文法 + `prove_finally_copy` 的行前调用拒绝与 `row.start_bci == current.bci()`；含插桩测量）、[results/03-gating.md](results/03-gating.md) + [03-gating.out](results/03-gating.out)（A-only 只翻 nestedLocks、B-only 只翻 interruptibly、A+B 翻两形、27 输入 ×4 构型除两锚外逐字节不变）
- [x] 1.2 冻结锚与负例双腿：ML（巡查件）+ 嵌套 try 形；负例=释放序≠获取逆序形、unlock 接收者无对应 lock 形、行内（非行前）可抛调用形——各保持拒绝。→ [tests/fixtures/recover-nested-lock-finally-bodies/README.md](../../../tests/fixtures/recover-nested-lock-finally-bodies/README.md)（ML 巡查件逐字 + MLOrder 顺序/异常腿 + MLNegatives 三负例 + MLProbe 两登记边界；两腿 javac 23 `--release 8 -g:none` / 真 javac 8）
- [x] 2.1 实现两扩展（多语句 finally 体的 SSA 配对 + 行外可抛准入）；既有证书判据逐字不动。→ 提交 `feat(java): present the nested lock's two-statement finally and a throwing acquisition outside the rows`（`lock_guard_copies` 序列副本文法、`acquires` 逆序 SSA 配对、lead 在块内 + 每一获取行外；`prove_finally_copy`/`cleanup_sequence`/`resources()` 零触碰）
- [x] 2.2 对照测试：两形恢复（重编+`-Xverify:all` 双驱动——normal/异常路径解锁序一致，`2` 输出）；`recover_lock_guard_loop_finally`/`recover_io_resource_finally` 套件零回退；负例拒绝逐字。→ [tests/recover_nested_lock_finally_bodies.rs](../../../tests/recover_nested_lock_finally_bodies.rs)（6 测试：4 常规 + 2 ignored 双腿驱动/锚行为）；LK 5/5、IO 4+2ignored 绿
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。→ [results/04-corpus-and-gates.md](results/04-corpus-and-gates.md)：945 class 普查仅 4 处移动（本片两锚两腿）、指纹纯增 18 条、读者人口断言按惯例更新、fmt/clippy(-D warnings)/workspace 342 ok·0 FAILED/oracle 3/3/openspec 314 全绿
- [ ] 3.2 root 独立复核：门控、SSA 配对/行外判据、锚/负例实测、账本（多锁族边界关闭、多等待点实测关闭记录）。（留 root）
