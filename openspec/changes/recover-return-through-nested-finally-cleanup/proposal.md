# 恢复经嵌套副作用 finally 的返回

## Why

封存的 `EX.nestedFin(I)I` 会在执行嵌套 `try/finally` 清理前计算并保存返回值。当前报告仍为 mixed/fallback，且清理块未覆盖。四条 catch-all row 分别描述正常清理、算术主体异常路径、嵌套清理异常路径和清理处理器绑定保护。若把这些 row 当作等价直线副本合并，可能漏掉清理调用或改变最终逃逸的异常。

## What changes

仅在首轮只读 Facts/SSA/effects 探针证明实际图、值身份、受保护调用和 block ownership 后，新增一个有界恢复叶片。保留保存的返回值、算术异常传播、清理顺序、恰好执行一次和 Java finally 覆盖规则。其余完成形式继续 fallback。

独立 fixture 的目标类只含构造器和与 EX 同体的 `nestedFin`。单独的 driver 临时把 `System.out` 替换为记录型 `PrintStream`，在 `in`/`out` 调用处注入已知 `RuntimeException` 或 `Error` 对象；不改目标方法。

## Scope

本叶片仅覆盖有界 `nestedFin` 的正常返回和除法异常完成，以及独立的双编译器匹配 fixture。不覆盖 finally 内 return/throw、添加新主体形状的 helper method、任意嵌套异常表，或相邻的仅异常完成 `tryThrowsFinallyRuns`。
