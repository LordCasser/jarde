## ADDED Requirements

### Requirement: 条件物化 RHS 的字段复合呈现

当字段复合赋值的右操作数是一个已证条件值物化（两臂常量、唯一 join、无异常边），包括位于循环体内的形态时，系统 SHALL 将复合赋值按源码形态呈现（如 `this.ok = this.ok & (x > 0)` 或既有复合形），方法行为完整。

#### Scenario: 循环内布尔复合主锚（critical 15）
- **WHEN** 输入为冻结 `bi.jar` 的 `BI`（`ok &= x > 0` 在增强 for 内）并恢复
- **THEN** `earlyRet` SHALL 完整呈现且整类剥离编译后自带 main 输出与原 class 逐字一致（`false/false/false/false`）

#### Scenario: 既有 FieldCopies 锚零回退
- **WHEN** 输入为 CH 链 / SC 累积 / BF 复合锚
- **THEN** 渲染 SHALL 逐字节不变

#### Scenario: 物化副作用形仍拒
- **WHEN** 条件物化臂含赋值副作用、双比较嵌套或跨异常表
- **THEN** 拒绝 SHALL 逐字保持
