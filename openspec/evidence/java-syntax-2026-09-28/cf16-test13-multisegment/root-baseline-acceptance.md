# 固定 Test13 多段 finally 独立基线

root 在主线 `9440dd30` 使用独立构建的 Jarde CLI 重跑 `replay.sh`，输出保存在 `/private/tmp/jarde-cf16-test13-root-replay`；固定 JADX checkout 是 `2fb1b16386941660fda07e9017285aec40fcb37f`。固定 class 与可观察 probe 的 `test(I)V` 源正文 token 相同，BCI、每条 opcode 和五行异常表逐项相同。probe 只将五个被调用 helper 改为包可见并记录效果，以便 Java 8 编译保留固定 class 的 `invokevirtual`；固定 class major 55，probe major 52。文件 SHA 均通过 `SHA256SUMS` 校验。

原 probe 与 pinned JADX 完整源码都通过 `javac --release 8` 和 `java -Xverify:all`，七条路径逐行一致：提前返回、三条普通分支，以及三个不同调用点抛出并捕获的异常。每条 trace 恰好一个 finally 事件，catch 记录原异常的具体类型。当前 Jarde 的 `test(I)V` 在 BCI 56 报 `jre_guard_finally_copy`，完整源码可重编但正文为空，第一条提前返回路径预期 `do1,finally,`，实际为空。重放脚本将这些状态作为修前基线门。

固定 JUnit 测试仅断言源码含一处 `finally`；七路径语义是同 opcode/异常表 probe 的补充证据，不冒称 JUnit 自带的运行断言。此形态的多段 catch-all、提前 return 与三条正常清理出口独立于 Test12，须另写 OpenSpec，不能靠放宽单一复制的相似度判断合并。
