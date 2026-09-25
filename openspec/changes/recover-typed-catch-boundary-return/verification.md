## 本工作树验证记录

- `cargo test --test p3_typed_catch_boundary_return`：5 项通过。覆盖独立 `PlainMultiCatch` 正例、范围外可抛错调用负例、`chooseFinally`/catch-all 拒绝、同步方法和前置跨块显式 monitor 拒绝，以及预算耗尽/预取消不发布部分源码。
- `cargo test --test p3_nested_try --test p3_sync_return --test p3_twr_catch`：nested-try 2 项、sync-return 3 项通过；TWR 1 项通过、1 项因既有 `build.rs` 机制忽略。
- 完整加入 `p3_typed_catch` 的同次命令有 1 项失败：`a_catch_type_of_zero_becomes_neither_a_catch_nor_a_finally`，错误为 `local 1 crosses a quoted fallback region`。为确认是否由本变更新增，将包含相同其它工作树状态的副本复制到 `/tmp/jarde-typed-catch-ab`，只撤销该变更在 `region.rs`/`report.rs` 的同步标志透传、monitor 扫描和 Return 结算，并移除直接测试调用新增的参数；再运行同一失败用例，得到相同失败断言与错误文本。因此该失败在本变更前后均存在，不能归因于本规则；它也不经过该规则，因为 catch-all 在 `exception_edge_accounted` 入口被拒绝。
- `openspec validate recover-typed-catch-boundary-return --strict` 与本变更文件的 `git diff --check` 通过。

以上为实现代理的定向验证；独立整类验收与组合探针复核记录如下。

根代理随后从当前工作树重建 CLI，并独立导出未手改的 `PlainMultiCatch` 完整类及组合探针报告。`choose` 为 `structured/java`、零 fallback，源码含一个命名多重捕获与受保护表达式的正常返回，全部 33 个物理 BCI 都在 source map；`chooseFinally` 仍为 `jre_guard_finally_copy` 等回退，没有错误生成 `finally`。根代理用 `javac --release 8 -g:none -Xlint:-options` 重编原始、JADX（仅移除虚构 package）和 Jarde 三份完整 `PlainMultiCatch` 与原 Runner，三份 class SHA-256 均为 `ac46b05865c086ccb03eea8e1e1890c3bdff1e45771a71c7acc0579c833b8eef`，四条 `java -Xverify:all` 输出逐字一致。[完整报告、源码、轨迹及哈希](../../evidence/java-syntax-2026-09-25/multicatch-finally-return/)另存，不覆盖修前基线。组合探针中 JADX `chooseFinally` 的正常路径 2/4 行重复 cleanup 仍是独立 finally 任务，不属于此变更。

根代理复跑 `p3_typed_catch_boundary_return` 5/5、nested-try 2/2、sync-return 3/3、TWR 1/1（另 1 项既有 ignored）；`p3_typed_catch` 的 catch-all 1 项失败与代理隔离 A/B 的同一基线错误一致。`cargo clippy -p jarde-java --lib --no-default-features` 以既存告警通过；`-D warnings` 因仓库既存告警及此次窄接线使 `region::recover` 达到 8 个参数而失败。该 lint 债务不应驱动本轮引入新的包装实体或混入大范围签名重构。
