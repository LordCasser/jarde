# Preliminary local-join `ireturn` narrowing audit

## 范围更正

本目录的第一版输入保留为基线，但它不能证明 `switch_returns` 的真实栈汇合，也不能证明
boolean 操作数到 B/C/S 返回的边界。`runByte`、`runChar`、`runShort` 的每个 arm 都是
“常量 → `istore` → join `iload` → `ireturn`”的普通局部变量路径；`boolByte`、
`boolChar`、`boolShort` 虽然以 boolean 参数控制分支，返回值仍是 int 常量，并非 boolean
操作数。真实的 arm 留值、共用单条 join `ireturn` 证据在
[`actual-stack-join/README.md`](actual-stack-join/README.md)，真实 boolean-presented
操作数证据在 [`boolean-operands/README.md`](boolean-operands/README.md)。

这是 `recover-narrow-integer-returns` 的 source-only 预备证据，不修改生产代码、Rust
测试、Cargo、永久 fixture、census 或 OpenSpec。`NarrowSwitchReturns.java` 用
`javac --release 8 -g:none` 编译；每个方法的两个 arm 都把一个 int 常量写入同一个局部，
之后共用一个真实的 join `iload`/`ireturn`。没有 call、field、array 或其它效果型
producer。

## 可重放方式

在仓库根目录执行：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-switch-returns/run_audit.py
```

脚本只在临时目录编译和运行，精确修改方法 descriptor，复制原始/补丁 class、`javap`、
JVM/JADX/jarde 日志到本目录，并检查固定 CLI `/tmp/jarde-cli-deferred-final-ecab` 的 SHA
前后一致。descriptor patcher 只新增 UTF8 descriptor 并修改 method_info descriptor index；
每个 `Code` attribute 的完整字节在补丁前后相同。

## class 与 BCI 事实

源 class 为 623 bytes，SHA-256 为
`8bbd3ca48fce4f11c8e4cc9ece31e09e7e0bb55b4cca293c905b0ea4d5c5ac17`；补丁 class 为 665
bytes，SHA-256 为
`3ea6861fdfc8ff741c55a1ee3ac61cb235b21281239e3e048d9be7d8deeab1de`。class major 为 52，
共有 7 个 `Code` attributes（私有构造器加 6 个测试方法），补丁前后合并 Code SHA-256
均为 `ecb398c4aee2b10e38595f112cf0c1a2a9b4a5c11aeae5093bef69b93d480db5`。

下表的 arm value BCI 是常量进入栈的位置，arm store BCI 是各分支写入共享 int 局部的
位置；真正的返回消费位置是最后一列 join `ireturn`，不是 arm 的 `at`：

| 方法 | 补丁 descriptor | arm value BCI | arm store BCI | join `iload` | join `ireturn` |
| --- | --- | --- | --- | --- | ---: |
| `runByte(int)` | `(I)B` | 20, 27 | 23, 30 | 31 | 32 |
| `runChar(int)` | `(I)C` | 20, 26 | 22, 28 | 29 | 30 |
| `runShort(int)` | `(I)S` | 20, 26 | 22, 28 | 29 | 30 |
| `boolByte(boolean)` | `(Z)B` | 4, 11 | 7, 14 | 15 | 16 |
| `boolChar(boolean)` | `(Z)C` | 4, 10 | 6, 12 | 13 | 14 |
| `boolShort(boolean)` | `(Z)S` | 4, 10 | 6, 12 | 13 | 14 |

`descriptor-patches.json` 保存 6 个原始 `(I)I`/`(Z)I` 到 B/C/S descriptor 的精确偏移和
method span，`patched-javap.txt` 保存完整 Code 与 StackMapTable。前 3 个方法是 switch
两臂，后 3 个只是 boolean 控制下的 int 常量负面观察；boolean 参数只控制分支，返回值
并非 boolean operand。

## 实际执行结果

源码 class 的 18 行输出是 int 返回值。补丁 class 通过 `java -Xverify:all`，同样输出 18
行；`patched-expected.txt` 按 JVMS `ireturn` 的返回 descriptor 计算并与补丁输出逐行相等：

* B：低 8 位有符号窄化，例如 `130 -> -126`、`-129 -> 127`；
* C：低 16 位无符号窄化，例如 `-2 -> 65534`、`65535 -> 65535`；
* S：低 16 位有符号窄化，例如 `32768 -> -32768`、`-32769 -> 32767`。

源运行与补丁运行数值不逐行相等是预期的 descriptor 语义差异，不是 class 无效；实际结论
是 `patched_runtime_matches_expected=true`，两阶段 exit 都为 0。

当前固定 ecab CLI SHA 前后均为
`ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`。CLI 对 6 个 join
`ireturn` 都返回完整来源的 `@bytecode`（quote 数 6），并在报告中把返回位置放在真实 join
BCI：例如 `runByte` 为 `@bytecode 32`、value 为 BCI 31；`runChar`/`runShort` 为 30、
value 为 29；三个 boolean 方法分别为 16、14、14，value 分别为 15、13、13。生成的
jarde 全类因此 `javac --release 8` 失败于 6 个“缺少返回语句”，没有把失败形状伪装成可
编译 Java，也没有执行生成类。

JADX 完整输出也不能用窄 descriptor 直接通过 `javac --release 8`：其恢复的局部类型/窄
返回在 `byte`/`char` 方法产生有损 int 转换诊断，实际失败日志保存在
`jadx-javac.log`，没有声称 JADX 运行一致。JADX 的完整文本仍保留在 `jadx.java.txt`。

## 对实现边界的说明

本基线只说明局部值的 join `iload` 与 `ireturn` anchor 需要分开记录；它不授权把 arm
位置当作栈 phi，也不授权泛化到一般控制流合并。实际 stack join 的 arm value、跳转和
单条 `ireturn` 由子目录中的精确 Code patch 单独验证。

旧类中的 `(Z)B/(Z)C/(Z)S` 三个方法只观察了 boolean 控制下的 int 常量路径，因此不证明
boolean-presented operand。独立 boolean descriptor patch、JVM 验证、完整 JADX/jarde
阶段和该边界的失败形状集中在 `boolean-operands/summary.json` 与其 README。
