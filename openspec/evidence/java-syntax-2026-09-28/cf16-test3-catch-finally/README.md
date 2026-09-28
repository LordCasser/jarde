# CF16 / TestTryCatchFinally3 证据

本目录固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestTryCatchFinally3.java` 与已编译的 `TestTryCatchFinally3$TestCls.class`。原审计基线 Jarde (`b1243f2bd9784a65f00b0b7806afeb1892f0aab9`) 安全拒绝 `test`；本 change 的 fresh CLI 输出和方法级验收记录在 `jarde-out/`、`jarde-method-harness/` 和 `results/`。

JADX 测试中的活动代码断言有 5 项：foreach 语句、catch、`LOG.error(...)`、finally 和 `cls.unload()` 各一次。原测试在 [TestTryCatchFinally3.java](TestTryCatchFinally3.java)。固定源码 SHA-256 为 `d61f1f6b7c092af2e5d947c8def626887cb0dc8adb40d6c68dec3be96f432acd`；物理目标 class SHA-256 为 `9c3dc61868a81eb87f90228c5ddad4336ca4ae540621e5cb9b1589474c90e71e`。其余固定源与 class 哈希见 `results/source-and-class-sha256.txt`。

## 目标方法物理形状

目标是 `TestCls.test(ClassNode, List)`，descriptor 为 `(Ljadx/core/dex/nodes/ClassNode;Ljava/util/List;)V`。完整 `javap -c -v` 在 [TestCls.javap.txt](bytecode/TestCls.javap.txt)。其循环为 BCI 0–35，正常路径在 38–42 执行一次 `unload()`；catch handler 从 45 开始，在 46–53 调用 logger、58–62 再执行一次 `unload()`；finally handler 在 65–73 执行第三处 `unload()`，并在异常路径重抛原 Throwable。异常表原序为：

| ordinal | 保护区间 `[start,end)` | handler | 类型 |
| --- | --- | --- | --- |
| 0 | `[0,38)` | 45 | `java/lang/Exception` |
| 1 | `[0,38)` | 65 | catch-all |
| 2 | `[45,58)` | 65 | catch-all |
| 3 | `[65,67)` | 65 | catch-all |

最后一行只覆盖 BCI 65 的 `astore 4`，不覆盖 BCI 67–70 的 `unload()`。因此清理抛错不会再次进入 handler，而是直接向外传播。stand-in 的抛错开关只在辅助类中；原 `test` 的 BCI、opcode、四行异常表未改。

## 三方结果

`original-src/` 是原 `TestCls` 的 Java 8 转写；`standins/` 提供目标方法所需的最小类型签名和可注入的 `load`、visitor、logger、`unload` 抛错。转写源码用 `javac --release 8` 生成的目标方法 BCI、opcode、异常表与固定 class 完全一致，分别记录于 `bytecode/original-java8.javap.txt` 和固定 class 的 `bytecode/TestCls.javap.txt`。它不是 class 文件级复刻：`-g:none` 会去掉调试属性，常量池索引和外围测试类的 `InnerClasses` 元数据也不作为对照目标。物理固定 class 另行直接运行，结果在 `results/original-class-run.txt`。

固定 JADX 完整输出在 `jadx-src/`，由固定 checkout 的 Gradle CLI 从已固定 class 生成。原 class、原源码转写和 JADX 完整输出都通过 `javac --release 8 -g:none -Xlint:-options` 及 `java -Xverify:all`。三者逐路径输出相同；第四份 `jarde-method-harness/` 使用 fresh Jarde class-source 中原样抽出的 `test` 方法，并明确从原类声明补上 `LOG` 初始化，也通过相同验证。九条路径覆盖正常、`load` 的 `Exception`/`Error`、visitor 的 `Exception`/`Error`、logger 抛错、正常/catch/handler 路径上的 `unload` 抛错。每条记录事件顺序、调用次数、logger 收到的原异常身份及最终异常身份；四份输出字节相同，见 `results/*-run.txt`。其中 handler 自行抛错后只执行一次清理，清理抛错覆盖原 Throwable。

```text
normal events=load,unload load=1 unload=1 logs=0 loggedOriginal=false terminal=ok identity=other
logger_exception events=load,visitor,logger,unload load=1 unload=1 logs=1 loggedOriginal=true terminal=IllegalArgumentException:logger-failure identity=logger
unload_handler events=load,visitor,unload load=1 unload=1 logs=0 loggedOriginal=false terminal=IllegalArgumentException:unload-failure identity=unload
```

当前 Jarde `class-source` 输出在 `jarde-out/TestTryCatchFinally3$TestCls.java`，单方法原样恢复文本在 [test.recovery.txt](jarde-out/test.recovery.txt)。`test` 无 fallback，且只有一份 finally；定向 Rust 测试逐项核 BCI 0–74 的全部 34 个指令起点、唯一物理块归属、预算和取消后的空文本/空来源。未经补齐的 Jarde **整类**源码仍因 `<clinit>` 未恢复、`private static final Logger LOG` 未初始化而重编失败，见 `results/jarde-javac.stderr`；方法级 harness 的原声明/静态字段初始化不能算作 Jarde 整类成功。

`neighbors.py` 从固定 class 冻结七份 verifier 有效近邻，SHA 和 `java -Xverify:all` 结果在 `neighbors/sha256-and-verify.txt`。它们分别改动清理接收者、清理目标、自保护行覆盖清理调用、原 Throwable 重抛、正常清理后重新进入受保护正文、catch-all 覆盖范围，以及将正文 `List.iterator` 改成另一调用符号。最后一份专门验证 catch 参数的 SSA 绑定不会为不相干调用放宽；这些 control 的语义均不同于目标字节码。Jarde 定向测试全部拒绝唯一 finally，并核无半个 try 或静默消失的物理块。原基线对固定目标安全拒绝；本 change 只让固定目标从拒绝转为方法级结构恢复。

## 重放

在本 Jarde checkout 根目录执行：

```sh
openspec/evidence/java-syntax-2026-09-28/cf16-test3-catch-finally/acquire.sh
```

`acquire.sh` 可重新从固定 JADX checkout 获取原输入，随后运行旧基线脚本。实现验收执行 `sh openspec/evidence/java-syntax-2026-09-28/cf16-test3-catch-finally/accept.sh`：它从当前源码构建 fresh Jarde CLI、重放原/JADX/整类 Jarde、验证所有近邻、从 fresh class-source 生成方法级 harness、以 `javac --release 8` 和 `java -Xverify:all` 运行四方并逐字节比较事件结果。传入 `JARDE_CLI=/path/to/fresh/jarde-cli` 可复用已在同一工作树构建的专用 Cargo target。脚本的临时 Java 文件和默认临时 Cargo target 在退出时清理；验收记录在 `results/acceptance.txt`。
