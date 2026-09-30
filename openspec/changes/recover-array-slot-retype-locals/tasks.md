## 1. 基线与负例

- [ ] 1.1 重放固定 A1/A2：核 fixture SHA、A2 不可编译输出与 A1 对照；确认槽 2 的两段定义/读取集事实（SSA def 归属）并记录。
- [ ] 1.2 构造并冻结至少四个 verifier 有效变体：三段类型交替（int[]/boolean[]/Object[]）、同类型多定义（不分段，逐字不变）、交叠读取真别名（维持现行为）、段内 phi 合流（维持）；记录实现前后输出。

## 2. 分段呈现

- [ ] 2.1 呈现层按定义分段（design 决策 1–2）：段边界、读取集不相交判据、各段自有声明（或新名，编译事实裁决）；A2 整类 `javac --release 8` 通过且行为与原 class 一致；A1 与 1.2 边界变体逐字/正确。
- [ ] 2.2 回归：既有数组证书（嵌套初始化器、布尔数组、窄存、部分分配）、`p3_java_recovery`、17b/typing 家族全绿；预算/取消不变。

## 3. 验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy（避免 `iter().copied().collect()` 类 CI-only lint）、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 A2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 正常路径（含副作用计数）逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复分段判据、呈现形态与三方行为，更新 EM-17/18 账本与巡查记录。
