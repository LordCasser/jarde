# CF-16 Test13 实施复放

`TestTryCatchFinally13$TestCls.java` 是独立验收夹具，class SHA-256 为 `2ba5aeb0aea209f11fb9f014c40ea7fa3da702ae5af11f844fc98eb9ef267ac8`，由 `javac --release 8 -g` 编成 class major 52。它保留冻结源的 `test(I)V` 正文，只把原 probe 的共享 `invoke` 辅助方法拆成三个单分支 helper。`replay.sh` 每次重新编译并逐字比较 class，同时将 `test(I)V` 的 BCI/opcode 序列及五条异常表行同 pinned 原 class 比较；因此辅助方法的改变不扩大 Test13 证书输入。原冻结 class 与本夹具的目标方法还由 `p3_shared_join_finally.rs` 分别核验结构、owner 和来源。

先在当前 checkout 使用专用 Cargo target 构建 CLI，再复放完整类：

```sh
cargo build -p jarde-cli --bin jarde-cli --locked --target-dir /tmp/cf16-test13-segmented-agent-target
openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/acceptance/replay.sh /tmp/cf16-test13-segmented-agent-target/debug/jarde-cli /tmp/cf16-test13-acceptance-output
JARDE_RANGE_CONTROL_RECOVERED=true openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/replay-neighbors.sh /tmp/cf16-test13-segmented-agent-target/debug/jarde-cli /tmp/cf16-test13-negative-output
```

`replay.sh` 用 fresh CLI、pinned JADX 和原 class 产生完整 Java 8 类，三者 `javac --release 8`、`java -Xverify:all` 七条路径及逐字轨迹比较全部通过。负例复放保留原四个 verifier 有效变体的安全拒绝；`range-control` 是未扩围的正对照，实施后应有一个 `finally`。扩围变体的原 class 仍产生两次清理并抛 `AssertionError`，Jarde 不把它折叠为一次。原冻结 `negatives/replay-neighbors.sh` 默认仍按实施前正对照拒绝状态验收；环境变量只用于本次实施后的复放。

另有受控 CFG 负例 `external-entry.class`：`mutate_external_entry.py` 只把正常清理后的 BCI 41 `goto 63` 改为 `goto 15`，从清理块回流到第二段受保护正文。`VerifyEarly.java` 用 `java -Xverify:all` 验证 class 并只执行早退路径，避免陷入变体故意制造的循环；Guard 和完整 Jarde 输出均拒绝为它生成 `finally`。它验证外部入口不能借两段正文的遍历 envelope 偷渡进 owner 集合。

原 probe 的共享 `invoke(int,String)V` 在 BCI 66 存在独立的 `UncoveredBlocks`，fresh Jarde 完整类虽可重编，但缺失其异常抛出路径。原 probe 复放记录在 `/tmp/cf16-test13-segmented-replay`，其四条正常路径与原 class 一致，三条异常路径因这个 helper 失败。该辅助方法的恢复债务单独记录于 [helper-debt.md](helper-debt.md)，本变更没有调整产品的通用分支恢复。
