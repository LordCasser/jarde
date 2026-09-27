# CF-16：固定 Test17 双具名 catch 与四份清理基线

固定 JADX HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`TestTryCatchFinally17` 对 Java 8/DX/D8 正向断言一次正文调用、一次 `finally` 调用、两个具名 catch，且不生成 `catch(Throwable)`；原测试关闭了 JADX 源码重编。这里从固定源提取 `TestCls.test()I`，只在 `probe/` 改辅助方法的可观察行为，目标方法字节码/异常行不变。原与 probe 的目标 class SHA-256 分别为 `a6f958b7e117786d684150527757162b708b100ecbe43e630252c95ae29ffc3a` 和 `fd132342a90bdca0fc50510a3e758e61215a44d5874e44f90828c1a5c6094cdd`。

Java 8 方法的异常表四行：`[0,3)→9 UnsupportedOperationException`、`[0,3)→16 NullPointerException`、`[0,3)→24 any`、`[16,19)→24 any`。四份 `invokestatic doFinally()V` 位于 BCI 3、10、19、25：正常路径返回 0；第一个具名 catch 吞异常后返回 0；第二个具名 catch 先保存 1、清理后返回 1；catch-all 在清理后加载并重抛原 Throwable。只有第二个 catch 中清理前的 BCI 16–18 受额外 catch-all 覆盖；四份清理自身均不在保护区。`replay.sh` 每次重编并比较原/probe 目标方法的 BCI、opcode、符号操作数和异常行。

原 class 与固定 JADX 输出的完整 Java 8 类均重编、`java -Xverify:all` 八路径逐字一致，见 `expected/eight-paths.txt`：正常 0、两种具名异常分别 0/1、`AssertionError` 原值逃逸，及四种清理抛错覆盖。固定 JADX 重排了两个互不相交的具名 catch 的源码顺序，但运行结果一致。当前主线 `2c1a9245` fresh Jarde CLI（SHA-256 `93b69427a12845d57720cfbd02aeafa644306b41e4452b477f08564476a037b8`）在 `test()I` 安全拒绝，BCI 24 提示重复清理缺证明；同一最小完整类的根类和辅助类方法没有 `@bytecode`。受控地仅把目标方法换回源码后，三份 Jarde 物理类可重编，验证运行八路径一致；这只用于隔离缺口，不算 Jarde 已恢复。

此形态与[固定 Test16 两行三副本](../cf16-test16-empty-catch/README.md)不同：第二个具名 catch 在清理前保存返回值，并有第四条覆盖其前缀的异常行。现有 FINALLY Guard pass 的三行共享证书、五行分段证书是可复用的分析入口，但不能把四行事实伪装成它们；先分析各副本的 CFG/SSA、异常覆盖和原值身份，再决定是否需要一个新的私有证书，暂不扩大正在实施的 Test16 OpenSpec。复现：`replay.sh JARDE_CLI OUTPUT_DIR`。D8/DX profile 尚未重放。
