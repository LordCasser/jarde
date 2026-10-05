# Tasks

- [ ] 1. 插桩定夺：定位嵌套 synchronized pair 的 `returns` 归属为何落到外层 body（build.rs synchronized return 形/guard nested-pair 计划）；记录内层求值区间与 exit BCI 的配对事实，回答"return 内置 vs temp 外置"哪个落点最小
- [ ] 2. 实现求值次序不变量（按 1 的定夺；不放宽判据）
- [ ] 3. 对照测试：NL 判别锚 `nY`；SR 往返一致；#75 monitor corpus 逐字节零回退；新增 fixtures 走双 javac 协议
- [ ] 4. 全门禁 + 分逻辑提交（不 push）
