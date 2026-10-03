## 1. 取证与基线

- [ ] 1.1 重放冻结反例 `tests/fixtures/proved-java-structure/anonymous-super-dispatch/`（SHA 核对 results/fixture-sha256.txt）：用 `results/repro.sh` 复现回归（原类 `visibleDuringSuper=true` → 渲染重编 `false`）；定位 build.rs 重排判据处并确认 super 目标事实可得；记录 C1/C2 正例的 `Object.<init>` 目标。
- [ ] 1.2 冻结至少三个变体/负例：super 目标为用户类（不重排）、super 目标为平台非 Object 类（不重排）、`this(...)` 委托链（既有行为不变）；各自 `java -Xverify:all` 通过并记录实现前后呈现与行为。

## 2. 判据收紧

- [ ] 2.1 重排准入加"super 目标 == `java/lang/Object.<init>()V`"前置条件（design 决策 1）；不满足时保持既有逐字呈现与诊断（决策 2，不改整方法拒绝、不丢成员）。
- [ ] 2.2 回归测试双向钉死（决策 3）：反例断言重排未发生（捕获写入文本在 `super()` 之前）、正例断言重排仍发生且逐字不变；反例的重编行为断言为"不可编译"或"`visibleDuringSuper=true`"，二者皆不得为"可编译且行为不同"。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 daa4fb31 的三正例两负例、capture-ctor 家族、synthetic-ctor 全部测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 `repro.sh` 复跑：原类与渲染重编的行为对照如实记录（回归消除的判据是"不再出现可编译且行为不同"）；C1/C2 家族三方对照逐字不变。
- [ ] 3.3 root 独立复核判据、回归测试双向性与既有正例零回退，更新 `recover-synthetic-ctor-super-order` 的 3.3 验收记录（补记该反例遗漏）与 `present-proved-java-structure` 2.10 的状态说明。
