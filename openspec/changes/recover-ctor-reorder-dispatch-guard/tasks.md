## 1. 取证与基线

- [ ] 1.1 重放冻结反例 `tests/fixtures/proved-java-structure/anonymous-super-dispatch/`（SHA 核对 results/fixture-sha256.txt）：用 `results/repro.sh` 复现回归（原类 `visibleDuringSuper=true` → 渲染重编 `false`）；落点在 `crates/jarde-java/src/ctor_order.rs::present_prologue_first`（daa4fb31 新增的 255 行模块；其 `build.rs` 侧仅 +17 行接线），确认 `init::Prologue` 携带的 `class`（super 目标内部名，init.rs:1507）与目标 descriptor 在判据处可得；记录 C1/C2 正例的 `Object.<init>` 目标（实测 `capture_ctor_class` 生成器 pool 第 4/8/43 行即 `java/lang/Object`+`<init>`+`()V`）。
- [ ] 1.2 冻结/构造至少三个负例：`anonymous-super-args/AnonymousSuperArgs$1`（super `Base`，非 Object → 退回 verbatim）、`anonymous-capture/AnonymousCaptureCases$1`（super `AnonymousCaptureCases$Base` → 退回 verbatim）、生成器构造的用户类 super 形（无需新 fixture）；各自记录实现前后呈现、重编状态与 `java -Xverify:all` 行为。

## 2. 判据收紧

- [ ] 2.1 重排准入加 super 目标等于 `java/lang/Object.<init>()V` 的单档前置条件（owner 与 descriptor 双匹配，design 决策 1）；不满足时保持既有逐字呈现与诊断（决策 3：不改整方法拒绝、不丢成员）。**不引入跨类读 super ctor 体的宽档**——决策 1 已用三条实证否证（跨类读体无既有先例、headers-only 覆写名代理会反向回归、单类事实无法区分三处 fixture）。
- [ ] 2.2 回归测试三向钉死（决策 4）：(a) `anonymous-super-dispatch` 断言重排未发生（捕获写入文本在 `super()` 之前）且不得出现"可编译且行为不同"；(b) C1/C2 与 `capture_ctor_class` 全部正例断言重排仍发生、逐字不变；(c) 新增负例（super 为非 Object 用户类）断言不重排、退回 verbatim。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 daa4fb31 的三正例两负例、capture-ctor 家族、synthetic-ctor 全部测试）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 `repro.sh` 复跑：原类与渲染重编的行为对照如实记录（回归消除的判据是"不再出现可编译且行为不同"）；C1/C2 家族三方对照逐字不变；两处非 Object super fixture 退回 verbatim 后行为仍与原 class 一致（响亮失败：完整源集编译退出 1，与 2026-09-27 既有登记状态一致）。
- [ ] 3.3 root 独立复核判据、回归测试三向性与既有正例零回退，更新 `recover-synthetic-ctor-super-order` 的 3.3 验收记录（补记该反例遗漏）与 `present-proved-java-structure` 2.10 的状态说明（本片实现了其"独立二进制名类视图不声称可编译"立场的一半；另一半即可编译性由 5.3 类级匿名语法承担）。
