# Root 独立验收（2026-10-08，合并）

## 巡查误记更正（root 自纠，实现片复核发现）

初版巡查 README 记 multiAwait"完整恢复"——**root 探针 awk 模式只匹配 `void <name>()`，该方法是 `int multiAwait()`，空段计数=0 的假零**（本仓反复警告的陷阱在 root 自己的巡查上自踩）。归档渲染 `results/jarde-ML.txt` 实为 3 处 not recovered（`local 1 crosses`——Condition 局部跨区）。README 已更正；multiAwait 在本片为拒绝控制件（实现片断言钉住），真缺口归 local-scope 域。

## 判据逐项

1. **双拒点复核**：R1=`lock_guard_copy` 单三字节组+`let [acquire]` 单获取+`cleanup_sequence` calls>1；R2=`prove_finally_copy` 行前调用拒+`prove_lock_guard_finally` 行起点=块起点。**关键发现**：两形的行都始于块内（canonical CFG 不在行边界断块）→ lock-guard 证书从未在匹配块被问——两 admission 都落该证书（`prove_finally_copy` 只证 Return 完成形，两形以 transfer 完成）。
2. **门控矩阵复核**：A-only 翻 nestedLocks、B-only 翻 interruptibly、A+B 双翻、multiAwait 全配置逐字节；27 输入×4 配置的 LK/IO/handlers/finally 系全 identical。
3. **diff 审查**：`guard.rs` +145/−66——`lock_guard_copies`（≤2 三字节组序列）、逆序 SSA 配对、`acquires: Vec<u32>`、**行内 lead 准入 + 每一获取不被任一行覆盖**（B 的行外判据）、宽化 cheap gate；`prove_finally_copy`/`cleanup_sequence`/`resources()` 未触。
4. **root 实测**：ML.nestedLocks 渲染 `this.b.lock(); try { count += 1 } finally { b.unlock(); a.unlock(); }`（逆序释放）；MLOrder 全类 0 引注；ignored 双驱动（normal `a b b a` 序 / caught finally 跑 / interrupted 无释放 / interruptible 形 / anchor `2`）双腿一致；ML 类自身 `2`。
5. **门禁（权威口径）**：全量 exit 0、**342 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0（root 重生成脚本——/tmp 清理后重建）；openspec **314/314**；oracle 3/3。
6. **corpus**：945 类基线对照恰 4 渲染移动（两锚×双腿）；指纹纯增；census 断言更新（双向实测）。
7. **CI**：合并推送后 run 为准（监控在案）。

## 边界裁定

三登记边界采纳：range 内嵌 try/finally（双行双 handler）、>2 锁、**分支体形**（释放副本独块+尾 return 融合→void 完成无后继块；`nestedLocksThrowing` 无该布局可呈现——配对即测量边界线）。multiAwait 真缺口归 local-scope（Condition 局部跨区）。
