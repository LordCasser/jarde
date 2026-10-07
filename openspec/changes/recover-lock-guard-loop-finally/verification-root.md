# Root 独立验收（2026-10-07，合并；root 代收尾）

## 事件记录

实现者（deepseek-flash）死于磁盘压力的收尾段（fingerprint/census/证据提交未竟）。按账本诚实纪律 root 接手：已提交 `8ca9a132`（证书实现）+ `80195b47`（锚/负例/双腿 fixture + 根测试 5/5）完好；root 合并后完成 census 登记（含逐项来源注释）、fingerprint（+46 纯新增）、results/ 归档与全套验证。root 侧同步清理：subagent baseline target/checkout（+1.5G）、worktree target（30G）——磁盘 5.2Gi 临界恢复至 35Gi。

## 判据逐项

1. **门控复核**：`results/01-gating.*` —— 证书单独翻转 LK 三法；真两份副本/不同一接收者/自保护行/io 探针/p3-handlers/local-scope 两腿逐字节。四件套证据链按名复核（1 行前置/行前调用/Loop 排斥/SavedReturn 体内写）。
2. **root 实测**：LK 三法 **0 引注**（take/put lockrefs=2、tryLockQuick=1——finally 形齐）；`03-roundtrip.sh` 双腿 `1/0/true` **identical to the original**；定向 `recover_lock_guard_loop_finally` 5/5 + `preserve_local_scope_plan` 5/5（其 LK 形负例已按 re-slice 预告显式更新）；oracle ignored 3/3。
3. **门禁（权威口径）**：全量 exit 0、**336 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **309/309**。
4. **corpus**：实现片自测 1987 类**零移动**（证书为新形状，无既有行为变化）；census `(867,3777,312,2333,8)→(873,3801,332,2359,8)`（六类二十四体，逐项来源注释）；指纹 +46 纯新增。
5. **CI**：合并推送后 run 为准（监控在案）。

## 账本

- local-scope **锚 12 关闭**（guard 层证书）；1.2/1.3 测试面在位，其回填勾选待锚 13（io 整类还差 family-6+jre_new_shape 位点）；
- 登记边界：多锁/嵌套锁、`lockInterruptibly`、多等待点、IO 整类残余两族位点。
