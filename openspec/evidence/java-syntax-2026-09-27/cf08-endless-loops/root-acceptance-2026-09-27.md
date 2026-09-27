# CF-08 双出口网关主线验收

主线包含 CF-06 后合入 `5b2656f4`。root 从该主线独立构建 CLI，SHA-256 为 `95295d3e1688077b9cde0620739178525fb95b77084425540452b79623b791d1`。用它运行 `replay-simple.py --require-jarde`，所得 Jarde 完整源码 SHA-256 为 `9fff4ace10f7da20544a4cd5852daa73d21c10f2f4e4c54c13ef9864d880c2e0`，与实施分支归档的 `simple-after` 完全相同，且不含 `@bytecode`。原 class、固定 JADX、Jarde 三份完整 Java 8 源码均重编成功，`java -Xverify:all` 对六个输入逐行同为 `0,1,2,3,3,3`，`limit=4` 不再超时。

同一主线 CLI 重放 CF-07 的八行 `0,10,39,63,9,3,-1,3` 和 CF-09 的 `Grid`、`OuterContinue`；原/JADX/Jarde 每组输出逐字一致且三份源码均能 Java 8 重编。root 另运行 `p3_loop_exit_gateways` 3/3、`p3_loop_terminal_return` 4/4、`p3_short_circuit_chain_controls` 1/1、`p3_loop_transfers` 5/5；`cargo fmt --all -- --check`、`openspec validate recover-proved-loop-exit-gateways --strict` 和 `git diff --check` 通过。定向测试检查 BCI 4/7/12/15/18/21/24/25 来源、双网关所有权、四类不成立网关的保守引用及预算/取消原子停止。

这只验收整数双出口首片。`TestNotIndexedLoop` 的外层条件与循环后 slot 2 汇合仍引用整个方法；没有据此把 CF-08 整单元标为追平。root 验收所用输出在 `/tmp/jarde-cf08-root-combined-3`、`/tmp/jarde-cf07-root-combined-3`、`/tmp/jarde-cf09-root-combined-3`；保存 CLI 后清理独立 Cargo target，释放 1.7 GiB。
