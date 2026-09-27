# CF-08 单出口循环臂汇合：主线独立验收

主线合入提交 `a6df342c` 后，root 审阅生产差异：`loop_arm_join_source` 仅接受一个完整 `Region::Loop` 与一个 `Region::Straight` 子臂，核对自然循环 owner、同路由 BCI 范围、已访问 scope、唯一来自内层分支的正常入口与唯一到内层 join 的出口；原有尾段入边及父边界证明仍执行。Builder 的中间值桥候选缩回它实际支持的双 Straight 子臂，没有替循环臂合成值。此变更没有放宽声明规划或引入新 Region 类型。

root 从主线独立构建 CLI（SHA-256 `4304369e9cb509450de517ec6f968272381021ab5b9ec9ed101a29c1d86d1d84`），恢复固定 class `542c856a156103f8c4e20372fcbeb3bc767b0c706f3270d73d5ec6dddfefdc2a`；完整 Jarde 源 SHA-256 为 `b249d4214bccd4b3968cd66582d26cdb61bed40f8d231e95b128291b06fd2999`，与实施分支归档相同，没有 `@bytecode`。原源码、固定 JADX 源码与主线 Jarde 源码分别以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行，三者逐行同为 `4 / 13 / 11`。

主线 `p3_loop_arm_join` 4/4、相邻 `p3_forward_join` 3/3、`p3_switch_forward_join` 3/3、`p3_loop_transfers` 5/5 均通过。`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check`、`openspec validate recover-loop-arm-join-continuation --strict` 均通过。root 重编原 `NotIndexedLoop` 并用同一 CLI 恢复，仍保留 `@bytecode`，因此 CF-08 的带效果双出口/循环后局部汇合仍是已证差距，本首片不将整单元标为追平。
