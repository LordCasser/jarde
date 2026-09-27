## 1. 固定基线与边界

- [x] 1.1 冻结 pinned `TestTryCatchFinally12.TestCls.runTest(II)String` 的 BCI/opcode/异常表、具名 row、当前 fresh CLI 拒绝，以及同布局完整类的原/JADX/Jarde Java 8 重编、`java -Xverify:all` 路径结果；root 复核输入 SHA 与脚本来源。
- [x] 1.2 冻结至少两个 verifier 有效的邻近边界，含部分 case 保护和真实 TWR 候选；不能构造 verifier 有效的出口负例时，用受控 canonical 图测试并说明限制。

## 2. 资源候选门

- [ ] 2.1 在既有字段赋值完成证明中涵盖受证实例写入，让 `resources` 对本类的 named row 返回 `NotGuarded`，随后 `catches` 得到原行 handler；测试同时证明局部资源 `Store`、Java 9 copy 和既有 static-field/TWR 路径不降级。

## 3. Try 出口的唯一续走

- [ ] 3.1 在 `try_level` 使用已存在的 `body_next`、`region_at` 和 `split`，仅在 design 的单 transfer/完整前驱/唯一后继/无 handler 竞争条件均成立时认领范围外出口块；测试断言本例 `Try→Switch→Catch`、BCI 61 单 owner 和后续 BCI 79 独立 owner。
- [ ] 3.2 核全部物理 BCI、异常行及 CFG 边的来源；验证部分覆盖、额外入口/出口、预算/取消或 Builder 失败保留原子拒绝，不出现双 owner、漏块或半份 try。

## 4. 三方验收

- [ ] 4.1 原 class、pinned JADX、修后 Jarde 的完整 Java 8 源码重编并 `java -Xverify:all` 跑固定 `runTest` 各 case、default 与 IllegalArgumentException 路径；确认效果次数、返回值及 catch 顺序一致，并单独记录 InnerClasses family 范围。
- [ ] 4.2 运行相关 Guard、Try、Switch、TWR、CF-16/CF-18 回归、`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、OpenSpec strict 和 diff check；清理专用 Cargo target。
- [ ] 4.3 root 独立审阅候选判定、transfer owner、来源、负例和三方运行，对通过的固定形状更新清单；未通过的独立债务继续单列。
