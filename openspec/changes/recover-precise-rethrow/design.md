## Context

自写场景（`Patrol.preciseRethrow`，javac --release 8）字节码：try 体两条条件 `athrow`（BCI 19、38），handler 在 BCI 42：`astore_2; aload_2; invokestatic log; aload_2; athrow; return`，异常表一行 `[0,39) → 42, Class Exception`。当前恢复被 `guard.rs::finally_copy`（`Unproven::FinallyCopy` → `jre_guard_finally_copy`）整方法拒绝：该候选把 `store; [含 invoke/field 的体]; load; athrow` 的 handler 识别为 finally 合成拷贝，而精确重抛的用户 catch 恰好同形。

`finally_copy` 的判别缺口是异常表的 `catch_type`：javac --release 8 的 finally 合成拷贝只由 any 行（`catch_type == 0`）进入（TWR close/addSuppressed 链同样）；用户 catch 的行具名类型。当前代码不读该字段。

## Goals / Non-Goals

**Goals:** 具名行的参数重抛子句按普通用户 catch 呈现；finally 候选加上 `catch_type == 0` 前置；三方对照与受控执行证据。

**Non-Goals:** 多捕获并集类型；`catch_type == 0` 行的任何改写；一般 handler 体 `throw` 新形状；`Exceptions` 属性拼写规则改动。

## Decisions

1. **catch_type 前置条件。** `finally_copy` 入口先判 `catch_type == 0`；具名行直接返回 `None`。这是最小且语义正确的判别：合成拷贝的到达边只有 any 行（老编译器的 `jsr` 路径不在本层支持范围）。备选「比较重抛值与 try 体抛出类型集合」需要跨方法数据流，拒绝。
2. **重抛子句的呈现。** handler 末条 `athrow` 的操作数与入口存储同一槽位时，子句体末写 `throw <参数名>;`——`recover-throw-statements` 的 throw 语句路径复用，值是 catch 子句已声明的参数局部，无新声明、无新转换。参数名沿用 `names` 的既定拼写（无 LVT 时按序数）。
3. **单一行前提。** handler 入口唯一行才适用；多行到达（多捕获）保持今天的行为，留给独立 change。
4. **不改动 TWR/monitor。** `close_of_level`、monitor 行的拥有权规则原样；本 change 只让具名行不再进 finally 候选。

## Risks / Trade-offs

- 误放行形似 finally 的用户 catch → 由 `catch_type == 0` 前置排除主要误判源；`prove_finally_copy` 对 any 行的完整证明链保持原强度，测试含 any 行负例。
- 重抛文本对 throws 声明的可编译性依赖精确重抛语义 → 报告的 `compile_status` 保持 `NotAttempted`，受控执行对照只覆盖 fixture 样例；不新增语义声明。
- 无 LVT 时参数名拼写与 jadx 的 `e` 不同 → 名称拼写是既有规则，不为本 shape 特判。

## Migration Plan

无数据迁移。行为开关随构建生效；golden/语料计数若因文本变化重录，按仓库既有流程。

## Open Questions

无。
