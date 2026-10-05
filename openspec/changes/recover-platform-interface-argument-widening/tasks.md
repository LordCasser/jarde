# Tasks

- [ ] 1. 复核 `snapshot_header_chain_widens_with` 的取 header 顺序；把"目标名命中"前移到目标 header 读取之前，并确认只影响单边（快照内源→平台目标）
- [ ] 2. 对照测试：CP.byAnon `[10,20,30]`、AH/AC `[a, bb, ccc]`、AN 两-sided 正例零回退、无关系负例仍拒；fixtures 双 javac 协议
- [ ] 3. 全门禁 + 分逻辑提交（不 push）
