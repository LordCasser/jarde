# CF-16 固定 Test13 分段 finally：主线独立验收

固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。本次从主线 `155ad864` 重新构建 Jarde CLI，而非复用实施代理的二进制。冻结固定 class 的 SHA-256 是 `7f5dc8b6e83912afbf8216ae26aba8dbba8f43c9eaaba8f28a966091e9c55d51`；独立完整类验收夹具是 `2ba5aeb0aea209f11fb9f014c40ea7fa3da702ae5af11f844fc98eb9ef267ac8`。复放脚本重新编译并逐字校验后者，逐项确认两个 class 的 `test(I)V` BCI/opcode 和五条异常表行相同。夹具只拆分了原 probe 中另有恢复缺口的共享 `invoke` 辅助方法；该缺口见 [helper-debt.md](acceptance/helper-debt.md)。

Guard 仅接纳连续 ordinal 的两段同目标具名异常行、对应的两段 catch-all 行和独立 catch-body 行；逐 BCI 核对真实覆盖范围，并证明四份清理是同一成员、入口 `this` 接收者的唯一消费，原 Throwable 从保存到重抛保持 SSA 值身份。所有 canonical 边必须落在已证明的段、早退、两个清理后汇合或异常 handler 中，额外外部入口拒绝。Region 用既有 `Try` 所有权和受限遍历容纳两个实际保护段；连续 `Plan.body()` 只是遍历 envelope，不赋予中间清理保护范围。Builder 沿用既有 finally checkpoint，把早退路径保留为 `return`，四份物理清理只输出一个 `finally`，其余 BCI 记为来源。

主线重新运行 [acceptance/replay.sh](acceptance/replay.sh)，输出在 `/private/tmp/jarde-test13-root-acceptance`：原 class、pinned JADX、Jarde 三份完整 Java 8 源码均重编并经 `java -Xverify:all` 运行，四条正常路径及三条异常路径逐字一致，每条路径 `finallyCount=1`。固定 class 和独立夹具的目标方法测试均有唯一结构和块 owner，34 个物理 BCI 都有来源。主线重新运行 [negatives/replay-neighbors.sh](negatives/replay-neighbors.sh)，输出在 `/private/tmp/jarde-test13-root-negatives`：四个 verifier 有效错误近邻不生成 `finally`，未扩围的正对照生成一个；额外的 verifier 有效外部入口类也拒绝。扩围负例原始运行两次清理并抛 `AssertionError`，Jarde 保守拒绝，固定 JADX 的一次清理结果不作为目标语义。`rethrow-changed` 保留不完整标记而非整个方法完全拒绝，符合“不能发布错误 finally”的边界。

根代理另行运行 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-segmented-finally-exits --strict` 和 `git diff --check`，全部通过。Test12、共享 catch-all、普通 finally、TWR/monitor 及 switch/catch 回归包含在测试中。该证据只验收固定 Test13 的五行形态；`FinallyOnce.handled/escaping`、原 probe 的共享辅助方法及 CF-16 其它编译形态仍各自单列，整单元不标作追平。
