# Root 独立复核：方法泛型头的静态返回子切片（2026-09-24）

Root 审读了类级投影的证明链：同一成员的 `Signature` 由 reader 解析并逐位置核对物理 descriptor；仅存在 `Signature` shell 时，恢复端从同次 `build::Program` 和 SSA 生成参数槽/返回值侧证据。局部引用须有对应槽的真实 load 来源，整方法不得写入参数槽；条件只能来自未改型的布尔参数，条件附带的来源限于 JVM 条件分支。类级装配遇同类同名 Methodref、注解、varargs、复杂签名或无此侧证据时整项拒绝。独立方法 `RecoveryReport` 不携带泛型 AST，也不依赖 XRef。这个闭环没有以输出字符串反推方法体类型。

Root 构建的 CLI SHA-256 为 `b50fd38aa480db684bfc92466e4da66c73959d33a0e81b1b3a7b45cc9f0b56e4`。以冻结 class 各自生成原/JADX/Jarde 完整 Java 8 类、原样 `javac --release 8`、`java -Xverify:all` 并核对重编后的 `Signature`：`GenericMethodProbe.class` SHA-256 `928311a035b4716e846804dfcf78165a7958a7531da1f635ea28f040d54af2e3` 三方 runner 均为 `3\n1`；`GenericThrowsProbe.class` SHA-256 `f1f1b4cff298113db985e13cc0fe3cc1df740ca4e388f1876712ae21dd2b2b27` 三方均为 `3|1\njava.io.IOException`。五个冻结的 JVM 可验证反例（改界、未绑定变量、类级变量、复杂签名、正文类型不兼容）由当前 CLI 全部记录拒绝、不输出泛型头，Jarde 完整源码均通过 Java 8 重编。

Root 复跑 `generic_method_projection` 6/6、`class_source` 47/47、`jarde-cli` 的 `class_source_cli` 16/16；先前独立复跑 reader 冻结测试 3/3 与 query 方法签名回归 1/1。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-generic-method-signatures --strict` 均通过。

验收只覆盖静态、一个方法局部类型变量、简单非参数化 class 上界，以及直接参数返回或布尔参数选择两个参数的正文。[代理的形状与拒绝矩阵](verification-class-source-2.3-3.1.md)列出其它未证明的赋值、调用、复杂声明。2.3/3.1 仍未整体完成；预算/取消及全局门禁另按 3.2/3.3 验收。
