# 私有候选 v4：局部 reference write compatibility

基于已冻结 OpenSpec 变更的私有实现候选，未应用到共享仓库。

- 基底 SHA-256：`3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7`
- 候选 `build.rs` SHA-256：`e45b9a8e1c6e8d32e82b65098853e5323101b4a4e2a400cebb545d950479bb52`
- `build.patch` SHA-256：`aa2f2a949f0eeceb39bc7c64ba3f3ecbc551899f8d0310fd8dab39af227457d0`
- V3 保留且未修改：`/private/tmp/jarde-local-reference-write-impl-luna-v3/build.rs`

相对 V3 仅将物理 Object 写触发筛选条件从 `bci == first.at` 改为 `bci <= first.at`。因此只有首个窄引用写之后的 Object Store 才可作为后续触发；更早且未归属的写不会被错算成该 lifetime 的 later 写。

未运行 Rust/JDK/JADX、构建、格式化或测试；本候选未验收。
