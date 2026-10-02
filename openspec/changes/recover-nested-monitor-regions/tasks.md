## 1. 基线与负例

- [x] 1.1 重放固定 T4（SHA 核对）：读 monitor 证书配对扫描结构，记录内层对的识别位与 T4.sync 基线（`jre_guard_monitor`@4 + 6 未覆盖块）。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：不同锁对象嵌套（T4 形）、内层在循环内、内层含 return、内层 exit 缺失/跨出（拒绝）；同锁可重入一形（登记现状）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 嵌套配对与呈现

- [x] 2.1 monitor 证书接受恰一对配对内层（design 决策 1–2）；T4.sync 完整恢复（嵌套双 synchronized + 循环 + 返回）、重编运行一致（`10`）；单层/既有 monitor 形态 diff 断言逐字不变。
- [x] 2.2 变体逐项恢复；负例保持拒绝；预算/取消原子性不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 monitor 全部既有测试：split-monitor-slot、synchronized-multi-exit 等）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 T4 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核配对判据、呈现与三方行为，更新账本与巡查记录。（root 于合并主线 58d629d7 复核：T4.sync 呈现嵌套双 synchronized + 循环 + 返回，2807/0、fmt/openspec 242/242；NM 纯 fixture 三方一致覆盖本片行为，T4 首行（nested，第二片目标）证据口径为原类腿已在其 README 区分。既有测试改动复核：p3_boolean_short_circuit_return 断言集逐字未变、仅以 16MiB 显式栈线程承载——render_value 深递归的既有余量债务登记（MAX_VALUE_DEPTH 不变）。同锁可重入按设计登记为接受；三层/空内层/monitor×finally 复合继续拒绝。）
