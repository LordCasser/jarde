# Root 独立验收（2026-10-08，合并）

## 判据逐项

1. **拒绝链复核（实现片第一发现）**：readAll 的发射诊断是 crossing 级联，**因**是 `prove_resource_guard_finally` 的 Transfer-完成收窄（`41: goto 53` 处 `Ok(None)`）——循环从未被认领故纯度从未被问。任务书"copy 纯度位"归因（承自 io 片登记）被测量纠正为**双部件**。采纳。
2. **门控矩阵复核**：四构建态×28 渲染——A 单独零移动、B 单独只翻 copy 族顶层循环测试位、A+B 翻 readAll+guardPlain 控制；LK/IO 守卫锚/负例/边界全列逐字节。
3. **spec 措辞自纠（root 立项失误）**：Requirement 句"读者全部是测试"与 readAll 场景自相矛盾（其 store 目标被体读=io 片"observable"注记）。实现者按可观测判据实施（双消费舞蹈身份+体声明槽+循环测试位原位表达式+其余位移动规则保留）并上报未改 root 文本——**root 现以实现判据为准更正 spec**（本验收一并修订）。
4. **root 实测**：IO 类 **0 引注**（readAll 关闭——io 域全恢复）；定向 4+2 ignored / io 4+1 全绿；oracle 3/3。
5. **门禁（权威口径）**：全量 342 ok + 2 FAILED=两已知家族（`bulk_recovery_delivery` 4-worker 计时 + `p4_plugins` 计时）单测复跑 ×2 双绿；fmt OK；openspec **315/315**；clippy root 脚本重建后 `Finished`。
6. **corpus**：指纹 +9 纯增；census `(950,4077,418,2540,8)→(956,4101,430,2576,8)`；三处期望更新（readAll/NEG.liveLine/cf06.loopCondition）均为断言更新。
7. **CI**：合并推送后 run 为准（监控在案）。

## 账本

**io 域全恢复**（countLines+readAll 0 引注）。边界：循环测试位**参数**目标保持拒绝（`LoopTestValues.storeTest` 逐字节）；movement 规则其它位保留。
