## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、dup-store 机制、jadx 消除解、家族四员定位——实测归档。（root 已完成）
- [ ] 1.1 插桩确认四形状（dance/postfix/putfield/dup-store）在同一门控（"copy … has no proved local assignment"）的同一性；转录存证据。
- [ ] 1.2 重验基线：主线二进制渲染 OP2（两形拒、移位/常量族恢复）；负例（>2 读者）现状拒绝记录。
- [ ] 1.3 冻结 fixture：OP2（javac23 `--release 8`）+ 真 8 腿入 `tests/fixtures/`，README 记编译命令与 SHA。

## 2. 实现

- [ ] 2.1 按决策 1 实现两分支（零读者消除 / 有读者拆语句）；>2 读者保持拒绝。
- [ ] 2.2 家族形状分派结构按 1.1 结论组织（同门控则单机制四分支）。

## 3. 验证与验收

- [ ] 3.1 主锚：condAssign/condAssignOld 恢复、整类拼接 `javac --release 8` exit 0、行为逐行一致（含 main 级联解锁）。
- [ ] 3.2 零回退：移位复合/常量族逐字节不变；负例仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：两分支判据保守性、零回退实测、家族结构合理；关闭 summary.md 登记行。（留 root）
