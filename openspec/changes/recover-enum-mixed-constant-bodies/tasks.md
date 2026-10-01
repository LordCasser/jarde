## 1. 取证与基线

- [ ] 1.1 重放固定 `p.Combo`（fam.jar，SHA 核对）：javap 核对组合形态桥/子类/主 ctor 描述符与 `<clinit>` 步序列；定位现有义务证明首个拒绝点并记录（区分常量步骤校验/关系解析/呈现合成三层）。
- [ ] 1.2 构造并冻结至少四个 verifier 有效负例/变体：嵌套+混合叠加一形、三参带体、getstatic 参带体、转发链断参（整组逐字段）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 义务参数化与呈现

- [ ] 2.1 子类/桥 ctor 义务参数化（design 决策 1，≤3 参、实参种类沿用 arbitrary-arguments）；`p.Combo` 三常量混合全量折叠 `ADD(1) { … }, MUL(2) { … }, ID(0);`、整 family 重编运行一致（`7`/`12`/`0`）。
- [ ] 2.2 呈现合成（design 决策 2，两段复用既有函数）；`demo.Op`、`N0`/`N3`/`N1` diff 断言逐字不变；负例整组保守。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 enum 全家族与 member-family 测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 `p.Combo` 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核义务参数化、呈现合成与三方行为，更新 DT-12/13 账本与巡查记录。
