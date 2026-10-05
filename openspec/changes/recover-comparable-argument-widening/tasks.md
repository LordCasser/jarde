## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、同因判别（与 CharSequence）、装箱参与、javadoc 集初枚举——实测归档。（root 已完成）
- [ ] 1.1 javadoc 逐行核对 9 行封闭集（String + 8 装箱；确认无遗漏的 java.lang 实现者），转录依据存证据。
- [ ] 1.2 重验基线：主线二进制渲染 RG（`callGen`/`callGen2` 拒、其余恢复）；负例探针（Object→Comparable）现状拒绝记录。

## 2. 实现

- [ ] 2.1 按决策 1 落 9 行表（落点与 CharSequence 片并列，结构按 Open Question 1）；命中走 `cast_argument`（决策 2）。
- [ ] 2.2 不动既有三条通道；不做 java.lang 之外的 JDK 实现者。

## 3. 验证与验收

- [ ] 3.1 主锚：`callGen`/`callGen2` 恢复、整类 `javac --release 8` exit 0、行为一致。
- [ ] 3.2 零回退：三条姊妹通道测试全绿；负例仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：javadoc 集核对、表封闭不外推、零回退实测；关闭 summary.md 登记行。（留 root）
