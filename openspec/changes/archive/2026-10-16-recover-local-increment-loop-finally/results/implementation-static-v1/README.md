# 有界证明与当前验证

新proof只接纳固定36指令、7真实canonical块、2catch-all row的完整NoCall lowering。normal两份iinc读loop phi，handler iinc读独立handler phi；闭合后者的三个实际输入，并证明所有protected操作/self binding均nonthrowing，保留原始异常边/表。不是把三份read ValueId当成相同值。plan owned限16/28/40/46，lead16..22在try外，Tail54..60保留一次索引更新，scope只额外授权唯一已证return28。

静态独审无实质阻断，首轮仅按值handler_ordinal误解引用导致E0614，修正后两真实类正例、实际收费末端预算/取消均通过。完整private回归42 passed，其中9组实际class变体均完成分析后拒绝：副本增量/slot、返回reload、索引步长、除法、额外读取及两种异常边界。部分变体在有限shape判别即拒绝，不声称都抵达单消费者检查。期限保持既有poll组合契约，未单独声称新动态deadline实验。

中间公开入口2测试已验证无bytecode正文与全部36BCI/真实方法身份。两份1430B完整生成Java、未改driver rebuilt-only四次Java8/23回放均通过；最终CLI/自身CI验收另记。计费不变静态结论限非候选不进入新proof扫描，实际旧P5 all-features测试5 passed/1 ignored已通过。

新JSON CLI测试与12个逐目标finally/loop/typed-catch兼容测试均实际通过。原12目标同批link触及1GiB目标磁盘门被终止，clean成功；后续仅拆分剩余目标逐个运行并回收test executable，未提高额度或重复宣称中止批次成功。最终source/CLI闭包取续跑收据，已通过的fmt/Clippy/reader/private/FP/P5费用门取同源前段收据。
