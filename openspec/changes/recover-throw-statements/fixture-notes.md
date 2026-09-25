## `p3-throw` fixture

本次只固定一份正面 Java 8 class：`ThrowProbe.class`。源码覆盖 `null`、参数异常、
`new`、调用返回值、显式 `checkcast`、条件双臂、命名 `catch` 和已有
`throws java.io.IOException` 声明。`ThrowEffects.java` 与 `ThrowProbeRunner.java` 只作为
JDK 对照的 source-only helper/runner；`ThrowGuardProbe.java` 的 `finally`、
`synchronized` 形状以及较低栈值形状只保留源码；合法边界的内存 `Code` patch 和 JVM
日志记录在 `openspec/evidence/java-syntax-2026-09-22/throws/refusal-boundaries/`。

编译命令为：

```text
javac --release 8 -g:none -d <temporary-classes> \
  ThrowProbe.java ThrowEffects.java ThrowProbeRunner.java ThrowGuardProbe.java ThrowStack.java
```

`ThrowProbe.class` 的 SHA-256 为
`0b7a0e314fc3c2ee0ecd2bd226033aa735374d9456bc502bfba0ac7b5845e8a3`，文件大小 946 bytes，
major version 52，class methods 9 个，带 `Code` 的方法 9 个（构造器加 8 个正面方法）。
`javap -verbose` 的正面方法与关键 BCI 为：

| 方法 | descriptor | throw / producer BCI |
| --- | --- | --- |
| `nullValue` | `()V` | `athrow=1` |
| `parameter` | `(Ljava/lang/RuntimeException;)V` | `athrow=1` |
| `allocation` | `()V` | `new=0`, `dup=3`, `ldc=4`, `invokespecial=6`, `athrow=9` |
| `call` | `()V` | `invokestatic=0`, `athrow=3` |
| `cast` | `(Ljava/lang/Object;)V` | `checkcast=1`, `athrow=4` |
| `conditional` | `(ZLjava/lang/RuntimeException;Ljava/lang/RuntimeException;)V` | `ifeq=1`, `athrow=5`, `athrow=7` |
| `namedCatch` | `(Ljava/lang/RuntimeException;)Ljava/lang/RuntimeException;` | `athrow=1`, handler `astore=2` |
| `checked` | `(Ljava/io/IOException;)V` | `athrow=1`, `Exceptions=java.io.IOException` |

原 class 使用 `java -Xverify:all` 执行 `ThrowProbeRunner` 的输入日志如下：

```text
null=java.lang.NullPointerException:false:0
parameter=java.lang.IllegalArgumentException:true:0
parameterNull=java.lang.NullPointerException:false:0
allocation=java.lang.IllegalStateException:false:0
call=java.lang.IllegalArgumentException:true:1
callFailure=java.lang.IllegalStateException:true:1
cast=java.lang.IllegalArgumentException:true:0
castBad=java.lang.ClassCastException:false:0
castNull=java.lang.NullPointerException:false:0
first=java.lang.IllegalArgumentException:true:0
second=java.lang.IllegalStateException:true:0
checked=java.io.IOException:true:0
caught=true
```

该 class 是 `javac --release 8` 的原始输入；Rust ignored JDK 对照会把同一完整正面类和
恢复后的完整 class-source 分别重编译，再使用 `-Xverify:all` 执行同一个 runner。测试不从
生成源码删除失败方法，也不把 source-only guard/stack 变体伪装成正面通过。

## 主代理复核与修前测试

root 独立复编译确认与冻结 class 逐字节一致；`throws/final-fixture-before/` 保存原 class、
JADX 和当前 jarde 的完整类输出、javac/执行日志及 javap。原 class/JADX 的13行完全相同；
当前 jarde 整类 javac 失败，不能把将来的 ignored JDK 断言当作已经通过。

`/tmp/jarde-throw-root-before-tests.log` 的专项 Rust 测试编译成功，结果为2通过、2预期失败、
1忽略：普通throw形态失败于nullValue仍引用athrow；来源测试失败于allocation的参数BCI4尚未
实际呈现。默认来源测试同时断言正文与all-evidence相同且每个方法确实返回RecoveryReport，
不静默跳过非Recovered状态；ignored JDK在original侧覆盖冻结class后执行，保证原侧与Engine
实际输入一致。本项生产实现和正面JDK对照仍未完成。

## 边界输入

不增加永久边界 class。Rust 测试从冻结 `ThrowProbe.class` 的完整 `method_info` 做唯一匹配的
内存 patch：`parameter` 变体在 `aload_0` 后插入 `aconst_null/astore_0`，`call` 变体在
producer 后插入 `dup/pop`，另一个 `parameter` 变体在参数读取前插入已有
`ThrowEffects.problem()` 调用和 `nop`，让带副作用的 lower stack value 被 `athrow` 清掉但不被
误读。三种 Code 变体都提高 `max_stack` 并同步更新属性和 code 长度；冻结输入仍为 major 52、
946 bytes、9 个 `Code` 方法，原始 SHA-256 仍为
`0b7a0e314fc3c2ee0ecd2bd226033aa735374d9456bc502bfba0ac7b5845e8a3`。

source-only `BoundaryRunner.java` 与原始/patch 后 `java -Xverify:all` 日志位于
`openspec/evidence/java-syntax-2026-09-22/throws/refusal-boundaries/`。原始参数、调用和 lower
结果分别保持参数身份、一次调用和零次调用；patch 后分别为
`parameter=java.lang.IllegalArgumentException:true:0`、
`call=java.lang.IllegalArgumentException:true:1`、
`lower=java.lang.IllegalArgumentException:true:producer=false:1`。每个 patch 后 class 的
SHA-256 记录在 `class-sha256.txt`，三者均保留 9 个 `Code` 方法。
