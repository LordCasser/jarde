# CF16 / TestTryCatchFinally9 证据

本目录只记录 Test9 的恢复行为，不改生产代码或 OpenSpec 规格。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。测试源码 SHA-256 是 `39c220b0081e01b1be7f60760bf6d7dbf7309b7ee3a5c8ec18a6c6fd74c99017`，物理目标 class SHA-256 是 `251e21b8ade9f6a9096d807dc31a9fcaae4e2a551600a7230b8f1b7f9f165381`。额外用于区分输入 profile 的 `IntegrationTest.java` 与 `JadxCLIArgs.java` SHA 在 `results/identities.txt`。

## 物理字节码与固定输入

目标 `TestCls.test()Ljava/lang/String;` 的完整 `javap -c -v` 在 `bytecode/TestCls.javap.txt`。它先把 `null` 存到 slot 1（BCI 0–1），从相对资源 `resource` 读流并仍存到 slot 1（BCI 2–11），再用 `Scanner` 生成返回值并保存到 slot 3（BCI 12–42）。正常路径在 BCI 43–50 检查 slot 1 并于 BCI 48 调用 `InputStream.close()`，之后在 BCI 51–52 返回。异常 handler 位于 BCI 53：保存 Throwable 后在 BCI 55–60 检查并关闭 slot 1，最后于 BCI 63–65 重抛。

异常表原序如下，范围为左闭右开：

| ordinal | 保护范围 | handler | 类型 |
| --- | --- | --- | --- |
| 0 | `[2,43)` | 53 | catch-all |
| 1 | `[53,55)` | 53 | catch-all |

固定源码 `TestTryCatchFinally9.java` 有 5 项活动断言：不含 finally 提取失败诊断、不含生成的 `throw`、含一个 `finally`、含一个 `if (input != null)`、含一个 `input.close()`。这些是 DEX profile 的源码形状断言，不直接执行 close 行为。

`fixtures/TestTryCatchFinally9$TestCls.java` 是该 `TestCls` 方法的独立 Java 8 转写，二进制类名保留 `$TestCls`，以便资源仍相对于相同 package 查找。它用 `javac --release 8 -g` 生成的目标方法 BCI、opcode 和异常表与固定物理 class 相同；完整 `javap` 在 `bytecode/original.javap.txt`。两份类的外围元数据不声称相同，方法体才是本次物理对照对象。

## DEX profile 与 Java-input CLI

JADX 集成测试默认设置在 `jadx-core/src/test/java/jadx/tests/api/IntegrationTest.java`：`DEFAULT_INPUT_PLUGIN = "dx"`；`loadFiles` 以 `useDx = !isJavaInput()` 设置 `JadxArgs.useDxInput`。在 `TEST_INPUT_PLUGIN` 未设置时，固定 `TestTryCatchFinally9` 集成测试已用 `--rerun-tasks` 独立重放并通过，结果见 `results/jadx-integration-test.txt`。CLI 的 `JadxCLIArgs.useDx` 默认是 `false`；不加 `--use-dx` 走 Java-input 路径，加 `--use-dx` 则请求 Java bytecode 转 DEX。固定源码和这些选择点的哈希在 `results/identities.txt`。

观察到的 Java-input CLI 完整源码在 `jadx-java-input/`：它把资源保存到 `input2`，留着初始化为 null 的 `input`，并生成空的 `if (input2 != null) {}`；finally 仍检查 `input`，所以有效资源路径漏掉 close。`run.sh` 将该完整源码以 Java 8 重编，在 `-Xverify:all` 下通过，并在有效资源探针中观察到 `close=0`。

作为 profile 对照，`jadx-dx/` 是同一固定 class 经 CLI `--use-dx` 生成的完整源码。当前 CLI 日志显示 DX 转换器失败后使用 D8，并成功完成 Java-to-DEX 输入。重编源码在有效资源路径观察到 `close=1`，结构与集成测试断言相符。由 `IntegrationTest` 源码和 CLI flag 选择可以判定两者都走转换到 DEX 的输入类型；二者入口不同，所以将 CLI 输出作为 profile 对照，集成测试只作为默认 DX profile 的通过证据。

资源探针由 `fixtures/Runner.java` 提供：自定义 `URLClassLoader` 为 package 相对路径提供固定字节 `resource-data`，并用 `URLStreamHandler` 返回计数型 `InputStream`。它也能返回缺失资源。固定物理 class、原转写、两份 JADX 源码各自隔离编译、以 `java -Xverify:all` 执行的结果为：

| 输入 | 有效资源 | 缺失资源 |
| --- | --- | --- |
| 固定物理 class | `resource-data`, close=1 | `NullPointerException`, close=0 |
| 原 Java 8 转写 | `resource-data`, close=1 | `NullPointerException`, close=0 |
| JADX Java-input 完整输出 | `resource-data`, close=0 | `NullPointerException`, close=0 |
| JADX `--use-dx` 完整输出 | `resource-data`, close=1 | `NullPointerException`, close=0 |

因此只有有效资源路径区分 Java-input 的丢失清理；缺失资源时，构造 `Scanner(null)` 先抛 NPE，finally 看到 null 输入，不会调用 close。

## Jarde 当前边界

Jarde 的完整 `class-source` 输出和单方法恢复正文在 `jarde-out/`。方法正文保留拒绝，理由为 `local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`，物理引用为 BCI `0 33 40 42 47 51 53 59 63`。报告摘要记录 `content = explanation_only`、`syntax_status = not_java` 和分析执行状态。

Jarde 完整 class-source 经 `javac --release 8 -g` 编译失败，报 `missing return statement`。`run.sh` 不会执行它；即使某个仅含 fallback 注释的 void 方法碰巧可编译，也不会被记为恢复成功。本证据没有将其改成方法桩再作行为比较。

## 重放

在 Jarde checkout 根目录执行：

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-test9-catch-finally/acquire.sh
```

`acquire.sh` 会校验固定 JADX HEAD 与四个源/class 哈希，导出 Java-input 与 `--use-dx` 两份源码，重放默认 DX 集成测试，运行 Jarde CLI，再调用 `run.sh` 编译和执行可验证的完整源码。若 JADX checkout 不在默认路径，可设置 `JADX_ROOT`。Cargo 使用独立 `/private/tmp` target，脚本退出时清理；`run.sh` 也自动清理 Java 编译目录。输出的路径、CLI 临时 jar 名、编译时间和运行时间会归一化，`results/source-and-class-sha256.txt` 使用相对路径。
