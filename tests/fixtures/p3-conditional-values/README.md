# 条件值（Java 8）冻结输入

本 fixture 固定 `Region::If` 两臂在单一 stack Phi 汇合、再由 return 消费的最小缺陷、包含 7 类消费/边界的完整正向语料，以及一个合法 switch 多入口反例。输入源码与 runner 由证据目录逐字节复制或新建；永久 class 只放 subject，所有 runner 均写入临时目录。JVM 执行只使用 `-Xverify:all`。

| 类 | 版本 | 大小 | Code 属性 | SHA-256 |
| --- | --- | ---: | ---: | --- |
| `TernaryCore` | 52.0 | 424 B | 6 | `9c867c925fb0f42b7fe11af18bb4e90645fe6ac8fcd2718114100161378b6ce4` |
| `TernaryValues` | 52.0 | 1414 B | 15 | `a2e96a4bb220fc3a61f4635827c628b84ad4571d8e71fd9da42a4c0e94c9a80f` |
| `ConditionalBoundarySwitch` | 52.0 | 545 B | 7 | `453c00b9ebb0974d084aca7a28e465c52c42ab8b92112ac0e913eb9c8d55a7ee` |

生成器为 javac 23.0.1，命令：

```sh
javac --release 8 -g:none -d v8 TernaryCore.java TernaryValues.java ConditionalBoundarySwitch.java
RUN_DIR=$(mktemp -d)
javac --release 8 -g:none -cp v8 -d "$RUN_DIR" TernaryCoreRunner.java TernaryRunner.java
java -Xverify:all -cp "$RUN_DIR:v8" TernaryCoreRunner
java -Xverify:all -cp "$RUN_DIR:v8" TernaryRunner
```

## 最小 core

`TernaryCore.returned(Z)I` 的 Code 是 `iload_0; ifeq 10; invokestatic a; goto 13; invokestatic b; ireturn`。原 class 和 JADX 完整类的两行执行输出相同，SHA-256 为 `286ac0edd72de3d58073c2a6b7659c784f24ecf993ca8dffbad49965a42639bf`：

```text
true=7:trace=1
false=11:trace=2
```

冻结 jarde class-source 阶段成功产生完整类文本（证据 CLI SHA-256 `7747b60a17dc635f0e8402d867cb9e74f9472a62057eb4865dbd8f4a207dfb34`）；该未修改文本以 `javac --release 8` 编译失败，唯一诊断是 `returned` 缺少返回语句，因此没有 jarde 可执行输出。原 class 与现有证据逐字节一致。

## 完整样本

`TernaryRunner` 固定 16 个输入，依次覆盖直接返回、局部赋值、算术、调用实参、`null` 引用、重载选择、两臂分别抛错与单臂触发。原 class 和完整 JADX 类编译后输出一致，输出 SHA-256 为 `2228b678cae9dbf92439c619ef502c42c5579620dcbe713deb465695b8e9aa1a`：

```text
returned:true=7:trace=1
returned:false=11:trace=2
assigned:true=7:trace=1
assigned:false=11:trace=2
arithmetic:true=15:trace=1
arithmetic:false=23:trace=2
call:true=107:trace=13
call:false=111:trace=23
reference:true=null:trace=0
reference:false=value:trace=0
overload:true=string:trace=4
overload:false=string:trace=4
throwing:true:failA=throws:java.lang.IllegalStateException:a:trace=1
throwing:false:failB=throws:java.lang.IllegalArgumentException:b:trace=2
throwing:true:onlyA=throws:java.lang.IllegalStateException:a:trace=1
throwing:false:onlyB=throws:java.lang.IllegalArgumentException:b:trace=12
```

冻结 jarde 在整类 class-source 阶段成功，正文含 20 个 `@bytecode` 引用；随后 `javac --release 8 -g:none` 以 7 个 `missing return statement` 错误退出，分别位于上述七个条件值方法，未尝试执行。JADX 编译并执行成功；这些阶段数字来自 `openspec/evidence/java-syntax-2026-09-22/ternary-values/`，不将生成正文或日志复制进永久 fixture。

原始审计位于 `openspec/evidence/java-syntax-2026-09-22/ternary-values/`；root 独立复放目录 `root-7747/` 只作为审计来源，不随 fixture 复制。完整源码、runner 与 class 直接供后续 recovery 测试消费。

## 执行与字节码核对

在 javac 23.0.1 下，按上面的两步命令编译 subject 与临时 runner 后，两个 `java -Xverify:all` 进程均以 exit 0 结束。生成的 subject class 与表中 SHA/字节数相同；runner class 只写临时目录。完整 runner 的 16 行输出 hash 为 `2228b678cae9dbf92439c619ef502c42c5579620dcbe713deb465695b8e9aa1a`，最小 runner 的两行 hash 为 `286ac0edd72de3d58073c2a6b7659c784f24ecf993ca8dffbad49965a42639bf`。

输出逐项锁住求值行为：`assigned` 的 Phi 存到局部再读取，`arithmetic` 的乘法在汇合后执行，`callArgument` 的 `add` 在所选分支后执行一次（trace `13`/`23` 中末位 `3` 是一次 add）；`reference` 的真臂为 `aconst_null`，栈图类型为 `String`。`overloadChoice` 在真臂传 `null`、假臂传字符串，原 class 的真实目标是 `overload(Ljava/lang/String;)Ljava/lang/String;`，每侧 trace 均为 `4`，没有误选 `Object` 重载。真/假臂 helper 分别记录 `1`/`2`，并能抛不同异常类与消息；两项同时配置为 throw 时，执行仍只见所选臂的异常和一次 helper trace（`1` 或 `12`），证明未选臂不执行。相关逐条 BCI、`Methodref` 和栈图已存在于证据源 `openspec/evidence/java-syntax-2026-09-22/ternary-values/original-javap.txt`，没有把大段反汇编复制进 fixture。

## 明确缺少的拒绝边界

以下都是可由 Java 8 源码生成的合法控制流形状。switch 多入口已有永久 subject class 与 source-only runner；循环、异常和类型不明仍缺永久输入或实际恢复结果。root 的验收应确认 switch 保持普通 switch 结构，条件值规则不把它误认作 `?:`；不要求整个方法或整类拒绝。当前正例和原 class 执行不能代替这些恢复侧断言：

- **多入口（已有永久样本）**：`ConditionalBoundarySwitch.choose(I)I` 是合法 Java 8 的三路 `lookupswitch`。三个分支分别调用 `left`/`middle`/`fallback` 并写 `value`，前两臂在 BCI 32、39 各用 `goto` 跳到 BCI 46，default 臂在 BCI 45 存储后顺序落入 BCI 46；最终是 local value 的三前驱汇合。这既没有单一 `Region::If`，也不是一个仅有 true/false 两个前驱的栈 Phi，不能由条件值规则认领。fixture 中 class SHA/大小见表，源码和 runner 为 `ConditionalBoundarySwitch.java`、`ConditionalBoundarySwitchRunner.java`。`javac --release 8 -g:none` 产物经 `java -Xverify:all` 成功，输出见下文；条件值规则保持拒绝/普通 switch 原样的恢复结果待 root 用新 CLI 核对。
- **循环**：`static int loop(boolean again, int n) { int value = 0; while (again) { value = again ? n : -n; n--; again = n > 0; } return value; }`。循环头/回边的 Phi 不属于本 fixture 的单一 `if` 汇合证明。
- **异常边界**：`static int guarded(boolean condition) { int value; try { value = condition ? left() : right(); } catch (RuntimeException error) { value = 3; } return value; }`（在同一类声明 `static int left() { return 1; }` 与 `static int right() { return 2; }`）。处理器也向后续值汇合提供来源，不能把 handler 结果并入一个两臂表达式。
- **类型不明**：当两臂由不同引用类型生产，且分析输入没有可证明它们共同静态类型/重载目标所需的类层级事实时，必须拒绝。这里未冻结这种缺依赖输入，也未证明恢复器当前在该情形下的拒绝行为。

循环和处理器两个方法已从上述形状临时组成同一个源码类，以 `javac --release 8 -g:none` 编译，并由 `java -Xverify:all` 执行成功（输出 `1:2:3:1:1:2`）；临时 class 未保存。上述代码形状只界定需要测的合法 Java 控制流，不声明当前 jarde 已验证拒绝。类型层级不可知的有效 classfile 仍明确缺失；没有使用手工拼接或无法通过 JVM 验证器的 classfile。

### switch 多入口样本

永久 class `v8/ConditionalBoundarySwitch.class` 由 `javac --release 8 -g:none -d v8 ConditionalBoundarySwitch.java` 生成；版本 52.0、545 B、7 个 Code 属性，SHA-256 `453c00b9ebb0974d084aca7a28e465c52c42ab8b92112ac0e913eb9c8d55a7ee`。`choose(I)I` 的核心 Code 是 `iload_0; lookupswitch {0:28, 1:35, default:42}; invokestatic left; istore_1; goto 46; invokestatic middle; istore_1; goto 46; invokestatic fallback; istore_1; iload_1; ireturn`。三个不同前驱对 local slot 1 提供值；按该变更设计的准入条件，它不是单一 if-region 的双臂栈 Phi。

runner 只保留源码，编译产物写临时目录：

```sh
RUN_DIR=$(mktemp -d)
javac --release 8 -g:none -cp v8 -d "$RUN_DIR" ConditionalBoundarySwitchRunner.java
java -Xverify:all -cp "$RUN_DIR:v8" ConditionalBoundarySwitchRunner
```

实际输出（exit 0，SHA-256 `2c02c050e89418088bfdc6fc82ecb50aa5abdc1b2ef31711fb75a6ad05e7e962`）：

```text
0=101:trace=1
1=202:trace=2
2=303:trace=3
-1=303:trace=3
```

不同 trace 表明每个 selector 只执行一条 switch 臂。root 用 CLI SHA-256 `3c65e6ec6766b83163b29ea62e0b551ef902f76262f82538aa74db399b726754`、JADX 1.5.6 与原源码做完整类重编和 `-Xverify:all` 对照，三者均为上述四行；Jarde 保留普通 `switch`，没有 `?:` 或 `@bytecode`。完整正文与日志见 `openspec/evidence/java-syntax-2026-09-24/conditional-switch-boundary/`。此反例约束新增条件值规则不得错误认领三入口局部汇合，不要求整类拒绝。

## Loop, exception and unknown-type boundaries

`ConditionalBoundaryCases.java` plus `MarkerValue.java`, `LeftValue.java`, and `RightValue.java` add javac-generated Java 8 boundary inputs. The source-only runner exercises a loop containing an ordinary conditional, a `try/catch` around a conditional producer (including each selected arm throwing), and a `MarkerValue` join where only the subject class is supplied to the recovery scope. All resulting classfiles are valid javac output; they are not hand-patched. Rebuild with `javac --release 8 -g:none` and run with `java -Xverify:all`; the three-way original/JADX/current-Jarde record is under `openspec/changes/recover-conditional-values/evidence/boundaries/`.
