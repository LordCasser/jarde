## 1. 取证与基线（root 已完成大半）

- [x] 1.0 四形判别、jadx 双解、时间引注已知旧值存在——实测归档。（root 已完成）
- [ ] 1.1 插桩定位时间引注发出处与 SSA 旧值节点表示；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 CM/CM2（两形拒、健康形恢复）；负例探针（`i = i++`）现状拒绝记录。
- [ ] 1.3 冻结 fixture：CM/CM2（javac23 `--release 8`）+ 真 8 腿（验证两版快照 dance 同构）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1 识别后缀模式（旧值 load + iinc + 单消费方）呈现 `x++` 表达式形；多消费方保持拒绝。
- [ ] 2.2 呈现按决策 2（后缀形，测试钉死）；健康三形零改动。

## 3. 验证与验收

- [ ] 3.1 主锚：immUse/postfixExpr/incDec 恢复；三类 `javac --release 8` exit 0、行为逐行一致（旧值语义精确）。
- [ ] 3.2 零回退：三健康形逐字节不变；负例（`i = i++`、多消费方）仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：归因非新机制、旧值语义精确（行为对比）、零回退实测；关闭 summary.md 登记行。（留 root）
