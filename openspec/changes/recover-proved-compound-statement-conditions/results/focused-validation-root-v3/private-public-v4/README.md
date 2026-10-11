# Compound condition physical-origin test candidate v4

以私有 v3 候选为基线的单点类型修正：`assert_default_compound_if_source` 的 BCI 参数从 `&[u16]` 改为 `&[u32]`，匹配 `SourceMap::direct_of_bci` 的参数类型。其余测试与断言逐字保持不变。仅私有候选；未编译或运行。
