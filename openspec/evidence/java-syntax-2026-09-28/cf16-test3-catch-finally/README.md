# CF16 / TestTryCatchFinally3 证据

本目录固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally3.java` 与已编译的 `TestTryCatchFinally3$TestCls.class`，并记录当前 Jarde (`b1243f2bd9784a65f00b0b7806afeb1892f0aab9`) 对同一 class 的恢复结果。这里只做可重放证据，不改恢复实现或 OpenSpec。

JADX 测试中的活动代码断言有 5 项：foreach 语句、catch、`LOG.error(...)`、finally 和 `cls.unload()` 各一次。原测试在 [TestTryCatchFinally3.java](TestTryCatchFinally3.java)。固定源码 SHA-256 为 `d61f1f6b7c092af2e5d947c8def626887cb0dc8adb40d6c68dec3be96f432acd`；物理目标 class SHA-256 为 `9c3dc61868a81eb87f90228c5ddad4336ca4ae540621e5cb9b1589474c90e71e`。其余固定源与 class 哈希见 `results/source-and-class-sha256.txt`。

## 目标方法物理形状

目标是 `TestCls.test(ClassNode, List)`，descriptor 为 `(Ljadx/core/dex/nodes/ClassNode;Ljava/util/List;)V`。完整 `javap -c -v` 在 [TestCls.javap.txt](bytecode/TestCls.javap.txt)。其循环为 BCI 0–35，正常路径在 38–42 执行一次 `unload()`；catch handler 从 45 开始，在 46–53 调用 logger、58–62 再执行一次 `unload()`；finally handler 在 65–73 执行第三处 `unload()`，并在异常路径重抛原 Throwable。异常表原序为：

| ordinal | 保护区间 `[start,end)` | handler | 类型 |
| --- | --- | --- | --- |
| 0 | `[0,38)` | 45 | `java/lang/Exception` |
| 1 | `[0,38)` | 65 | catch-all |
| 2 | `[45,58)` | 65 | catch-all |
| 3 | `[65,67)` | 65 | catch-all |

最后一行只覆盖 BCI 65 的 `astore 4`，不覆盖 BCI 67–70 的 `unload()`。因此这份表不能证明清理抛错会再次进入 handler；清理抛错会直接向外传播。当前重放只覆盖正常与 visitor 抛出 `IllegalStateException` 的路径，其他抛错路径由后续实现任务补齐。

## 三方结果

`original-src/` 是原 `TestCls` 的 Java 8 转写；`standins/` 只提供目标方法所需的最小类型签名，并给 load、unload、logger 和 visitor 结果计数。转写源码用 `javac --release 8` 生成的目标方法 BCI、opcode、异常表与固定 class 完全一致，分别记录于 `bytecode/original-java8.javap.txt` 和固定 class 的 `bytecode/TestCls.javap.txt`。它不是 class 文件级复刻：`-g:none` 会去掉 `LineNumberTable`、`LocalVariableTable` 与 `LocalVariableTypeTable`，常量池索引和外围测试类的 `InnerClasses` 元数据也不作为对照目标。转写类的外围 JUnit 测试壳与 JADX 测试基础类不参与编译；物理固定 class 也另行直接运行，结果在 `results/original-class-run.txt`。

固定 JADX 完整输出在 `jadx-src/`，由固定 checkout 的 Gradle CLI 从已固定 class 生成。原 class、原源码转写和 JADX 完整输出都通过 `javac --release 8 -g:none -Xlint:-options` 及 `java -Xverify:all`。三者输出相同：

```text
normal load=1 unload=1 logs=0
visitor_exception load=1 unload=1 logs=1 logged=java.lang.IllegalStateException: visitor-failure
```

当前 Jarde `class-source` 输出在 `jarde-out/TestTryCatchFinally3$TestCls.java`，单方法原样恢复文本在 [test.recovery.txt](jarde-out/test.recovery.txt)。`test` 被拒绝，拒绝理由是 `local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice`，并保留 BCI `0 11 20 38 45 65 74` 的回退标记。整类 Java 8 重编按原输出失败：Jarde 没有恢复 `<clinit>`，所以 `private static final Logger LOG` 未初始化。没有运行该失败编译的类，也没有把只保留注释的空 `test` 方法算作恢复成功。

## 重放

在本 Jarde checkout 根目录执行：

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-test3-catch-finally/acquire.sh
```

`acquire.sh` 会先校验 JADX HEAD、测试源码与物理 class 的固定 SHA，再由 JADX Gradle CLI 和当前 Jarde CLI 重新生成两边输出。它在独立 `/private/tmp` 目录构建 Jarde CLI 和 JADX 导出，退出时清除两者；随后 `run.sh` 重编三份完整源并运行可编译者，Jarde 编译失败作为预期拒绝记录。`run.sh` 使用独立临时目录并自动清理 Java class 文件。运行结果、版本、源/class 哈希与编译诊断均在 `results/`。
