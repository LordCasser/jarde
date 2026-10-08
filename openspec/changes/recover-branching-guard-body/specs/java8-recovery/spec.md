## ADDED Requirements

### Requirement: 分支体锁卫的呈现

当锁卫的保护体含分支、其 void 完成形的 transfer 终块被融合进释放副本所在块、且该块的尾 span 是直线 Return/Throw 时，系统 SHALL 按源码形态呈现整个卫（体含分支），方法行为完整。

#### Scenario: 分支体主锚
- **WHEN** 输入为固定 branching 形（guard 体内 if/else，`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且剥离编译后 `-Xverify:all` 双驱动（normal/异常）与原类一致

#### Scenario: 五族零回退
- **WHEN** 输入为 LK/IO/nested-lock/loop-test-copy 各族锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 非直线尾仍拒
- **WHEN** 融合块的尾 span 含 control flow（非直线 Return/Throw）或多重分支
- **THEN** 拒绝 SHALL 逐字保持
