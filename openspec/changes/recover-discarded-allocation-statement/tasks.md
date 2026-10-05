## 1. 取证与基线（root 已完成大半）

- [x] 1.0 四形判别、jadx 有解、CD 级联、根因推断——实测归档。（root 已完成）
- [ ] 1.1 插桩定位读者门零分支的精确落点（拒绝文本 "an allocation, a copy or a cast is present that no shape verified" 的发出处）；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 DN/CD（现状拒绝态）；真 javac 8 腿 DN（确认与 Open Question 2 的组合行为）。
- [ ] 1.3 冻结 fixture：DN + CD（javac23 `--release 8` 腿）+ DN8（真 javac 8 腿）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1/2 实现：读者 `==0` 分支语句化呈现（`new N(args…);`），构造器证明复用既有链；`single_use_at` 本体与 `>1` 拒绝零改动。
- [ ] 2.2 呈现保序：丢弃语句与其后语句按 BCI 序（"after" 在 new 之后）。

## 3. 验证与验收

- [ ] 3.1 主锚：`DN.discarded` 恢复、整类 `javac --release 8` exit 0、行为一致（含真 8 腿）；`CD.main` quotes 3→0 级联解锁。
- [ ] 3.2 零回退：DN 三消费形逐字节不变；既有分配/构造测试全绿；负例（多读者）仍拒；corpus 双腿扫描差异类仅为丢弃分配形。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：落点插桩、三分支判据未破（>1 拒 / ==1 旧路径 / ==0 新语句形）、零回退实测；关闭 summary.md 登记行。（留 root）
