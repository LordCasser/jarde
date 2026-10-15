# Proposal

## Why

`CanonicalBlock.protected()` 承诺发布异常保护范围 ordinal，但当前初始化为空且从未填充，所有发布块恒空；既有遍历断言因此可以无条件通过。融合节点可能只在部分原始块受保护，公开契约还需要明确这一含义，避免被误用为整个节点都在同一范围内。

## What Changes

- 发布 canonical block 的真实原始成员块与物理异常保护范围相交的 ordinal，按声明顺序去重；保持 clone path 与物理 origin。
- 明确融合后的字段表示至少一个实际成员块受保护，不能用非连续块的包络区间或整节点覆盖假设推导。
- 与从异常边得出的 handler-row/throw-site 事实分别定义，不能借该字段扩充异常输入。
- 对新增扫描、ordinal 存储及融合/装配副本保持现有计费与停止契约，补充非空、负向、融合、clone 和预算回归。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `jvm-ir`：补充公开 canonical block 的异常范围事实与真实成员、融合、clone、停止语义。

## Impact

主要涉及 `crates/jarde-jvm/src/canonical.rs`、既有内部 canonical 回归及 `tests/p3_method_ir.rs`；文档和当前 accessor 保留。前置为完整 raw CFG、已验证指令边界与既有 normalization 流程。未发现当前生产源码恢复依赖此 accessor，因此不宣称已修复运行行为或扩大 Java 恢复能力；不改变 handler-row/effect/Frame/SSA 异常边模型，不新增 API、依赖或 crate。该问题独立于正在实施的 Try/List 准入，工具与共享实现操作按 coding root 窗口串行。
