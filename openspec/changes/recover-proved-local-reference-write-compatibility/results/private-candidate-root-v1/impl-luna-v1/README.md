# 私有候选：局部 reference write compatibility

候选仅包含 `build.rs`；源自共享基底 `crates/jarde-java/src/build.rs`，SHA-256 `3bce6972b9b8516a270725148e8e3e662b9dce09ff86c11207c6bb3b124d20f7`。共享文件仍与保存的 base 字节相同。

- candidate `build.rs` SHA-256：`d1d6f6eda86f60f29933f7c5da81568be908935da23cd9b40a54cec90ae52ab8`
- unified diff：`build.patch`，SHA-256 `d62b5642237a0d828232ff89eb57b687699b586cf0886fbfc87d1904deb83c9c`

实现把新决定放在 `decide_types` 的普通 first-write fallback：只有首写已选窄标量引用、首操作数是非-null reference、body 非参数/资源/catch header，且后续存在真实 `Store` 的 `stored` 操作数经 `direct_reference_write_type` 精确得到 `java.lang.Object`，才触发证明。证明要求至少两次写和至少一次读；每次写的 stored/written 均是 initialized reference 或 null、路径完整、非 caught；已知数组或无法证明非数组的 Unknown 来源拒绝。读端必须逐个是当前 identity 的 `Load`，只产生一个 Stack 输出，定义在相同 block/BCI，且唯一实际 consumer 读取该 Stack 值并为 `Return`；方法 descriptor 必须返回 exact Object。

完整性门另对 SSA 中同物理 slot 的所有 local read/write 重新枚举：每个 occurrence 查询 `reuse.variable_at`，None 拒绝、其他 identity 隔离；当前 identity 的 (BCI/value/read-vs-write/stored) multiset 与 `SlotUse` 一一匹配，因此 cleanup 被省略、访问缺失和重复 occurrence 都不能被集合去重隐藏。索引构建、访问枚举与消费者检查逐项 charge `AnalysisSteps` 并 poll，使用一次全方法指令索引避免 `instruction_at` 重复线性扫描。资源/catch header slot 集合复用既有 `resource_slots` 结果。

如果找到精确 Object 写而证明失败，候选走现有 `NoType::ConflictingWrites`，文案改为“选定类型未证明与该写兼容”，不声称不存在任何能容纳写入的 Java 声明。成功时一次性选择 Object；未触发仍保留原决策。

没有新增 API/永久测试；真实 class/API 正反控制由 root 另行准备。未运行 Rust/JDK/JADX、未格式化或编译、未执行测试；这是待独立审查的私有未验收候选，不能视为产品已交付。
