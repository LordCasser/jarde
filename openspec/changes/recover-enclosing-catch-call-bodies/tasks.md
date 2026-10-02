## 1. 基线与负例

- [ ] 1.1 重放固定 P2/P3（SHA 核对）：读 enclosing_clause 体语句枚举位与 17a 判据接入形态；记录双形基线与 17b fixture 复核。
- [ ] 1.2 构造并冻结至少三个 verifier 有效变体/负例：多语句 catch 体（丢弃调用 + return）、结果被消费形（拒绝）、体含分支（保持拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 白名单扩展

- [ ] 2.1 enclosing_clause 体白名单接入丢弃调用判据；P2/P3 恢复、整类重编运行一致（catch 路径注入触发）；17b fixture 与既有子句形态 diff 逐字不变。
- [ ] 2.2 负例保持拒绝；预算/取消不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 17a/17b/17 系列全部既有测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [ ] 3.2 P2/P3 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核白名单边界与三方行为，更新账本与巡查记录。
