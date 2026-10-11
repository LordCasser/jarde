# Compound Builder Stop tests — private candidate

这份候选只给 `crates/jarde-java/src/build.rs` 增加测试，不含生产逻辑修改。它复用现有 `fixture_value_attempts_with_tree_observing` 的真实 IR/SSA/Region 回调，并向回调补传现有 method/class facts；没有复制 reader 或 IR 装配。新测试用的是 OpenSpec 锁定的真实 `CompoundStatementConditions.class`，方法 `test(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;`，并核对物理测试 BCI 26、37、48、59。

测试用私有 Builder literal 构造完整相关计划，literal 连同声明和局部赋值计划约 140 行；这样可以直接调用真实 `prepare_compound_condition` 与 `Builder::region(CompoundIf)`，而不靠 BCI 猜测哪个阶段触发了 Stop。两个预算用例各先以高预算跑完同一阶段，再以实际成功 usage 减一触发 `AnalysisSteps` Stop，断言 Stop 的维度、limit、区域内 BCI 与实际 usage。Builder 中预置非空语句、binding、binding plan、deferred 项、presentation/snapshot/name 哨兵，验证停止后状态原样恢复。

取消用例先完成真实 preflight，再取消既有 `CancellationToken`，随后直接调用 CompoundIf emitter 并验证 `Cancelled` 与状态恢复。这是确定性的“分析后、发射前取消”，不声称覆盖异步运行中途取消。`Builder` 阶段没有 Program/source-map 发布点；本测试验证 emitter 返回 Stop 且无部分 AST/binding 状态，整次 build 不生成 text/map 的外层行为仍需 root 的 guarded runner 覆盖。

候选尚未编译或执行。没有运行 Git、Cargo、JDK 或修改主工作区。

- base SHA-256: `c55ac016b402fb54ef4f6072dc54ee7d55809ea3d067e1d46c5eb30136afe392`
- candidate SHA-256: `08b625a2ea2e94b4ee08abf920435d7c18612da83ea06359362bd57e5183e373`
- patch SHA-256: `04515f79b39c513980320105dc822781cb09c3c8c946691eb53c164c7aa42c4f`
