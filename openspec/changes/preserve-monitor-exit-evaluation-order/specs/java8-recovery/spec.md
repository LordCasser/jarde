## ADDED Requirements

### Requirement: 嵌套 synchronized 的 return 求值不得越过内层 monitorexit

当方法在嵌套 synchronized 的内层块内 `return expr;` 且字节码在配对内层 `monitorexit` 之前完成 `expr` 求值时，系统 SHALL 以保持求值发生在该 `monitorexit` 之前的方式呈现（return 保留在内层体内，或内层体内 temp 赋值 + 体外 return），SHALL NOT 呈现为内层空体 + 求值外移。

#### Scenario: Thread.holdsLock 判别锚（sync-return-timing 巡查）
- **WHEN** 输入为固定 `NL`（外层 `synchronized(LOCK)`、内层 `synchronized(NL.class)`、`return "n"+o`，`Box.toString` 返回 `Thread.holdsLock(NL.class)?"Y":"N"`，`javac --release 8`）的 class 并恢复
- **THEN** 去注释呈现文本经 `javac --release 8` 编译若成功，运行输出 SHALL 为 `nY`（与原 class 一致）；呈现 SHALL NOT 是内层空 synchronized 体 + return 在内层体外的求值外移形

#### Scenario: 单层同步零回退
- **WHEN** 输入为固定 `SR`（单层 return-in-monitor/跨锁局部）或既有 monitor 巡查 corpus
- **THEN** 渲染与本变更前逐字节一致（`retInside`/`localAcross` 原样恢复；`voidBody` 保持安全拒）
