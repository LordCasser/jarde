## 1. 取证与基线（root 已完成大半）

- [x] 1.0 两形拒绝、jadx 双解、同型全恢复、机制（int 化）——实测归档。（root 已完成）
- [ ] 1.1 插桩定位拒绝发出处（"bitwise operator … operands presented as"）与操作数类型呈现载体；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 BW（两形拒、其余恢复）；负例探针（真混合算术）现状拒绝记录。
- [ ] 1.3 冻结 fixture：`BW`（javac23 `--release 8`）+ `BW8`（真 javac 8 腿，验证两版 int 化机制一致）入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1 实现保守回投（生产链全布尔 + 消费全位运算/布尔出口）；非布尔消费不回投。
- [ ] 2.2 呈现按决策 2（与 jadx 同构的布尔形）；出口自然消解或如实保留，测试钉死。

## 3. 验证与验收

- [ ] 3.1 主锚：`andNot`/`mix` 恢复、整类 `javac --release 8` exit 0、行为一致（双腿）。
- [ ] 3.2 零回退：同型位运算逐字节不变；负例仍拒；corpus 差异类仅为混合布尔位运算形。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：回投判据保守性（数据流充分而非语法猜测）、零回退实测；关闭 summary.md 登记行。（留 root）
