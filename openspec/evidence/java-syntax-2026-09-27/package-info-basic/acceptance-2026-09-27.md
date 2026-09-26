# EM-04 标准包信息类首切片验收

主线实现 `699f1868` 将准确证明的 `p/package-info.class` 输出为 `@java.lang.Deprecated` 后接 `package p;`，保留原物理类身份、注解 facts 和来源；不再写出非法 `interface package-info`。候选限于 Java 8.0、准确标志 `0x1600`、空成员表、唯一运行时可见的空 `java.lang.Deprecated` 注解，其他类属性和注解形状保持物理展示并标记拒绝。

root 在合并后的主线独立运行 [固定 replay](replay.py)：原始、JADX、Jarde 的完整 Java 8 源码均重编成功，`java -Xverify:all p.Check` 都输出 `true`。两个 JVM 可加载负例分别移除 `ACC_SYNTHETIC` 或把注解改成非包目标 `java.lang.Override`；Jarde 对两者明确拒绝投影并保留物理表示。负例 jar 使用固定 ZIP 元数据，证据中的 snapshot 标识可重复。

`cargo fmt --all -- --check`、`cargo test -p jarde --lib --locked`、`cargo check --workspace --locked` 和 `openspec validate recover-proved-package-info-source --strict` 全部通过。回放的 Cargo target 自动清理。当前只关闭 EM-04 的冻结标准形状，其余包注解与属性组合继续逐项审计。
