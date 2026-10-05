## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、jadx if 嵌套解、行为等价验证、循环标签判别——实测归档。（root 已完成）
- [ ] 1.1 插桩定位 "intermediate join, bridge and outer conditional do not form a closed value" 发出处与 join/桥事实载体；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 LB（标签拒、无限循环恢复）；负例探针（join 不可证形）现状拒绝记录。
- [ ] 1.3 冻结 fixture：LB（javac23 `--release 8`）+ 真 8 腿入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1/2 实现：非循环标签 break 的 if 嵌套等价呈现（跳过可证才呈现）；循环标签通道互斥判定零改动。
- [ ] 2.2 MVP 两层嵌套先行；三层按实测记录（Open Question 2）。

## 3. 验证与验收

- [ ] 3.1 主锚：labeledBlock 恢复 if 嵌套形、整类 `javac --release 8` exit 0、行为 `100/110/111/6/5/104` 逐行一致。
- [ ] 3.2 零回退：循环标签与无限循环归一化逐字不变；负例仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：等价性由数据流可证性驱动（非语法猜测）、零回退实测；关闭 summary.md 登记行。（留 root）
