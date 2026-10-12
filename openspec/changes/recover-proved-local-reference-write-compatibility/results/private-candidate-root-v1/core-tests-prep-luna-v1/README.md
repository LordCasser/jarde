# 私有 core 对抗测试候选 v1

这是一份只含 `#[cfg(test)]` 改动的私有补丁，基于 root 应用并格式化后的 production v4 快照；未改共享文件。

- 基底 `build.rs` SHA-256：`56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d`
- 候选 `build.rs` SHA-256：`c3bc8333cf3aedd0fd19b75b416cf6425d813920b84e180ea9db7bbb3db6cff0`
- `build.tests.patch` SHA-256：`db22d7f42246a79fa3f8d1ab1c568c7c6fe725c6a0c941ab8597be90627e4d53`
- 文件：`build.rs.base`、`build.rs`、`build.tests.patch`

测试复用现有 `iterator_test_context` 的真实分析、regions 与 split-aware reuse；仅给 `IteratorTestContext` 增加保存 regions 的字段，以按生产路径重建 `region_paths`、cleanup 集及 `slot_uses`。fixtures 直接从现有 `tests/fixtures/p3-local-reference-write-compatibility/debug/javac23/` include，不在这份补丁中复制或修改 fixture。

四个测试覆盖：真实 `noCastAssign` 的窄 String 首写、direct Object 后写与 Object Return 正向证明；对同一份 SSA/SlotUse 做访问遗漏、重复、path=None 和无读扰动并要求 `ConflictingWrites`；真实 CheckCast consumer（另用 helper 的 Object 返回参数只为越过更早的 return-type 门并隔离 consumer 检查）、异常处理写拒绝、以及真实 slot reuse 中旧 lifetime 的同槽读取与新 target identity 隔离；以及动态测得完整 proof usage 后以 `usage - 1` 在真实 Return consumer BCI Stop、预取消 Stop 且不发布 decision/cache。

对抗扰动只修改传给 helper 的 SlotUse 列表，底层 SSA、regions、reuse 与物理 Object cache 输入仍来自真实 class；因此它验证完整性 reconciliation，但不声称这些原始 class 本来含 `reuse=None` 的访问。候选也没有 fake cache / None-owner 测试；slot-reuse 控制只验证真实、已归属的另一 lifetime。没有构造或宣称真实 class 存在更早的未归属 Object Store。

未运行 Rust/JDK/JADX、构建、格式化或测试。此补丁仅待 root 审读后再应用和实跑。
