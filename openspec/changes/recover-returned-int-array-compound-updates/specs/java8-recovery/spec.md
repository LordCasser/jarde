## ADDED Requirements

### Requirement: Proved returned int array compound assignment

系统 SHALL 在原始int行数组/下标与RHS、dup2和后续dup_x2的准确身份、旧值读取/int加法、写回及紧邻int返回消费全部闭合时恢复返回更新后值的数组复合赋值。输出 MUST 保留一次求值、异常顺序、原行对象和新值语义及全部相关物理来源。未闭合、停止或取消 MUST 不发布局部表达式或认领部分写入范围。

#### Scenario: Scalar and rank-descended return values

- **WHEN** 一维/二维/三维int数组元素按同一已证明左值读加写回，新值经准确dup_x2仅供store和紧邻ireturn
- **THEN** 完整成员和类恢复成可编译Java，写回及返回值都与原程序一致，既有语句更新和postfix旧值结果不回退

#### Scenario: Effectful lvalue and RHS

- **WHEN** 行/index/RHS调用可观察，或外层/行数组为null、下标越界
- **THEN** 成功调用各一次且trace123，外层失败trace1、元素失败trace12，异常类别与次序及成功返回的新值保持

#### Scenario: RHS replaces the row

- **WHEN** 旧元素读后RHS把外层行替换为新行
- **THEN** 写回和返回依据原行的更新值，基线旧行17、新行100、返回17，不能再次读取外层行来选择store对象

#### Scenario: Unproved consumers and operand copies

- **WHEN** sum/row/index副本有额外用途，dup_x2类别/次序/身份或store/return消费不符，或者行/type与紧邻返回不能证明
- **THEN** 不发布该returned compound表达式，只保留其它独立已证明形式或完整拒绝及真实来源；row-Phi不因本片扩大支持

#### Scenario: Atomic publication and instruction origins

- **WHEN** 返回更新的证明/构建预算耗尽或取消，或依赖闭包失败
- **THEN** 既有停止/拒绝状态保持，不留下部分store/return认领；成功时复制、旧值读、add、写回、return与左值/RHS来源均可查询到同一真实物理方法
