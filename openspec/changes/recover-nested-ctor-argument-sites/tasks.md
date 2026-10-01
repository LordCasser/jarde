## 1. 基线与负例

- [ ] 1.1 重放固定 X1/X2（SHA 核对）：读 init.rs 站点序列扫描结构（实参收集推进/接收者检查位），记录 X2.nested 的 `@12` 误判路径与判别三档基线。
- [ ] 1.2 构造并冻结至少四个 verifier 有效变体/负例：双内嵌实参、内嵌于第二参位、三层（登记现状）、内嵌双用途（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 走查扩展与呈现

- [ ] 2.1 走查接受实参位内嵌完整站点（design 决策 1，深度上限 2）；X2.nested/X1.main 完整恢复、整类重编运行一致（`outer`/`inner` 回归）；内嵌呈现复用构造拼写（决策 2）。
- [ ] 2.2 既有构造形态（单参方法实参、throw 位、receiver、varargs ctor、lambda/匿名构造）diff 断言逐字不变；负例保持拒绝。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 new@1 全部既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 X1/X2 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核走查扩展、区间判据与三方行为，更新构造呈现账本与巡查记录。
