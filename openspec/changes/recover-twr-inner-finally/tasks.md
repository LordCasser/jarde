## 1. 取证与基线

- [ ] 1.1 重放固定 T4（SHA 核对）：javap 精确记录三层降低（TWR-a 副本、TWR-b 副本、finally-mid 副本）的行表与 BCI；判定 dispatch 尝试序与 `prove_finally_copy` 在该语境的具体失败环；记录基线。
- [ ] 1.2 构造并冻结至少四个 verifier 有效变体/负例：外层 TWR+内层 finally（无内层 TWR）、双层 TWR（无 finally，回归）、内层 finally 含 return、内层行与 suppression 交叠（拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 正文子证书与呈现

- [ ] 2.1 TWR 正文证明接受内层显式 finally 子形态（design 决策 1，按取证定接入层）；T4.nested 完整恢复、整类重编运行一致（`body[b]mid[a]`）。
- [ ] 2.2 纯 TWR/纯 finally 全家族 diff 断言逐字不变；交叠负例保持拒绝；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含 TWR 全家族：multi-resource、nullable、17a/17b、void-loop 等）、fmt、CI 完整 30 项 allowlist clippy、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [ ] 3.2 T4 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [ ] 3.3 root 独立复核子证书边界、呈现序与三方行为，更新账本与巡查记录。
