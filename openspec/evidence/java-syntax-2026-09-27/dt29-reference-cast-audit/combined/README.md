# 组合字段夹具快照

`inputs/` 保存第一次按 `TestFieldCast` 组合结构构造的审计输入，以及固定 JADX 测试源。`outputs/` 保存该复杂类族当时的原始/JADX/Jarde 源码和报告。

这个快照包含字符串拼接、`run`/`bits` 方法、多个嵌套类、泛型 `D` 与额外 synthetic 构造器。它说明组合输入里存在多个独立拒绝点，只用于后续扩展集成验收；实现任务的验收边界以父目录 `fixtures/private-field/` 的单 setter 夹具为准。该快照的路径清单已在迁移到 `combined/` 后重算。
