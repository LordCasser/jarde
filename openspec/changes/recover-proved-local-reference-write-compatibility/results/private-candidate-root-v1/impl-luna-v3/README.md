# 私有候选 v3：局部 reference write compatibility

基于 OpenSpec 已冻结变更的私有实现候选；未应用到共享仓库。

- 基底 SHA-256：`3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7`
- 候选 `build.rs` SHA-256：`5d42d8fcf1fd8210a2e394a9742888cfde82357cd8c32ca73239862456fcbd67`
- `build.patch` SHA-256：`f041fac7b57149aca14529fef468b24958f69484972b6154b14891ed1f67f23c`
- V2 保留且未修改：`/private/tmp/jarde-local-reference-write-impl-luna-v2/build.rs`

相对 V2 仅有三处调整：caller 将 helper 的 `Some(Decided)` 原样采用，修复返回类型迁移后遗留的旧 tuple match；在物理 Object 写索引处说明懒整方法 census 用于发现 `SlotUse` 省略的精确 Object Store，保留 `reuse=None` 让候选保守失败，并在后续局部候选间复用一次扫描；`NoType::ConflictingWrites` 文档改为描述未能证明单一 Java 声明覆盖该 source-local lifetime，涵盖 JVM 类别冲突以及归属不明/不兼容的引用写。

未运行 Rust/JDK/JADX、构建、格式化或测试；本候选未验收。root 需按其流程先审阅并决定是否应用。
