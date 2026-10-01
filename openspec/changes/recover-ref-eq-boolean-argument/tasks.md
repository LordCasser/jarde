## 1. 基线与负例

- [x] 1.1 重放固定 S1（SHA 核对）：定位实参位类型判定点与既有布尔证明接入位；记录 BCI 148–156 基线。
- [x] 1.2 构造并冻结至少三个 verifier 有效变体/负例：`!=` 拼写、数值相等入双 boolean 形参、比较结果存后传参（正变体）；非比较来源的 0/1 int → boolean 位（保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 实参位布尔呈现

- [x] 2.1 实参位接入比较双臂布尔证明（design 决策 1）；S1 完整恢复、整类重编运行与基线逐字一致（含 `falsetrue` 尾段）；变体逐项恢复；负例保持拒绝。
- [x] 2.2 既有布尔位（赋值/return/条件）测试与负例零回退。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 S1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核判据、呈现与三方行为，更新 EM-05 账本与巡查记录。（root 于合并主线 94098a8a 复核：S1 恢复 `append(local2 == local2.intern())`、重编运行逐字一致（尾段 `falsetrue`）；`ireturn` 位语义逐字未动、序分支/零测试/非比较来源负例保持拒绝；全仓 2790/0、fmt/openspec 238/238。接入点复核认可：`boolean_position_values` 抽出复用分支值塌缩，形参映射按 `typed_arguments` 位移。实施模型 bigmodel/glm-5.3（非 flash——flash 周配额 10-06 重置期间的替代）。附带记录：8ffe22a8 纯文档 CI 失败为 `bulk_recovery_delivery` 已知 4-worker flake，本地复跑绿。）
