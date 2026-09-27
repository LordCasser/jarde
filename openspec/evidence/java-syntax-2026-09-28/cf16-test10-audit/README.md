# CF-16 Test10 有限事实审计

固定 JADX checkout 是 `2fb1b16386941660fda07e9017285aec40fcb37f`。审计仅覆盖这个固定测试、它直接读取的 smali、固定 CLI 输出，以及一份按测试注释的 Java 8 对照；没有修改生产代码或 OpenSpec 任务。

## 测试实际保证什么

固定 JUnit 源在 `jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally10.java`。唯一活动测试第 35 行调用 `disableCompilation()`，然后断言反编译代码不含 `boolean z = null;`、不含 `} catch (Throwable`，且各有一次 `} finally {`、`.close();`、`} catch (IOException e`、`.logException(`。固定测试通过：

```text
./gradlew :jadx-core:test --tests jadx.tests.integration.trycatch.TestTryCatchFinally10
BUILD SUCCESSFUL; 1 test task executed
```

第 11–31 行注释中的 Java 只是意图示例，不属于额外的编译、行为或完整源码断言。活动断言验证了一组源码子串及计数；没有验证返回表达式、保护范围归属、清理执行路径或清理抛错时的行为。`doesNotContain("} catch (Throwable")` 也只排除该打印形式，不能单独证明异常覆盖关系。

## 固定 smali 与 JADX 输出

smali 字段 `l` 在输入中只有声明、没有初始化器（第 5–6 行）。方法目标是 `test(Context, int): String`。DEX 输入的异常保护由标签直接可见：主体从 `:try_start_0` 到 `:try_end_0`，有 catch-all handler；正常清理副本和 catch-all handler 中的清理副本各有自己的 `IOException` handler（`try_start_1/2`）。输入不是 JVM classfile。

固定 CLI 在该 smali 上确实输出一个 `finally`，内部只打印一次 `close()`、`IOException` catch 和 `logException()`，并与活动正向字符串断言相符。输出完整保存为 `jadx/TestTryCatchFinally10.java`（字段初始化按下文 adapter 说明调整）；未经调整的固定 CLI 输出含 `private static final DebugLogger l = null;`，记录于回放生成的原始 JADX 源。

## Java 8 对照及边界

`original/trycatch/TestTryCatchFinally10.java` 的目标方法逐句采用测试注释中的 Java 形态；`original/` 下其余小型 stub 和 `probe/Runner.java` 只提供可观察的资源、流和 logger。它由本机 `javac 23.0.1 --release 8` 编译。目标方法的真实 JVM exception table 是：

```text
from to target type
51 55 58 Class java/io/IOException
6 47 74 any
80 84 87 Class java/io/IOException
74 76 74 any
```

这是从 Java 源重编的等价形态，不是 Test10 DEX 的地址/异常表复刻。javac 在 catch-all handler 的 `astore` 上还生成自保护行 `[74,76)→74`；smali 中没有相同行。源码、DEX 与重编 JVM classfile 的 BCI/异常表不可混称为相同输入。

回放会用固定 smali 再跑 JADX，并只把 CLI 源的空 logger 字段初始化改为 `new DebugLogger()`，使探针能够运行日志路径。目标 `test` 方法文本不作改写。原 Java 8 类和该 adapter 在 `ok`、`empty`、`open-throw`、`close-io`、`close-runtime`、`logger-runtime` 六条探针路径输出逐字相同，结果见 `original-run.txt` 和 `jadx-run.txt`。这些结果只证明注释形态与固定 JADX 目标方法在这个小型 Java stub 环境中的对照；输入 smali 的 `l` 默认为 null，而探针有意提供非空 logger，因此不声称 IOException 日志路径等于 DEX 运行时。

使用主线 `528170e4b0733b02bb9492ffb29dc8e15c820555` 构建的 Jarde CLI 对这份 Java 8 classfile 没有给 `test()` 发射语句。它标记 explanation-only：`local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`。`jarde-source.txt` 保留 CLI 文本。CLI 结果中的 `@bytecode` 地址是此 javac classfile 的 BCI，不是 smali 地址。该拒绝没有被视作测试失败，也没有把 DEX 交给 Jarde。

本次固定源 SHA-256：JUnit `191f1041e514124eea94670afed560745d8ddac8ae3e1fd7baee549636e81b25`，smali `5ca0f91a19ddbb97ab7432638f77192068ca84686fed19fa079179488fa2b1be`。Java 8 目标 class SHA-256 为 `057e4fd522c67f3fdbf51d8abfd350a1eb11fdd3079f903a869a599b580317a7`，JADX adapter class 为 `b00c896dc9ae9f033fd3a9a18422443e5c8a3f722badb5f836d7ef814374882a`。这些 class hash 对应本次 JDK 23.0.1 `javac --release 8 -g:none` 输出。

本次工具链：JADX checkout 如上；JADX CLI 为该 checkout 的 `dev` 安装版；Java/Javac `23.0.1`；Jarde CLI 源码 HEAD 如上，执行文件 SHA-256 `e3ceb93bf2171d8c5d5eb67ef5c79b7f5e456405dd77365cfbf4b7d99560517c`。可将 CLI 路径和输出目录传给 `replay.sh`，例如：

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-test10-audit/replay.sh \
  /path/to/jarde-cli /tmp/cf16-test10-replay
```

这份事实审计不建议直接把固定测试升格成语义正例。若后续要支持该 Java 8 形态，应另以固定 javac classfile 明确目标异常行和 `IOException` handler 归属，并独立定义缺少 Jarde 输出时的验收，不可将 DEX 断言当作 classfile 验收。
