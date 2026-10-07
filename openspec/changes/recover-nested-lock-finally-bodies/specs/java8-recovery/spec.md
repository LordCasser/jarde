## ADDED Requirements

### Requirement: 嵌套锁 finally 体与行外可抛获取的呈现

当锁卫的 finally 体是语句序列（其每一 unlock 的接收者与某一行前 lock 调用 SSA 同一、释放序为获取逆序），或行前获取调用可抛但位于全部异常行范围之外时，系统 SHALL 按源码形态呈现（含如实 throws 子句），方法行为完整。

#### Scenario: 嵌套双锁主锚
- **WHEN** 输入为固定 `ML`（`a.lock(); b.lock(); try { … } finally { b.unlock(); a.unlock(); }`，`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且整类剥离编译后 `-Xverify:all` 输出与原一致（`2`）

#### Scenario: 可中断获取
- **WHEN** 行前调用是 `lockInterruptibly()` 且位于全部异常行范围之外
- **THEN** 方法 SHALL 按源码形态呈现（throws 子句如实）

#### Scenario: 既有证书与负序形零回退
- **WHEN** 输入为 LK/IO 锚，或释放序非获取逆序/unlock 无对应 lock/行内可抛调用形
- **THEN** 渲染/拒绝 SHALL 逐字不变
