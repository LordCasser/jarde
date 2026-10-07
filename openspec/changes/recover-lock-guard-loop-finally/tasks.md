# Tasks

> 纪律：门控实验先行（锁卫证书单独翻转 LK 三法；真两份清理副本负例不翻）；行号按锚点名重验（取证行号可能漂移）；`preserve_local_scope_plan` 的两条 LK 形负例是**需显式更新**的对照非可删断言。

- [x] 1.1 复核 03-boundary.md 证据链（guard.rs/region.rs 锚点按名重验）；javap 转录 LK 三法的完整序（lock→body→save→unlock→return / handler: unlock→rethrow）；门控实验。（`results/01-anchors-and-boundary.md`、`results/04-gating.md`）
- [x] 1.2 冻结锚与负例双腿：LK（巡查冻结件）+ IO 单方法探针（同证书验证，整类验收登记边界）；负例=真两份清理副本形（非锁）、unlock 接收者不同一形、handler 含自保护行形。（`tests/fixtures/recover-lock-guard-loop-finally/` + README；探针=边界负例，见 `results/04-gating.md`）
- [x] 2.1 实现锁卫证书（形状判据 + SSA 同一性 + 循环 body region reader 接线 + SavedReturn 体内写提升 + tryLock 四件套）；既有 guard 证书与 TWR 路径逐字不动。（commit `8ca9a132`；SavedReturn 就地声明而非提升，见 `results/01-anchors-and-boundary.md` §3）
- [x] 2.2 对照测试：LK 三法恢复（重编+`-Xverify:all` 行为一致）；`preserve_local_scope_plan`/`p3_loop_test_values`/TWR 系列零回退；负例拒绝逐字；两条 LK 形负例显式更新。（commit `80195b47`；`tests/recover_lock_guard_loop_finally.rs` 5/5）
- [x] 3.1 全门禁（含 oracle ignored 腿）+ corpus 指纹 + 分逻辑提交（不 push）。（`results/05-gates.md`、`results/03-corpus-and-oracle.md`）
- [x] 3.2 root 独立复核：门控、证书安全来源（SSA 同一性）、锚/负例实测、账本（local-scope 锚 12 关闭、1.2/1.3 回填评估）。（root 2026-10-07 完成，见 [verification-root.md](verification-root.md)：**实现者死于磁盘压力收尾段，root 按账本诚实纪律代收尾**（census/fingerprint/results/全套验证）；门控与四件套链复核 ✓、LK 三法 0 引注 root 实测 + 双腿 roundtrip identical ✓、336 targets ok/0 FAILED + oracle 3/3 ✓；锚 12 关闭、1.2/1.3 回填待锚 13；多锁/lockInterruptibly/多等待点/IO 残余登记）
