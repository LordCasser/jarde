## Why

同一份 Behinder 应用类（`behinder-app.jar` 中与 jadx 1.5.6 共同产出的 115 个 `net/rebeyond` 类，2026-09-22，本机 release 二进制）上，jarde 弱在结构没写出来，不是弱在引用写得不够好看：

| 结构 | jadx | jarde |
| --- | --- | --- |
| `catch (` | 692 | 11 |
| `for (` | 369 | 0 |
| `while (` | 202 | 21 |
| lambda 箭头 | 402 | 0（另有 1059 处 `lambda$` 方法名） |
| `synchronized (` | 23 | 0 |
| `access$N(` | 0 | 189 |

62 个类里 jadx 有 `catch`、jarde 一个都没有。根因在 `crates/jarde-java/src/region.rs` 的 `region_at_inner`：分支或守卫证明不了时，已走完的前缀和当前块合成一个 `Region::Fallback`，后继为 `None`。`build.rs` 对 fallback 只发引用。`FileService.uploadFile` 因此整段是 `explanation_only`，尽管块 0 之前的语句本可呈现。

`emit.rs` 已经在 `else_body` 为空时省略 `else`。单臂 `if` 被拒绝，是因为走法里「不印空 else」那条规则，不是 AST 做不到。

已有规格要求无法证明的 try-with-resources 降级并保留 close/suppress 顺序（`java8-recovery` 的 TWR 场景）。本 change 不把那条失败改写成普通 `catch`。上一轮口头门槛里「TWR 失败可退回 try/catch」不采纳。

## What Changes

- 一个方法里已经证明的前缀 MUST 作为语句留下。证明不了的形状只引用自己的块，不得把前缀吞进引用，也不得收成空的 `if` / `catch` / `switch`。
- 两后继之一就是直接后支配点时，MUST 写成没有 `else` 的 `if`。另一臂被证明每条路径都是 `return` 或 `throw` 时，MUST 写成 `if/else`，且该 `if` 之后没有汇合代码。证明不了就留局部缺口。
- 异常表里 `catch_type` 非 0、且守卫检查认定该区域不是 try-with-resources 或 monitor 的记录，MUST 写成带该类型的 `catch`。`catch_type` 为 0、以及 TWR/monitor 规则声明拥有的区域（包括按其形状拒绝），MUST NOT 改写成 `catch` 或 `finally`。
- 离开循环的边只有在目标就是该循环的出口、头部或已证明的更新闩锁时才写成 `break` / `continue`。计数循环只在「一条无副作用初值 + 头部测试 + 闩锁对同一局部的一条更新」同时成立，且移入头部不改变入边与作用域时写成 `for`，否则保持 `while` / `do-while`。
- `accessor@1` 证明为纯字段转发的调用点 MUST 写成字段读写。lambda 合成方法的恢复结果含语句且捕获对得上时，使用点 MUST 呈现该体；对不上则保持转发调用。匿名类内联除无名、在所选物理输入范围内唯一分配点和方法体完整外，还必须证明基类构造实参与合成捕获实参的分界、捕获字段读取的词法替换和创建时求值顺序；证明不全则保留单独类，不把合成捕获参数写进匿名类的 `new Base(...)`。合成方法与未内联的匿名类 MUST 仍出现在类文本里。
- 类源码视图的声明拼写：有包名时写 `package` 与简单名（`$` 保留）；`throws` 只来自已解析的 `Exceptions` 属性；方法的 `ACC_VARARGS` 且末参是数组时，末参写成 `T...`。字符串字面量中已证明的 Unicode 标量按字符写出，不再一律 `\uXXXX`。
- `content` 三值、正常流视图不含异常边、以及「不引入字节码里不存在的符号」保持不变。不新增依赖。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增可观察要求——局部缺口不吞已证明前缀、命名 `catch`、已证明的循环出口与 `for`、已证明的 accessor/lambda/匿名类使用点、以及类源码声明与字面量拼写。既有 TWR、monitor、content 平面与 fallback 降级要求继续成立。

## Impact

实施落在 `jarde-java` 的区域走法与语句构建（`region.rs`、`build.rs`、`guard.rs` 只负责不要把普通 `try` 认成 TWR）、发射（`emit.rs` 的字面量转义）、以及 `src/class_source.rs` 的类装配。`Exceptions`、`InnerClasses`、`EnclosingMethod` 已由 `jarde-reader` 的 `AttributeFacts` 解析；本 change 只把这些既有事实交到装配层，不新写属性解码，也不把它们放进单方法 IR。

`jarde-jvm` 的 CFG/SSA 语义不变。正常流视图继续丢掉异常边。不改 `RecoveryContent` 的三个取值，不改 bulk JSONL 的记录种类。

验收 MUST 使用本 change 自行编写的 Java 源码场景，而不是 Behinder 语料。每个场景经 `javac` 编译后，用同一组 class 文件跑 jadx 1.5.6 与 jarde 的类源码视图。对照记录同时留下源码、jadx 文本、jarde 文本。手写源码提供该语法可表达的正例，不要求从非单射的字节码逐字还原它；语义与声明事实以原 class 的执行、异常顺序和 classfile 属性为准。要求不写的结构（失败的 TWR、`finally`、空 `switch`、外部符号）即使 jadx 写了，jarde 也不得照抄。与 jadx 逐字相同不是通过条件。Behinder 计数只作对照，不作为勾选任务的门槛。

非目标：`finally`（`catch_type == 0` 不写成 `finally`）、泛型与 `Signature`、注解与 `@Override`、import、bridge 隐藏、嵌套 jar 的自动展开、包前缀 scope、默认预算、`classes_refused` 记账、数组参数槽宽、T5 的 `append(int)` 消费 `char`。字符串 `switch` 的 `hashCode` 分派不在本 change 的完成条件里；本 change 不得把未证明的 `switch` 收成空 `switch`。jadx 不进入生产依赖，也不是正确性 oracle。
