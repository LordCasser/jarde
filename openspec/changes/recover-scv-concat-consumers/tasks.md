## 1. 取证与基线

- [ ] 1.1 重放固定 B1–B5（SHA 核对）：javap 确认拼接消费的实际 append descriptor（Z vs 装箱 Object）；读短路值切片实现定位消费方枚举点与 "no SSA proof" 拒绝路径；记录判别链三档基线。
- [ ] 1.2 构造并冻结至少三个 verifier 有效负例/变体：布尔装箱拼接（`"" + hasA` 头位）、双短路双拼接、拼接前局部被二次读（保持退化）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 消费方扩展

- [ ] 2.1 消费方集合纳入拼接 append 位（design 决策 1，descriptor 按取证）；B5.s1/s2、B4.v1–v3、B2.compound 全恢复、整类重编运行一致；B3/s3 diff 断言逐字不变；负例保持退化。
- [ ] 2.2 装箱/头位变体边界正确（按既有装箱通道呈现）；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 short-circuit 全家族与 concat 切片）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 B5/B4/B2 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核消费方边界与三方行为，更新 EM-19/CF-20 账本与巡查记录。
