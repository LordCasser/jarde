## 1. 基线与负例

- [x] 1.1 重放固定 F1/F2：核 fixture SHA、F2 的 `all_reads_reach_presented_writes` 拒绝基线与 F1 对照恢复；在 `presented_int_store_value` 确认 `iaload` 失败点并记录（含读侧 phi 链已闭合的证据）。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：`double[]`/引用数组累计、catch 内多次写、嵌套 try 循环累计；共享元素值、跨块读、区间插副作用指令（后三者保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 白名单扩展

- [x] 2.1 `presented_int_store_value` 新增同块数组元素读臂（design 决策 1）；F2 完整恢复、整类 `javac --release 8` 通过、行为一致（`14`）；变体逐项恢复（体内两个连续保护区域恢复；真嵌套 try 两形态为区域/图构建层既有拒绝、前后逐字一致，见证据 README——design Non-Goals 的层，不在本片放宽）；负例保持拒绝；预算/取消原子性不变。
- [x] 2.2 F1 对照 diff 断言逐字不变；既有数组/foreach/finally 切片与 `p3_java_recovery` 全绿。

## 3. 验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy（避免 CI-only patch lint）、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 F2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 正常与注入异常路径逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据、内联纪律与三方行为，更新 CF-10 账本（勾销登记债务或收窄）与巡查记录。
