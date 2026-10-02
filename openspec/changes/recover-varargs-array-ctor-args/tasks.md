## 1. 基线与负例

- [x] 1.1 重放固定 W1/W3/W4（SHA 核对）：定位既有 varargs 内联数组证明与呈现（裸位判据源）及其与构造走查的复用接口；记录 W3.viaArrays 的 `jre_new_interleaved_effect` 基线。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体/负例：空 varargs、混合装箱、`new HashSet<>(Arrays.asList(…))`、链中插语句（拒绝）+ 数组双用途（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 走查扩展与呈现

- [x] 2.1 走查接受实参位内联数组链（design 决策 1，判据同源复用）；W3 两形与 W1.use 完整恢复、整类重编运行与基线逐字一致（`3:0:W3`、`6.0:7:W1`）；裸位/嵌套构造/普通调用位 diff 断言逐字不变。
- [x] 2.2 负例保持拒绝；预算/取消原子性不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 varargs 切片、nested-ctor-argument-sites、new@1 全家族）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [x] 3.2 W1/W3 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核判据同源性、区间边界与三方行为，更新账本与巡查记录。
