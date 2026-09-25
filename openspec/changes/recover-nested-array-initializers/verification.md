# 实施阶段验证

任务 1.3、2.1–2.3 已实施；3.1、3.2 保留给 root 独立验收。

- `tests/fixtures/p3-nested-array-initializers/v8/` 固定正例及乱序反例的 Java 8 class、源码与 runner。新增 `NestedArrayExtraUse.class` 由 `javac --release 8 -g:none` 编译；原 class 经 `java -Xverify:all` 输出 `[[4]]:4`，Jarde 没有发布 `new int[][]{…}`。
- `CARGO_TARGET_DIR=/tmp/jarde-nested-array-agent cargo test --test p3_nested_array_initializers -- --include-ignored`：4/4 通过。完整 Jarde 类以 Java 8 重编并在 `-Xverify:all` 下输出 `[[1, 2], [3]]:3:123`、`[[a, b], [c]]`，乱序控制输出 `[2, 1]:12`。正例两个方法的每个真实 BCI 都有 source map 条目；低 IR 预算在 `dynamic` 处停止并给出 `budget_exceeded_ir_items`，不含部分嵌套表达式；输出预算耗尽与预取消均不提交该表达式。
- 一维数组初始化、普通数组读写、窄类型数组写入、部分维度分配及短路异常链的定向测试通过。JDK 参与的一维布尔数组与部分维度完整类运行对照也通过。

待 root 独立重建 CLI、复核原/JADX/Jarde 三方结果、审查拒绝边界与通用检查，然后补记 3.1、3.2 的验收证据。
