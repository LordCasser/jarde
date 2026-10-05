## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、三通道不覆盖、javadoc 实现者族、jadx 有解——实测归档。（root 已完成）
- [ ] 1.1 实现者全集核对：以 **release 8 javadoc** 逐一核对 CharSequence 实现者（String/StringBuffer/StringBuilder/CharBuffer；Segment 是 9+ 勿入），转录依据存证据。
- [ ] 1.2 重验基线：主线二进制渲染 `SB`（`join` 拒、其余恢复）；负例探针（String→Runnable、Integer→CharSequence）现状拒绝记录。

## 2. 实现

- [ ] 2.1 按决策 1 落表（四行封闭；落点按 Open Question 1，实现者定并说明）；命中走 `cast_argument`（决策 2，零第三种呈现）。
- [ ] 2.2 不动 `DIRECT_EDGES` 集合表与 Throwable 通道。

## 3. 验证与验收

- [ ] 3.1 主锚：`SB.join` 恢复 0 引注、整类 `javac --release 8` exit 0、行为一致。
- [ ] 3.2 零回退：集合/Throwable 扩宽既有测试全绿；负例两条仍拒；corpus 双腿扫描 diff 为空。
- [ ] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
- [ ] 3.4 root 独立复核：实现者全集与 javadoc 一致、表封闭不外推、零回退实测；关闭 summary.md 登记行。（留 root）
