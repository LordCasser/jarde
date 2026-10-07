## ADDED Requirements

### Requirement: 锁卫循环 finally 的呈现

当方法的形状是锁卫（`lock()` 为行前唯一会抛调用、保护体含循环、正常路径保存返回值后 `unlock()`、异常路径 handler 仅 `unlock()` 与重抛、两份 unlock 的接收者 SSA 同一）时，系统 SHALL 按 `lock(); try { … } finally { unlock(); }` 源码形态呈现，方法行为完整。

#### Scenario: 有界缓冲主锚
- **WHEN** 输入为固定 `LK`（take/put/tryLockQuick，`javac --release 8`）的 class 并恢复
- **THEN** 三法 SHALL 完整呈现且整类剥离编译后 `-Xverify:all` 输出与原一致

#### Scenario: TWR 与真两份副本零回退
- **WHEN** 输入为 TWR 系列锚或非锁形的真两份清理副本形
- **THEN** 渲染/拒绝 SHALL 逐字不变

#### Scenario: 非同一锁与自保护行仍拒
- **WHEN** 两份 unlock 接收者不同一，或 handler 含自保护行
- **THEN** 拒绝 SHALL 逐字保持
