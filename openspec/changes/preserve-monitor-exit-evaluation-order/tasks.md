# Tasks

- [ ] 1. 插桩定夺（root 预审计已收窄）：复核 InnerMonitor 无 returns 字段与 build.rs ~14525 外层追加路径；在"内层 returns 字段"与"内层 temp 赋值"两案间按最小 diff/零回退面定夺并记录求值区间-配对 exit 事实
- [ ] 2. 实现求值次序不变量（按 1 的定夺；不放宽判据）
- [ ] 3. 对照测试：NL 判别锚 `nY`；SR 往返一致；#75 monitor corpus 逐字节零回退；新增 fixtures 走双 javac 协议
- [ ] 4. 全门禁 + 分逻辑提交（不 push）
