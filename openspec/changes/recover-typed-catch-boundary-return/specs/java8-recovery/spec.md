## ADDED Requirements

### Requirement: 命名捕获保护区间末端的正常返回可在证明后呈现

当 Java 8 方法的一个规范块同时含有受命名 `catch` 保护的返回值生产者和紧随保护区间末端的无效果终止返回，恢复器 SHALL 在确认处理器、返回值来源和异常覆盖范围均不改变后，呈现完整 `try/catch` 正常返回。多条同范围、同处理器的命名记录 SHALL 保持为一个多重 `catch`，其正文只发射一次。若无法证明范围末端后的指令不会产生可观察行为或抛错，或者返回值表达式被移出其原来的保护范围，恢复器 MUST 原子拒绝候选，并保留真实字节码来源与拒绝原因。

#### Scenario: 多重捕获后正常返回
- **WHEN** 冻结的 `PlainMultiCatch.choose(int)` 具有两条 `[0,32)`、同 BCI 33 处理器的命名记录，BCI 30 产生 `"ok"`，BCI 32 无效果返回该值
- **THEN** 完整类 SHALL 可用 Java 8 编译，四条 `-Xverify:all` 路径的返回文本与原 class 一致；正文 SHALL 包含一个 `catch (IllegalArgumentException | IllegalStateException)`，正常分支 SHALL 返回 `"ok"`，该方法不得再含 BCI 30/32 的引用

#### Scenario: 保护区间外有其它效果或可能抛错的指令
- **WHEN** 命名捕获行结束后，同一规范块还有调用、字段操作、monitor 操作、显式抛错或其它无法证明无效果的指令，之后才返回
- **THEN** 恢复器 MUST 保持该候选为引用，不得将范围外指令写入 `try` 而使其异常误入命名 `catch`

#### Scenario: catch-all 或未证明的异常边
- **WHEN** 块另有 catch-all、外部异常入口或不匹配的处理器记录，且没有独立证明完整的资源或 finally 结构
- **THEN** 本规则 MUST NOT 放宽异常边或把它当成命名 `catch`；现有拒绝和来源 SHALL 保留

#### Scenario: 完成语义不同的 finally 副本
- **WHEN** `MultiCatchProbe.chooseFinally(int)` 另含正常、命名捕获和异常出口的三份 cleanup 及两条 catch-all 记录
- **THEN** 本规则 MUST NOT 将三份副本单独当作普通返回恢复，也不得输出一个使正常路径 cleanup 执行两次的 Java `finally`
