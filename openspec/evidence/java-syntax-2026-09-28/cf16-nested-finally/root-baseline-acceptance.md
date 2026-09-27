# CF-16 固定嵌套 finally 基线独立验收

root 在主线 `61a167b2113f16715d8038834e2952339c3844d6` 使用 CLI SHA-256 `851b335980eaffb216ab9a164767b47ed357256caa655d95559fb9b655a19937`，独立运行本目录 `replay.sh`，输出保存在 `/private/tmp/jarde-cf16-nested-root-replay-final`。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`；内部类 class SHA-256 为 `d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185`。

固定 `test1/2/3` 与最小顶级类重编后的三方法，BCI、opcode 和异常表形状逐项相同。原类和 pinned JADX 最小完整源码均通过 `javac --release 8`、`java -Xverify:all`，九条路径 stdout 逐字节一致。pinned JADX 默认对固定类输出三处 `finally`，`--no-finally` 输出七份清理副本。当前 Jarde 最小完整源码虽能重编，九路径 runner 第一项断言失败：预期 `call-out-finally`，实际为空串；三个目标方法在 BCI 42/32/42 因 `jre_guard_finally_copy` 安全拒绝。

这只确立三方法的独立差距及同字节码最小完整类的运行基线。固定内部类所属完整类的 `runTest(II)String` 另有 switch/try Region 多 owner，InnerClasses family 也未闭合；两项不由本变更验收。保存的完整 Jarde report 含计时字段，因此独立重跑的原始 report SHA 可随耗时变化；诊断、BCI 和运行结论保持一致。
