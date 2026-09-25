# 2.2 直线 finally 结构呈现验收

`finally@1` 在既有 `guard::Plan` / `Region::Guard` 上认领半开受保护正文、正常 cleanup、副本 handler 和保存的 return；`StmtKind::Try` 仅加可选 `finally_body`。`statement_free` 门槛拒绝平坦 builder 无法表达的 `Comparison` / `Transfer` / `Throw`，因此含 `if` 的 `ImplicitCleanup.run` 没有被错误包装。未增加第二套异常区域或通用控制流机制。

代理的定向检查：`jarde-java --lib` 139/139、`p3_finally_straight` 2/2、`p3_guard` 13/13，TWR catch 测试 1 通过、1 既有 ignore。root 独立从 CLI SHA-256 `8d93cf642dbca5de2a625677c3e13a44a989b2d5824ed43058d5724927744faf` 复制二进制，再重编 593 B 原类 SHA-256 `6c1e6ee18ca8370ffc5ad44f8ad911a000ed34f6c258d95d603004d4aeab701a`。未经修改的 Jarde 完整类通过 `javac --release 8` 和 `java -Xverify:all`，与原类同为 `normal:return:1:12`、`throw:java.lang.IllegalArgumentException:same=true:12`。同一 CLI 对 `ImplicitCleanup` 与扩围异常表变体均没有输出 `finally`，仍在 BCI 25 引用；未把有分支正文或重入清理误收。

root 新增[直线清理调用抛错样本](../../evidence/java-syntax-2026-09-24/finally-straight-cleanup-throws/analysis.md)：原/JADX/Jarde 完整类均编译验证，四条完成路径逐行相同；Jarde 只发射一份 cleanup。该样本证实当前直线形状的抛错覆盖行为，但不解除 `ImplicitCleanup` 正文含分支的 2.3 门槛。完整来源/预算/取消和根级全量测试仍属 2.4/3.2，未据此勾选。

隔离检查发现共享工作树中 `p3_typed_catch::finallyIncrements` 仍有 `local 1 crosses a quoted fallback region` 失败；该 handler 无调用/字段/cast，不满足既有 `finally_copy` 候选预筛，故本次 `finally@1` 认领路径不会进入。将其作为独立局部作用域回归债务，不扩大当前改动。root 的 `cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-finally-cleanup --strict` 通过。代理清理专用 Cargo target 后可用空间约 14 GiB。
