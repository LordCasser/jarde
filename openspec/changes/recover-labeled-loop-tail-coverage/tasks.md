## 1. 基线与负例

- [ ] 1.1 重放固定 L1/L2/L3：核 fixture SHA、L2 的 BCI 66 残留与 L3 对照；定位标注循环尾段走查的语句核验点（与 17a 启用点的关系）并记录。
- [ ] 1.2 构造并冻结至少三个 verifier 有效变体：break-label 尾段、双层标注嵌套、尾段混合 void/链式调用语句；记录实现前后输出。

## 2. 尾段覆盖与标签拼写

- [ ] 2.1 尾段走查接入 discarded-call 判据（design 决策 1）；L2 零 `@bytecode` 残留、行为逐字一致；17a 负例与 TWR 路径零变化。
- [ ] 2.2 标签源码式拼写（design 决策 2）；全仓 `jarde_loop` 期望同步更新并注明；L1/L3 除标签外逐字不变。

## 3. 验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 L1/L2/L3 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核泛化面、标签规则与三方行为，更新控制流账本与巡查记录。
