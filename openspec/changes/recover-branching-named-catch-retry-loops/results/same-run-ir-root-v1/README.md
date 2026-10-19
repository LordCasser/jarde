# Loop2 同轮 IR 实测（2026-10-19）

状态：root 已运行并审查轨迹；仅定位根因，不是修复验收。`capture.tar.gz` 共 9 个 payload，SHA-256 `5e47b73b21c2cba5e6e2c85594680b340d09453314f95e5452604fcaf555d1e1`；各成员 bytes/mode/hash 已复开，见 inventory.json。

- 冻结准备：[v1 拒绝与 v2 静态审核](../../../../evidence/java-syntax-2026-10-19/cf09-testbreakinloop2-ir-probe-preparation-root-v1/README.md)；[完整原件基线](../../../../evidence/java-syntax-2026-10-19/cf09-testbreakinloop2-original-root-v1/README.md)。
- 实际命令为裸 Cargo 精确单测，1 passed / 0 failed / 0 ignored；两份原始 debug/no-debug class 各走实际 Reader→IR→Region。构建默认栈、guard 无事件，peak target 175,729,232 bytes，finally cargo clean 成功。
- `launch/execution.json` 记录外部输入前后 pin、临时源码与 env；`run/execution.json` 保存源码快照和两命令原始流。临时插桩只增加诊断，最终精确恢复原 region.rs SHA `5da4d9f40e0892ac134bb42fb1508fffad090b92338a86b9403efd47b7977510`，移除临时模块，target absent。

两模式真实 normal edges 均为 0→34、9→19/17、19→0，17/34 无 normal successors；natural-loops=none。Catches 为一条 Named([13])、handler9、parameter2、join34、protected_end6、lead(0,0)，无 inner/transparent；top Frame 无 boundary/loop/try。

`retry_loop_facts` 实际首拒绝为 check_20：单 handler 的 normal successor 必须唯一回到 header，但9是条件分支。未获得 retry Frame 后，handler19沿31的 goto0再进 Try，最终树在外 Try 的 catch If else 中嵌入第二个 Try；owners `[0,9,17,19,0,9]`，另有出口34。最终 overlap gate 正确拒绝 BCI0。原始 stderr 的 debug 部分在815–867、980–1158、1551–1554行；no-debug重复同形。

结论：扩展既有异常 retry 证明，保留单物理保护块，严格加入条件 handler、throw叶和retry叶的闭合证明。不是自然循环发现缺失；不能删除 overlap 检查或全局去重，也不能仅放宽 check_20 而保留单 handler owner/无条件尾 continue 假设。当前无产品源码变更。

独立只读复核见 independent-review.json：实际 run/launcher pin、四条 raw 流、9 个归档成员、源码恢复及两模式轨迹逐项通过。首次审计将 README 的 owner 列表按含空格字面量匹配导致假失败，按原文格式复核后通过；未重跑探针或修改实际原流。
