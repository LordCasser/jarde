## 1. 冻结对照证据

- [x] 1.1 冻结 Java 8 源码、class 哈希、JADX/jarde 输出、直接 CFG/SSA Phi 与原/JADX runner 结果；核对三条 join 入边和唯一三输入栈 Phi，见 `../../evidence/java-syntax-2026-09-26/nested-conditional-value/`。

## 2. 恢复层证明与呈现

- [ ] 2.1 在现有 `If` 与只读 CFG/SSA 上实现共同 join 的有界条件树证明，精确配对每条叶子边与 Phi 值、唯一消费者及完整闭包；用内层在真臂/假臂和外部入边反例的定向测试验证。
- [ ] 2.2 复用现有条件表达式、类型/目标与来源构造，整树成功后原子发布最终 Phi 值和折叠计划；用冻结 `nested`/`nestedEffects` 正例及独立副作用、类型不明、重复消费者反例验证无重复执行和完整拒绝。

## 3. 独立验收

- [ ] 3.1 用 Java 8 重编完整 jarde 类并以 `-Xverify:all` 执行 runner；逐行对照原 class 与 JADX 的返回、效果和异常，检查默认/完整证据正文、来源 BCI 与语法状态。
- [ ] 3.2 运行 `cargo fmt --all -- --check`、`cargo test -p jarde-java --lib`、相邻条件值/短路/循环/异常回归及 `openspec validate recover-nested-conditional-values --strict`；记录结果并清理临时 Cargo target。
