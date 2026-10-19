## 1. Evidence and architecture

- [x] 1.1 归档并独立复核四原件真实 IR 探针、精确恢复和清理收据；以闭合 trace、输入 SHA、1 pass/0 fail、debug/no-debug 拓扑相等及公共报告一致为验证。
- [x] 1.2 固定完整 Loop4 Java/outer/target/debug/no-debug 输入及 manifest，验证与三方基线逐字节一致，不能裁剪 check 或依赖边界。

## 2. Recovery implementation

- [x] 2.1 Luna 提交私有补丁实现闭合 break 分支体证明、候选 join 与精确 owner 扩展；主 agent 审查后运行真实 Reader 测试，验证 owners17/23/29、ForHeader init3/update36、break origins26/33、无重复或 uncovered。
- [x] 2.2 Luna 补充外部入口/额外出口/回边/未解释异常/switch/已占用或父 scope 负例及预算、预取消、晚期停止；主 agent 运行并检查真实拒绝、完整状态回滚及用量不回退。

## 3. Acceptance

- [ ] 3.1 冻结新 CLI，执行完整 Loop4 三方对照：8 API、全部方法/字段、完整生成源码重编、双 JDK 隔离 VM、实际 check、错误 CodeSource 控制、物理 source maps；独立重开 raw 后才能记验收通过。
- [ ] 3.2 执行完整工作区和该产品提交自身的 CI，保留旧关键 gate 与 Loop2/ComplexIf/finally 回归；比对实际 named outcomes、source bindings 与退出码，不借用前版 CI。
- [ ] 3.3 同步主 spec、严格验证并更新 handoff/71 单元账本中的精确局部状态；提交推送 main 后检查工作区干净、target 不存在，保留未完成 CF09 范围。
