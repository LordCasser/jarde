# CF-16：固定 Test14 的条件 finally 基线

固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。固定测试 `TestTryCatchFinally14.java` 的 `TestCls.test()` 要求唯一 `.doSomething();`、唯一 `finally`、唯一 `.doFinally();` 以及两次 `!= null` 条件。本目录的目标方法正文逐句保留该测试形态，只让被调用的辅助方法可观察：`doSomething()` 可把字段 `t` 置空或抛错，`doFinally()` 可抛另一个错。因此七路径能区分“在 finally 重新读取字段”与“沿用第一次读取的接收者”，以及最后抛错覆盖原抛错。

`javac --release 8 -g` 编出的目标 class major 52，SHA-256 为 `8857b84944f1a0c8ec0d8805d2ba0e7430dfadfda7629b4a14ea4064eb1d4ece`。`test()V` 的异常表只有 `[0,14)→31 any`，正常清理为 BCI 14–28 的字段读取、判空、调用和转移，异常清理为 BCI 31–47 的保存原 Throwable、重新读取字段、判空、调用、加载原值并重抛；BCI 48 返回。这个形态与先前 Test13 的五行分段保护范围不同。

在主线 `df205b29` 构建的 fresh Jarde CLI 与固定 JADX 上运行 [replay-baseline.sh](replay-baseline.sh)：原 class 与固定 JADX 的完整 Java 8 类均重编、`java -Xverify:all` 七路径逐字一致：无字段时无效果；正常为 `body,finally,`；正文把 `t` 置空时只有 `body,`；正文抛错时清理仍执行；清理自身抛错时 `IllegalArgumentException:finally` 覆盖原 `IllegalStateException:body`。Jarde 对目标 `test()V` 保守拒绝，首个缺口在 BCI 0 的 catch-all 形态，后续块由 normal-flow view 报为未覆盖。当前完整类另含辅助 accessor 缺口，不能把整类编译失败单独归因于 `test()`。

现有 `guard::finally_copy` 可以识别保存异常/重抛的外形，但 `prove_finally_copy` 要求正常与异常清理都是直线指令序列、正常端保存返回值且同块返回；它不能证明此处各有一次条件分支并在两个路径上重新读取字段。已有 `Shape::Finally { structured: true }` 只允许受保护正文分支，不表示清理自身的分支。因此下一步需要先设计两个清理 CFG 的有界等价证书、条件/字段读取的 SSA 身份和所有入口出口，再决定是否沿用现有 `Region::Guard`/`finally_body` 和 Builder；不能简单放宽 `cleanup_sequence` 或合并两次字段读取。`FinallyOnce.handled/escaping` 另有独立差距，不与本样本混同。

脚本每次重编目标 class 并逐字比较冻结 class，核 pinned JADX HEAD、三方基线状态和原/JADX 七路径；输出目录由调用方指定。此处仅冻结差距和架构边界，尚未建立该条件 finally 的实现任务，也不把 CF-16 记作追平。
