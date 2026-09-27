## 1. 关系与注解元素证明

- [x] 1.1 只选择一个直接 `ACC_ANNOTATION|ACC_INTERFACE` 成员行，校验 owner/child 双方关系、版本、名称、flags 与唯一 selected definition；用真实 class 字节变体证明缺失、重复、冲突及顶级 `$` 均不进入投影
- [x] 1.2 保留 child 独立物理报告，对完整方法/字段表和每个元素的 `AnnotationDefault` 声明/解析事实做同轮门槛；以截断属性、错误 annotation flags 与取消/预算单测证明不产生局部根声明

## 2. 结构化源码装配

- [x] 2.1 复用 DT-13 根级 writer 的共同嵌套声明放置与 derived range 翻译，按 child 声明/元素记录写出 `public @interface A`，不裁切 child 展示文本；验证 DT-13 原有 enum regression 仍全绿
- [x] 2.2 根报告发布 owner/child 物理锚和嵌套声明范围，保留 child `Holder$A` 查询；验证默认值的 `float` 原始位未改变且顶级 `Dollar$A` 仍是顶级

## 3. 固定三方重放与门禁

- [x] 3.1 更新固定 `dt22-nested-annotation/replay.py` 为修后验收：原/JADX/Jarde 全源码与原 `Holder.A` consumer 以 Java 8 重编、`-Xverify:all` 运行逐字一致，保留修前诊断在报告中
- [x] 3.2 跑 `openspec validate recover-proved-nested-annotation-source --strict`、相关 class-source/annotation/enum 单测、fmt/check；记录已知独立门禁债务和临时 Cargo target 清理，提交可复核结果

验收记录：`cargo fmt --check`、`cargo check --workspace`、DT-22 新增 7 项测试、`p3_annotation_default` 8 项测试、DT-13 nested-enum 原子预算测试和 OpenSpec strict validation 均通过。完整 `cargo test -p jarde --lib --no-run` 另被现有 `tests/bulk_recovery_cancel.rs` 的 `BulkProbe`/`with_probe` feature-gate 编译错误挡住；这不属于 DT-22，未扩大本次范围。固定 replay 命令和三方运行结果见同目录 `results.json`。
