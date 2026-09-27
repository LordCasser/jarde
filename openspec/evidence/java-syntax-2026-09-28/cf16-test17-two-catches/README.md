# CF-16：固定 Test17 双具名 catch 与四份清理基线

固定 JADX HEAD 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`TestTryCatchFinally17` 对 Java 8/DX/D8 正向断言一次正文调用、一次 `finally` 调用、两个具名 catch，且不生成 `catch(Throwable)`；原测试关闭了 JADX 源码重编。这里从固定源提取 `TestCls.test()I`，只在 `probe/` 改辅助方法的可观察行为，目标方法字节码/异常行不变。原与 probe 的目标 class SHA-256 分别为 `a6f958b7e117786d684150527757162b708b100ecbe43e630252c95ae29ffc3a` 和 `fd132342a90bdca0fc50510a3e758e61215a44d5874e44f90828c1a5c6094cdd`。

Java 8 方法的异常表四行：`[0,3)→9 UnsupportedOperationException`、`[0,3)→16 NullPointerException`、`[0,3)→24 any`、`[16,19)→24 any`。四份 `invokestatic doFinally()V` 位于 BCI 3、10、19、25：正常路径返回 0；第一个具名 catch 吞异常后返回 0；第二个具名 catch 先保存 1、清理后返回 1；catch-all 在清理后加载并重抛原 Throwable。只有第二个 catch 中清理前的 BCI 16–18 受额外 catch-all 覆盖；四份清理自身均不在保护区。`replay.sh` 每次重编并比较原/probe 目标方法的 BCI、opcode、符号操作数和异常行。

原 class 与固定 JADX 输出的完整 Java 8 类均重编、`java -Xverify:all` 八路径逐字一致，见 `expected/eight-paths.txt`：正常 0、两种具名异常分别 0/1、`AssertionError` 原值逃逸，及四种清理抛错覆盖。固定 JADX 重排了两个互不相交的具名 catch 的源码顺序，但运行结果一致。当前主线 `2c1a9245` fresh Jarde CLI（SHA-256 `93b69427a12845d57720cfbd02aeafa644306b41e4452b477f08564476a037b8`）在 `test()I` 安全拒绝，BCI 24 提示重复清理缺证明；同一最小完整类的根类和辅助类方法没有 `@bytecode`。受控地仅把目标方法换回源码后，三份 Jarde 物理类可重编，验证运行八路径一致；这只用于隔离缺口，不算 Jarde 已恢复。

此形态与[固定 Test16 两行三副本](../cf16-test16-empty-catch/README.md)不同：第二个具名 catch 在清理前保存返回值，并有第四条覆盖其前缀的异常行。现有 FINALLY Guard pass 的三行共享证书、五行分段证书是可复用的分析入口，四行事实由独立私有证书证明。固定 Java 8 重放：`accept.sh JARDE_CLI OUTPUT_DIR`。D8/DX profile 尚未重放。

## Test17 Java 8 验收

`accept.sh` 每次核固定 JADX HEAD、原/probe 的逐 BCI/opcode/异常表和 class SHA，再把原、JADX、Jarde 的完整类以 `javac --release 8` 重编。三方运行 `Runner` 的八条路径，均经 `java -Xverify:all`，结果逐字等于 `expected/eight-paths.txt`。Jarde 的 `test()I` 有两个具名 catch、一条 `return 1`、一次源码清理调用和唯一 finally；四份物理清理 BCI 3/10/19/25 与全部 18 个 BCI 均能从 source map 查询。其 Canonical CFG 有五块：`0→30`、`9→30`、第二 catch 的 BCI 16–23 同块，以及 catch-all BCI 24–29；第四行 `[16,19)→24` 只覆盖第二 catch 的保存前缀，不覆盖 BCI 19 的清理。证书逐指令核真实覆盖、SSA 保存/加载/返回和原 Throwable 重抛，没有放宽旧两/三/五行准入。

`neighbors.py` 从固定 probe class 生成并冻结七个 verifier 有效近邻；每个均运行完整八路径，Jarde 均保留 `@bytecode` 且不输出唯一 finally。冻结 SHA-256：

| 近邻 | SHA-256 |
| --- | --- |
| `different-target` | `e014a8a90fe790899cfcfa05dd2008a7011cc17aad025c9238d890b23e7b4db3` |
| `cleanup-self-protected` | `e7bedd305214717f5ec8574b6450f66aac4b14e6631cbc2cc66ad70c9646af82` |
| `cleanup-covered` | `7ccf811997d89e9b3ed374ca1dd88632f13d4ceceb589750d44b9a26bf0e3d7c` |
| `rows-swapped` | `1f1cf607cff2ba24d76ab492be356604e2092fe358bfe5c5be9dc363606cf43e` |
| `saved-return-rewritten` | `3c4b723f917a98085ba95575609674a654bfd0072f3f023f5420b1cca07e8a0d` |
| `throwable-rewritten` | `0d0eabc180ea89e23e2c328e3aaf70c6b04680bf9e60b41ad13287065ec0b1a4` |
| `external-cleanup-entry` | `003d17f1ccd230e43d763d1b5a9ea92283acc55ba481b02ac8a57a71b3bb8776` |
