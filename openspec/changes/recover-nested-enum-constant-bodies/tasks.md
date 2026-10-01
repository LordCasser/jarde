## 1. 定位与基线

- [x] 1.1 重放矩阵四格（冻结 jar 与 SHA）：确认两嵌套格的首个拒绝点（子类选择/名匹配/投影处插桩或逐步诊断记录），区分"子类未被准备"与"准备后名匹配失败"。
- [x] 1.2 构造并冻结至少三个 verifier 有效负例/变体：人为 `$` 命名的顶层枚举类（用户类名含 `$`）、多级嵌套 `A$B$C` 枚举（登记不做，保持现状）、嵌套 + 抽象/接口双形；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 名派生修复

- [x] 2.1 按定位修复（design 决策 1，结构事实优先）；两嵌套格全量折叠（常量体含方法文本）、整 family 重编运行一致；两顶格局与旧切片 fixture diff 断言逐字不变。
- [x] 2.2 负例与变体边界正确（人为 `$` 顶层名不误伤、多级嵌套保持现状）。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含旧常量体切片全部测试）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 两嵌套 jar 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核名派生语义与三方行为，更新 DT-12/13 账本与矩阵记录。
