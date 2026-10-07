# Tasks

> 纪律：门控实验先行（A/B 各自独立门控：A 单独翻 nestedLocks、B 单独翻 interruptibly、LK/IO 不动）；行号按锚点名重验。

- [ ] 1.1 插桩复核两形的四件套拒绝链（finally 体单调用判据/行前可抛判据的发出处）；门控实验转录存证据。
- [ ] 1.2 冻结锚与负例双腿：ML（巡查件）+ 嵌套 try 形；负例=释放序≠获取逆序形、unlock 接收者无对应 lock 形、行内（非行前）可抛调用形——各保持拒绝。
- [ ] 2.1 实现两扩展（多语句 finally 体的 SSA 配对 + 行外可抛准入）；既有证书判据逐字不动。
- [ ] 2.2 对照测试：两形恢复（重编+`-Xverify:all` 双驱动——normal/异常路径解锁序一致，`2` 输出）；`recover_lock_guard_loop_finally`/`recover_io_resource_finally` 套件零回退；负例拒绝逐字。
- [ ] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。
- [ ] 3.2 root 独立复核：门控、SSA 配对/行外判据、锚/负例实测、账本（多锁族边界关闭、多等待点实测关闭记录）。（留 root）
