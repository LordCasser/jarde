## Why

2026-09-26 巡查（自写 `Patrol` 场景，javac --release 8 编译，jadx 1.5.6 与 jarde 三方对照）发现：Java 7+ 的**精确重抛**形状——`catch (Exception e) { …; throw e; }` 配合窄 `throws` 声明——在 jarde 中整方法退为字节码引用。根因在 `crates/jarde-java/src/guard.rs` 的 `finally_copy` 候选：它把「参数存储; 含 invoke/field 的体; 参数读取; athrow」的 handler 判为 finally 合成拷贝候选，而精确重抛的 handler 恰好长这样（`astore_2; aload_2; invokestatic log; aload_2; athrow`），`prove_finally_copy` 失败后整条行被 `jre_guard_finally_copy` 拒绝。

决定性判别事实是异常表：javac 的 finally 合成拷贝**只**由 `catch_type == 0`（any）行进入；精确重抛的行具名类型（如 `Class Exception`）。当前候选检测不读 `catch_type`。

jadx 1.5.6 同场景把它恢复为 try/catch，但把 `throws java.text.ParseException, java.io.IOException` 写成 `throws Exception`（宽化，违背原 class 的 `Exceptions` 属性）。jarde 的 `throws` 拼写本就来自已解析的 `Exceptions` 属性，修复后**比 jadx 更忠实**：窄 throws + 用户 catch + 参数重抛。

## What Changes

- `finally_copy` 候选检测 MUST 先要求该行的 `catch_type == 0`（any）；具名类型行不再进入 finally 拷贝候选，保持其用户 catch 身份。
- 具名 catch 行的 handler 体以重抛该行 catch 参数结尾（末条 `athrow` 的值是 handler 入口存储的同一局部）时，该子句按普通用户 catch 呈现，体末是 `throw e;`（`e` 为子句参数名）。来源保留 handler 入口存储与 `athrow` 的 BCI。
- 单一具名行覆盖该 handler 入口时才适用本形状；同一 handler 入口被多行到达（多捕获 `A | B` 编码为多行）不在此 change 范围，保持今天的处理。
- `catch_type == 0` 的行与 TWR/monitor 的既有证明路径不变；本 change 不把 any 行改写成用户 catch。
- 自写 Java 8 fixture 经 javac、jadx、jarde 三方对照；重编译执行比较两种抛出模式与正常完成路径的返回值与异常类型；jadx 对 `Exceptions` 属性的宽化偏离如实记录为 jadx 偏离，不作为 jarde 的通过条件。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增「具名 catch 行的参数重抛子句」呈现要求与 finally 候选的 `catch_type == 0` 前置条件。

## Impact

实现集中在 `crates/jarde-java/src/guard.rs`（候选检测、catch 子句建造）与 `build.rs`/`emit.rs` 的子句体末 `throw` 语句呈现，及对应 fixture/test。`recover-throw-statements` 已交付的 `throw` 值语句是本 change 的直接复用；TWR/finally 的行几何规则（`close_of_level`、`enclosing_clauses`）不变。无新 crate、pass、生产依赖。

非目标：多捕获（多行到同一 handler）的并集类型拼写；handler 体中非参数值的一般 `throw` 新形状；`catch_type == 0` 行的任何放宽；方法级 `throws` 拼写规则改动。
