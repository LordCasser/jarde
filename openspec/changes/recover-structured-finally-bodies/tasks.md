## 1. 冻结边界与证书

- [ ] 1.1 重新核对 `ImplicitCleanup.run()` 的 Java 8 class、异常表、SSA/CFG、原/JADX/Jarde 完整类与四行 runner；冻结哈希和执行差异，并用 `java -Xverify:all` 复现 JADX 的 `299` 与原类的 `29`。
- [ ] 1.2 复用既有 `FinallyCopyProof` 核对 `[0,20)`、两份清理、保存返回、原异常身份及唯一正常出口；对扩围、不同副本、竞争 handler 与外部入口运行定向拒绝测试。

## 2. 有界结构化正文及所有权

- [ ] 2.1 让已证非直线 finally 候选在 visited 提交前用受限 Frame 恢复正文，并只由外层证书消费匹配的 catch-all 边；以分支/抛出正例和额外异常/Call 边反例验证不重入、不吞边。
- [ ] 2.2 核对子 Region 的完整结构、受保护块/BCI 闭包及唯一物理 owner 后原子提交 Guard；用外部入口、重叠块、预算/取消在递归中停下的测试证明失败时恢复状态并完整引用。

## 3. 融合块的正确发射

- [ ] 3.1 在 Guard 子正文中只发射受保护半开 BCI 区间，将已证明的保存值 return 插在唯一正常臂内，清理仅作为一个 `finally` 正文；用完整 Java 8 重编、作用域检查及四种完成路径的值/异常对象/trace 测试验证。
- [ ] 3.2 让声明规划、条件值预备和来源遍历看到内部正文而不增加第二物理 owner；对子树、return 或清理的呈现失败作可变状态回滚，验证没有半个 `finally`、重复效果或丢失延迟生产者。

## 4. 独立验收与回归

- [ ] 4.1 Root 从重建 CLI 对冻结完整类输出直接以 `javac --release 8` 和 `java -Xverify:all` 运行，逐行比对原/JADX/Jarde；核对默认/all 正文一致、真实 BCI/成员来源完整，明确记录 JADX 的错误路径。
- [ ] 4.2 运行 finally、guard、Region 来源、资源、monitor、命名 catch、覆盖型拒绝及预算/取消回归，执行 `cargo fmt --all -- --check`、相关 Rust tests 和 `openspec validate recover-structured-finally-bodies --strict`；记录磁盘占用并清理隔离 Cargo target。
