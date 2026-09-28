# 固定 Test5 主线验收

主线变更为 `2622d132`。独立审阅了 Guard 的三行 `[21,34)→93`、`[44,83)→93`、`[93,95)→93` 与 45 个指令起点、完整 canonical 边、三份 `close()` 的调用目标和 local 4 接收者；两处保存值分别由 local 5、6 返回，handler 的 local 7 原 Throwable 被重抛。正文回边 `76→53` 属于正常执行的 do-while，清理自身不在自保护范围内。Region 走现有循环及有界 finally 正文，Builder 一次提交 `try/finally`、在两个分支内返回，全部目标 BCI 有来源。证书只承认固定物理布局；相邻 lowering 需要另行取证。

在主线使用临时 Cargo target 从头构建 CLI，并将新生成的完整 class-source 与 [冻结输出](TestTryCatchFinally5$TestCls.java)逐字 `cmp`；`cargo test -p jarde-java --tests --locked` 通过，包括 45 个 BCI 来源、预算/取消及旧 finally 切片。独立运行 [九路径脚本](run-behavior.sh)，固定原 class、原 Java 8 转写、固定 JADX Java-input 与新 Jarde 完整类均通过 Java 8 编译（原 class 不重编）和 `java -Xverify:all`，四份结果逐字相等。运行 [近邻验证](neighbors/verify.sh)确认六个 class 均通过 verifier；以新 CLI 重放六个近邻，全部仍为 `@bytecode` fallback，没有发布 `finally`。临时编译产物已清理。

验收范围是固定 Test5 JVM classfile 和六个有效近邻。Test2 的 void 三循环、Test9 的可空清理、其他 JVM/DEX profile 不从本结果外推。
