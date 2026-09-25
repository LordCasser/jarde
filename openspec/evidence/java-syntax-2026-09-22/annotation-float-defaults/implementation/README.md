# Jarde 注解 float/double 默认值实现证据

`FloatDefaults.java` 由重建后的 `target/debug/jarde-cli` 根据冻结的 293 字节基线 `FloatDefaults.class` 生成；相邻 runner 是冻结证据中的 runner。生成源码通过 `javac --release 8 -g:none` 编译，并由 `java -Xverify:all` 执行，四行输出与原始/JADX 结果相同：`80000000`、`1`、`7f800000`、`7ff8000000000000`。

`FloatingBoundary.java` 是使用相同 release 参数编译的 Java 8 边界输入。Jarde 完整 class-source 输出保存在 `FloatingBoundary-jarde.java`，与未修改的反射 runner 一同编译后，其 raw bits 输出与 `boundary-original-run.txt` 相同。覆盖负零、最小次正规值、float/double 最大有限值、正负无穷及 canonical NaN。

构造的集成测试覆盖一个先含有限值、后含 payload NaN 的 float 数组，并断言整个 `default` 被省略。负 NaN 单池项 patch 的冻结证据及其 JVM/JADX 基线保留在父目录。`FloatDefaults.json` 是 CLI 的完整 evidence 报告，只保留既有结构化成员事实，不发布解析后的默认值树。

`FloatingArrays.java` 及其 runner 验证 F/D 数组的合法有限值默认值：原 class 与 Jarde 完整输出经 Java 8 编译和 `-Xverify:all` 执行后，raw bits 完全一致。`patch_array_payload.py` 只把 `floats()` 数组引用的一个池项改成 payload NaN；patched class 在 JVM 下反射出 `7fc00001`，Jarde 生成文本省略整个 `floats` 默认值，同时保留可拼写的 `doubles` 默认值。省略默认值后的源码仍可编译；runner 对不存在的 float 默认值读回 `null` 并按预期以 NPE 退出，退出码和输出保存在相邻文件中。

`javac` 日志只有当前 JDK 对 Java 8 source/target 选项的预期过时警告。`boundary-jarde-run.txt` 与 `boundary-original-run.txt` 逐字节一致。
