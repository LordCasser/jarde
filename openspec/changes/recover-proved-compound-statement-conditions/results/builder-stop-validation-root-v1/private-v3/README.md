# Compound Builder Stop tests — private v3

基于已应用并格式化的 root 源 SHA-256 `82757c6ee1f8257507f33c26dc5ed7f965b19b240736cda71b6e02aadd5adf5d`，只改测试。

修正 Builder 测试使用完整 `recovered.regions` 构造 declarations 和 local assignments；没有清理或绕过 `declarations.incomplete`。测试从 `region_paths` 查出 root test block 的真实 lexical path，并断言此 fixture 的 CompoundIf 是 top-level `root[0]`，也核对第一个 Region 确实是同一 root BCI 的 CompoundIf。emitter 以该真实 path 调用。

高预算 emission 现在必须新增结构化 `StmtKind::If`、没有 fallback，并且分析步数高于 prepare 完成时；仅 `Ok(())` 不再算通过。proof/emission 的 usage 与 Stop 诊断日志保留。

候选未编译或执行；没有修改主工作区，也没有运行 Git、Cargo 或 JDK。

- base SHA-256: `82757c6ee1f8257507f33c26dc5ed7f965b19b240736cda71b6e02aadd5adf5d`
- candidate SHA-256: `4146065db4f5f17b473384b74903debafee130b625f6ede0b4a9ccaa37acf437`
- patch SHA-256: `0604e54c8321abf496f2ae3f9e929174c698a77911b57d8b57bb4c598e4f81a5`
