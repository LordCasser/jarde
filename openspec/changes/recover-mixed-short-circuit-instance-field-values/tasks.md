## 1. 扩展消费者前先解决已观测到的 Region 边界

- [x] 1.1 跟踪规范 BCI 20 在 Region 构造、完整所有权校验和报告中的流转；[Region/SSA 证据](../../evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/region-trace.md)确认消费者锚点首先漏掉 `putfield`，其后的通用 `if` 树真实重叠，所有权校验本身正确。
- [x] 1.2 为已观测到的拒绝和成功候选各添加永久定向断言：不得放宽所有者检查；候选证明失败时仍报告真实拒绝原因，成功时只认领 BCI 20/25 一次。
- [x] 1.3 完成审查后，使用最新 CLI 重跑冻结的源文件/类文件/Runner；保留整类文本、JSON 报告、Java 8 重编结果及全部 16 条 `-Xverify:all` 轨迹，且不得覆盖带哈希的稳定快照基线。

## 2. 证明并发射带接收者的字段消费者（仅在所有权审查通过后）

- [x] 2.1 仅为普通 `putfield Z` 扩展私有消费者证据：匹配精确的 FieldPlan 身份、栈深度 0 的接收者、栈深度 1 的 Phi，并在字段写入还读取接收者操作数的情况下证明 Phi 只有这一个值使用。
- [x] 2.2 证明接收者精确的字段 owner 类型、完整 SSA 所有权、单次求值以及没有第二个发射消费者；拒绝未解析的接收者图和副作用关系。
- [x] 2.3 仅在接收者和条件表达式都已完整证明后，复用 `FieldAssign { receiver: Some(..), value }`；验证接收者先于 RHS 求值、b/c 分支保持惰性、来源完整、null 故障发生在 RHS 之后，以及预算中止时原子回退。
- [ ] 2.4 添加永久负例，覆盖未解析/重叠的 Region 所有者、第二个 Phi 消费者、接收者的第二次使用/所有者、字段身份或描述符不匹配，以及复合字段更新。

## 3. 独立验收

- [x] 3.1 使用 Java 8 编译原始/JADX/Jarde 完整类呈现，并在 `-Xverify:all` 下执行全部 16 条路径；对照冻结的原始类，核对执行结果、字段值、b/c 调用次数、接收者调用次数和 null 故障时点。
- [x] 3.2 运行受影响的定向测试、格式检查/适用 lint，以及 `openspec validate recover-mixed-short-circuit-instance-field-values --strict`；记录剩余边界并清理私有 Cargo target。
