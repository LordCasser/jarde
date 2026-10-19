# Loop4/5 四原件真实 IR 与架构定位

四个冻结原件 `test` 方法的同轮 Reader→canonical→SSA→NormalFlowView→recover 探针已实际执行并独立复核：1 test passed / 0 failed（一个测试内部遍历两个目标的 debug/no-debug）。物理 class 库存包含 ctor/test/check；IR 探针只分析 test。没有产品算法改动，临时 helper 精确恢复，3,320 项执行前后源码快照相同；默认 Rust 栈、5 GiB 余量/target 1 GiB/每秒守卫，peak target 168,685,359 bytes，finally target absent。

| 目标 | 真实自然循环 | 实际恢复结果 | 待处理边界 |
| --- | --- | --- | --- |
| Loop4 | header5，成员5/11/36，latch36；退出5→42、11→17 | ForHeader init3/update36 已证明；Loop 被保留但17/23/29为uncovered | 外侧条件DAG到同一break42，不能写为空臂或单条break |
| Loop5 | header14，成员14/23/39/88，latch88；退出14→106、39→46 | Try body仅Straight0，catch124；其余11块uncovered | 完整loop外侧体和普通catch中prefix+loop+return-tail均需证明 |

debug/no-debug 的结构/BCI事实相等。Loop5唯一物理row为ordinal0、[0,123)、handler124、typeCP54；block106为[106,124)，表达式到122、末尾return123在范围外。旧候选的范围末尾单条goto证书不适用，不能直接放宽旧证书。

公开路径与探针的 uncovered 集合逐项相同，独立审计复开了每目标8个公开基线单元。**范围限制**：helper 使用 `Sites::empty()`，公开报告传入 `sites_after_array_composition` 的结果；不能由相同fallback推断全部内部Sites或恢复输入相同。根因判断另经 root 对当前 Frame/join/try_level/writer 源码审查，不是独立探针审计的结论。

root 选择两个局部 OpenSpec：`recover-proved-loop-break-branch-bodies` 保留自然循环并证明外侧体；`recover-named-catch-loop-return-tails` 依赖前者，补精确命名行和右边界return尾。复用既有 Frame/If/LoopBreak/Try/Sequence，无新增公共 AST、CFG pass 或依赖。实现、完整源码重编、行为及新产品自身CI尚待验收；本目录不宣称特性已完成。

[完整归档](evidence.tar.gz)含34个payload、6,517,415未压缩bytes；归档1,205,055 bytes，SHA-256 `c585022c33a24f7e3466a5c57f7ada0da02fc8b2c17b9fda5dfd988d7fd7f42b`。root已重开全部成员并比对bytes/mode/SHA，[库存](inventory.json)和[回执](capture.json)记录身份。`preparation/`保留未执行准备原文，实际权威在`actual-launch/`、`actual-run/`；`independent-review/`含复核与方法覆盖措辞修正后的结果，`architecture/`含独立方案，`source-and-tools/`固定当时源码/守卫。核心trace为292,286 bytes、SHA-256 `cddd66ab28b5c4e16536bc5315bee53e32f7c43f47a91f115db16dfcc4888e74`。

完整三方基线及所有原件输入仍以[前序归档](../cf09-loop4-loop5-original-root-v1/README.md)为准。两个特性的完整Java/outer/target/JAR及许可证已逐字节复制到各自`tests/fixtures/<change>/`，manifest保留来源SHA。没有重跑或覆盖旧基线，也没有把JDK8框架55/52边界当作Jarde恢复成功。
