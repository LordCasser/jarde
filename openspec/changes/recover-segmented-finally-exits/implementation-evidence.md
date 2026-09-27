# 实施证据（2.x–4.2）

五行证书仅在异常表恰有两条同目标具名行、两条同目标 catch-all 行和一条 catch-body catch-all 行，且 ordinal、范围、handler 均匹配时进入。两段实际保护范围由 `Shape::SegmentedFinally.segments` 保存；`Plan.body()` 的连续区间仅供有界 Region 遍历，中间早退副本不被误认作受保护指令。Guard 对每个物理 BCI 检查异常行覆盖，比较四次调用的成员、入口 `this` 接收者与 SSA 唯一消费，核原 Throwable 从 handler store 到 load/throw 的值身份，并检查所有 canonical 边和外部入口。固定原 class 与独立验收类的目标方法都输出一个 `try/catch/finally`。

`p3_shared_join_finally.rs` 核固定原 class 和验收类的十个 canonical 块各有唯一 owner，34 个物理 BCI 均能反查源码；Guard 定向测试核五个 ordinal、两段、四份清理、早退返回、两个 transfer、后续 join 以及行缺失、换序、扩围、四个 verifier 有效负例和额外外部入口图负例。预算、取消及输出预算耗尽时文本和 source map 均为空。源码构建沿用原 Builder checkpoint；早退路径保留 `return`，其物理清理调用以来源折叠，两个正常转移和异常保存/重抛同属外层 `try` 的来源。

独立 `acceptance/replay.sh` 已核目标方法同 pinned class 的全部 BCI/opcode 与五行表逐项相同，并把原验收 class、pinned JADX 与 fresh Jarde CLI 的完整类用 `javac --release 8` 重编、`java -Xverify:all` 跑七条路径；两组逐字轨迹比较相同。`negatives/replay-neighbors.sh` 在 `JARDE_RANGE_CONTROL_RECOVERED=true` 下核原四个 verifier 有效负例仍拒绝错误归并，未扩围正对照恢复为一个 `finally`，扩围原 class 的二次清理轨迹与冻结文件相同。额外 `external-entry.class` 在 verifier 下有效，Jarde 安全拒绝。

原冻结 probe 的共享 `invoke(int,String)V` 另有 BCI 66 未恢复块，导致其完整类异常路径不能用来验收 Test13；本次没有混入通用修复。见 `acceptance/helper-debt.md`。

合入 `origin/main` 的 switch/catch 修复后，`cargo test -p jarde-java --tests --locked` 全绿，覆盖 Test12、共享 catch-all、普通 finally、TWR/monitor 和主线新增的 catch/switch 定向测试；`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-segmented-finally-exits --strict`、`git diff --check` 均通过。fresh CLI 在合并后重新构建，三方七路径与负例复放再次通过，输出分别位于 `/tmp/cf16-segmented-acceptance-after-merge` 和 `/tmp/cf16-segmented-negative-after-merge`。`cargo clean --target-dir /tmp/cf16-test13-segmented-agent-target` 移除了本任务专用 target 的 21707 个文件；目录已不存在，未触及共享 target。4.3 留给 root 独立审阅。
