## 1. 取证与基线

- [ ] 1.1 重放固定 N2（含 `N2$Operation`、`N2$Operation$1/$2`、`N2$IOperation`）：核 fixture SHA、主线逐字段降级输出；确认家族准备通道是否为匿名枚举子类准备报告（第一个取证义务），记录通道现状与扩展点。
- [ ] 1.2 构造并冻结至少五个 verifier 有效负例/变体：子类 ctor 体有额外语句、子类含非覆盖成员、子类体方法恢复拒绝（整组逐字段）、常量体带字段构造（混合形态，保持现状）、常量体方法恢复 structured 的正变体；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 子类义务证明与折叠

- [ ] 2.1 常量步骤接受 `new Sub`（design 决策 1：关系/ctor 委托体/成员集义务）；匿名子类准备集扩展（决策 3，如需）；N2 全量折叠、子类不再独立呈现。
- [ ] 2.2 常量体呈现（决策 2）：子类方法文本按序拼接、@Override 按既有通道；混合形态与拒绝负例保持逐字段；N0/N1/N3 与四固定形 diff 断言逐字不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含全部 enum 切片与 member-family 装配测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 N2 家族三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致（`7`/`7`）；记录输出 SHA。
- [ ] 3.3 root 独立复核子类义务、准备集边界与三方行为，更新 DT-12/13 账本（family 清项或登记混合形态剩余）与巡查记录。
