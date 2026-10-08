## ADDED Requirements

### Requirement: 分支体锁卫的呈现

当锁卫的保护体含分支（顺序/嵌套分支与体内循环经区域 walk 呈现）、其 void 完成形的 transfer 终块被融合进释放副本所在块、且该块的尾 span 恰为方法自身的无值 `Return`（布局中立判据——不采用会使 throw 尾随布局改变准入的通用尾读法）时，系统 SHALL 按源码形态呈现整个卫，方法行为完整。（root 2026-10-08 验收时按实测判据更正 MVP 措辞。）

#### Scenario: 分支体主锚
- **WHEN** 输入为固定 branching 形（guard 体内 if/else，`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且剥离编译后 `-Xverify:all` 双驱动（normal/异常）与原类一致

#### Scenario: 五族零回退
- **WHEN** 输入为 LK/IO/nested-lock/loop-test-copy 各族锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 非直线尾仍拒
- **WHEN** 融合块的尾 span 含 control flow（非直线 Return/Throw）或多重分支
- **THEN** 拒绝 SHALL 逐字保持
