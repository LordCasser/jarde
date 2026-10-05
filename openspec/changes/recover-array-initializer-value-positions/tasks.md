## 1. 取证与基线（root 已完成大半）

- [x] 1.0 六位判别、双重证伪、jadx 双解——实测归档。（root 已完成）
- [ ] 1.1 插桩定位 "copy … has no proved local assignment" 发出处与 dup 值的 SSA 表示；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 MD/MD2/MD3（两位拒、其余恢复）；负例探针（双消费 dance 值）现状拒绝记录。
- [ ] 1.3 冻结 fixture：MD/MD2/MD3（javac23 `--release 8`）+ 真 8 腿（验证两版 dance 同构）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1 泛化：dance 单写者 + dup 单读者即呈现于消费位（元素存储 RHS / 立即下标先行——主锚两位）；多读者保持拒绝。
- [ ] 2.2 呈现保持显式形（决策 2）；四位合格位与裸立即消费零改动。

## 3. 验证与验收

- [ ] 3.1 主锚：MD.partSet 与 MD3.bareIdx2 恢复；三类 `javac --release 8` exit 0、行为逐行一致（双腿）。
- [ ] 3.2 零回退：四位合格位 + 裸立即消费 + 锯齿族逐字节不变；负例（双消费）仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：判据未放宽（单读者语义）、零回退实测；关闭 summary.md 登记行。（留 root）
