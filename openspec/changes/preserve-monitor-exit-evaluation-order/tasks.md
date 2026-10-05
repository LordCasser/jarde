# Tasks

- [x] 1. 插桩定夺（root 预审计已收窄）：复核 InnerMonitor 无 returns 字段与 build.rs ~14525 外层追加路径；在"内层 returns 字段"与"内层 temp 赋值"两案间按最小 diff/零回退面定夺并记录求值区间-配对 exit 事实
      → 三条预审计断言逐条复核成立（行号已漂移，机制为准）；定夺**(a) 内层 `returns` 字段**（(b) 在同一注入点之外还要新造合成命名/类型/声明/账本四面）；求值区间与闭合时机实测见 [instrumentation.md](instrumentation.md)
- [x] 2. 实现求值次序不变量（按 1 的定夺；不放宽判据）
      → `guard.rs`：`InnerMonitor.returns` + `expression_inside`（整条表达式链落内层 body 才写）；`build.rs`：`NestedPairBraces.returns` + 闭合前渲染 + 走中闭合 fail-closed；不放宽任何既有判据
- [x] 3. 对照测试：NL 判别锚 `nY`；SR 往返一致；#75 monitor corpus 逐字节零回退；新增 fixtures 走双 javac 协议
      → `tests/preserve_monitor_exit_evaluation_order.rs`（3 非 ignored + 1 ignored 编译-运行重放）；fixtures `tests/fixtures/preserve-monitor-exit-evaluation-order/`（v8 = javac 23.0.1 `--release 8`，v8-javac8 = Corretto 1.8.0_432）；591 个既有 fixture class 全量渲染对照：585 逐字节不变、6 变更全为本片目标形
- [x] 4. 全门禁 + 分逻辑提交（不 push）
      → 门禁与提交见 [verification.md](verification.md)

