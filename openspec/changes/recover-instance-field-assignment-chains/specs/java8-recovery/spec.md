## ADDED Requirements

### Requirement: 实例字段链式赋值的呈现

当 `dup_x1` 的值跨 n 个实例 putfield 存活、每一 putfield 的 receiver 是同一 `this` 的 SSA 值、且值的全部消费是这些 putfield 时，系统 SHALL 按字节码序呈现 n 个独立实例赋值（源求值序从右到左），方法行为完整。

#### Scenario: this 链主锚
- **WHEN** 输入为固定 `CP`（`this.a = this.b = this.c = 5`，`javac --release 8`）的 class 并恢复
- **THEN** 方法 SHALL 完整呈现且整类剥离编译后 `-Xverify:all` 行为一致

#### Scenario: 静态与复合零回退
- **WHEN** 输入为 CH 静态链或 SC/BF 复合锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 跨对象链仍拒
- **WHEN** 链中 putfield 的 receiver 非同一 SSA 值（如 `o1.a = o2.b = 5`）
- **THEN** 拒绝 SHALL 逐字保持
