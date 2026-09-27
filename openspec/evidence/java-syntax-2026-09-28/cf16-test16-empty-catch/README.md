# CF-16：固定 Test16 的空 catch 与三份调用清理

固定 JADX 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。`TestTryCatchFinally16.java` 的 `JAVA8` profile 正向检查一次 `doSomething()`、一次 `doFinally()`、`catch (Exception e)` 和唯一 `finally`，但显式关闭了 JADX 的源码重编检查。`original/TestTryCatchFinally16.java` 从该测试逐字保留 `TestCls.test()V`，`probe/` 只给两个辅助方法加可观察副作用与异常，未改目标方法。两份目标 class SHA-256 分别为 `a32f63ffe5749eac6553192b4f4dd10cf109883fc993681891f99e0140f60402` 和 `84567faf733ef3932df19f8e89478cc796d2d3511cff88b8a64301313b2f1cb2`；`replay.sh` 逐 BCI/opcode/符号操作数和异常行比较二者的 `test()`。

Java 8 目标方法为 BCI 0 `invokestatic doSomething`；BCI 3、10、17 三份 `invokestatic doFinally`；BCI 9 是具名 `Exception` handler，BCI 16 是 catch-all handler，BCI 22 正常返回。异常表恰为 `[0,3)→9 Exception`、`[0,3)→16 any`。清理调用若自身抛错，不落入同方法的 catch-all；正常、具名 catch 与 catch-all 三份清理不能仅凭调用目标相同就折成 `finally`，还须证明其覆盖、入口、出口和原 Throwable 的重抛。`Test15` 是独立的寄存器错误合并负向测试，不作为本样本的正例。

当前主线 `d81df523` 的 fresh Jarde CLI（SHA-256 `9ea58208a54a769b53126bc3374e1fa5ac207d21eeb78a24507aae3d1b3adc44`）对 `test()V` 安全拒绝：BCI 16 提示重复清理缺少完整证明，正常视图还留下 BCI 9、16、22 三个未覆盖块；同一最小完整类的其余根类与辅助类方法无 `@bytecode`。受控地仅把拒绝的目标方法换回源码后，三份 Jarde 物理类可用 Java 8 重编，`java -Xverify:all` 六路径与原 class 相同；这隔离了目标方法缺口，没有把受控补丁算作 Jarde 恢复。

固定 JADX 的完整源码与原 class 都经 Java 8 重编、`java -Xverify:all` 运行，六路径逐字一致，见 `expected/six-paths.txt`：正常与 `Exception` 被捕获时清理一次，`AssertionError` 逃逸时仍清理一次，清理自身抛错覆盖先前异常。复现命令：`replay.sh JARDE_CLI OUTPUT_DIR`；脚本会检查固定 checkout、目标 class 哈希、方法布局、三方当前状态及原/JADX 六路径。此基线只证明 Java 8 的固定 lowering，D8/DX 两个 profile 待独立审计。

`classes/Test16.class` 冻结了 probe 目标 class，`classes/near/` 冻结了五个由 `neighbors.py` 对该 class 做局部字节改动的有效近邻。每个近邻保留完整辅助类和 Runner，以 `java -Xverify:all` 执行六条路径；`test.javap.txt` 在回放输出中记录真实异常行。近邻的 SHA-256 与可观察差异如下：

| 近邻 | SHA-256 | 真实异常行／路径变化 |
| --- | --- | --- |
| `different-target` | `7c9ac834e0f7c2399df38154ba28bbad1878fa0019f1dadda7c046b99bf48f65` | 两行仍为 `[0,3)→9 Exception`、`[0,3)→16 any`；BCI 10 调用 `doSomething`，具名异常路径变为 `body,body,IllegalStateException:body`。 |
| `cleanup-covered` | `a5cebcd98c8cbfc87f8c78119ed320d6106801adab9c6d15484b19845fa07f16` | 第二行扩成 `[0,6)→16 any`，清理调用 BCI 3 进入 catch-all 保护。 |
| `rows-swapped` | `2558e48e9d4ffa1dc57a9713a3be745f63c7e4b5fb1dcc59a661a923b77d3c29` | 行顺序变成 `[0,3)→16 any`、`[0,3)→9 Exception`；具名异常现在重抛。 |
| `throwable-rewritten` | `e0a7ad27275aea31cebd910fab0e576b79741bf24f37224d6e4ad3cf3fd85a59` | 两行保持原状；BCI 20 改为 `aconst_null`，非具名异常路径变为 `NullPointerException`。 |
| `external-cleanup-entry` | `0b66d41b8887e3a181ba950d3cdab350d58489a7e77257b8a2336a56100a3366` | 两行保持原状；BCI 13 转入正常清理 BCI 3，并补有效 stack map；具名异常路径清理执行两次。 |

验收命令为 `accept.sh JARDE_CLI OUTPUT_DIR`。它重放原/JADX 基线，重编 Jarde 三个完整物理 Java 8 类（Runner 仅将嵌套源码名改为对应物理二进制名），核三方六路径逐字一致；再核目标仅一个 `finally`、空具名 catch、11 个来源 BCI 和五个有效近邻的安全拒绝。没有修改固定 `test()V` 或用受控方法补丁通过 Jarde 路径。
