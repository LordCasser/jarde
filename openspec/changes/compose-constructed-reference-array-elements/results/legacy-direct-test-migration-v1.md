# 旧 direct-family 断言迁移记录

本次只调整 `tests/p3_heterogeneous_array_initializers.rs`。依据冻结的 `results/legacy-family-cli1-root-v1/manifest.json`（SHA-256 `a46bf17b6ef31aa51b530598175399cf38ba068b4dfd7dc00908cac82afa4a50`）及其 `fixture/direct/{javac8,javac23}/render-Main` 原始 JSON stdout；CLI SHA-256 为 `69a4a4bf86ca3cda412ea5ca6fba4dda24e84253daca781ddfec2da31b4d8495`。两腿 Main 报告 SHA-256 分别为 `f1946eb8b7c1f8d11ddbf51216b6bff6105144523f7dc673b754953638e805a6`、`430a92a5287fd4e7fba3b7974a5e2ff2cbe115a6a6365d60f69c2a9bad16cb67`。两份报告中的以下方法数据一致。

| 方法 | 构造记录（class / new / dup / init / argument） | `aastore` BCI |
| --- | --- | --- |
| `sequenceDirect` | `String` / 6 / 9 / 17 / 14；`StringBuilder` / 23 / 26 / 34 / 31 | 20、37 |
| `collectionDirect` | `ArrayList` / 6 / 9 / 20 / 17；`HashSet` / 26 / 29 / 40 / 37 | 23、43 |
| `throwableDirect` | `IllegalStateException` / 6 / 9 / 17 / 14；`IllegalArgumentException` / 23 / 26 / 34 / 31 | 20、37 |
| `ownTwoHopDirect` | `DerivedA` / 6 / 9 / 14 / 11；`DerivedB` / 20 / 23 / 28 / 25 | 17、31 |
| `ownInterfaceDirect` | `DerivedA` / 6 / 9 / 14 / 11；`DerivedB` / 20 / 23 / 28 / 25 | 17、31 |

每个表列方法的实际 report 均为 `structured` / `java`，恰有两条 `presented=true`、`refusal=null` 的 `NewRecord`。新断言逐条固定上述字段，并要求 argument、head、dup、constructor 和 store BCIs 都能从该方法的 source map 查询到；这只是方法级证据。完整 direct Main 的 CLI 重放仍 `compile_exit=1`、两腿均未接受，Main 保留 `@bytecode`；javac8 原始诊断列出 4 个“缺少返回语句”。因此测试没有把这些方法拼成“direct 全家族通过”。

旧测试对 `sequenceDirect`、`collectionDirect`、`throwableDirect`、`ownTwoHopDirect`、`ownInterfaceDirect` 的 refusal 断言已改成精确的结构化记录/source-map 断言。`boxedDirect` 六项 refusal 原样保留：Byte、Short、Long、Float、Double 的 `jre_new_interleaved_effect`，Integer 的 `jre_new_shape`；`ownGridDirect` 两个 `jre_new_shape` 也保留。`numberGridDirect`、`collectionGridDirect` 原有 fallback、物理 marker 及空 `news` 断言保留。CLI1 原始 Main JSON 中这三个边界方法对应的表现与这些断言相符。兄弟 integration `tests/p3_constructed_reference_array_elements.rs` 负责新的完整七类族源码编译/运行验证；本文件不对剩余控制作类型非法声明。

输入来源为 `results/legacy-family-cli1-root-v1/direct-javac8.jar`（SHA-256 `78edae441cc1114e96c34feee027f862967adcf7bf606427e408bfbc0caf3788`）和 `direct-javac23.jar`（SHA-256 `edfce48db43b09de0df8ded0d3b29dfe51d73e2714b8d2e6bdfc301614a8bf79`）。本次未运行 Java、Cargo、Git 或 rustfmt；测试编译状态待 root 后续 Cargo 验证。
