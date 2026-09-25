## Why

Java 8 多重命名 `catch` 中，`javac` 可把无效果的正常 `areturn` 放在异常表半开保护区间的末端之外，却与受保护的返回值生产者融合成一个规范块。Jarde 已恢复 `catch (A | B)`，但把该块引用为 `jre_region_exception_edge`，整类因此缺返回；[独立普通 catch 类与组合探针](../../evidence/java-syntax-2026-09-25/multicatch-finally-return/analysis.md)已确认原 class 与 JADX 的普通捕获行为相同。

## What Changes

- 在现有异常边结算中增加有界证明：仅当命名 `catch` 行与规范块匹配，且行尾后只有无效果的终止 `Return`，才允许将该块作为 `try` 的正常出口呈现。
- 保留半开异常范围、返回值来源及求值顺序；行尾后存在调用、字段访问、monitor、抛错、其它效果或无法证明的指令时继续引用。
- 用单独的 `PlainMultiCatch.choose` 做完整 Java 8 编译和 JVM 正向对照；同字节码形状的组合探针 `choosePlain` 交叉验证，`chooseFinally` 的 catch-all/重复清理属于现有 finally 专项，不在本变更内。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证明的命名 `try/catch` 可以在异常表保护范围边界后承接无效果终止返回，且不得改变 catch 的实际覆盖范围。

## Impact

实现限于 `jarde-java` 的私有 Region 异常边结算和定向测试，复用 `Catches`、规范块、SSA 与现有 `Return` AST/emitter；不新增公开 IR、异常图层或 pass，也不修改 finally、TWR、monitor 或 JVM reader。
