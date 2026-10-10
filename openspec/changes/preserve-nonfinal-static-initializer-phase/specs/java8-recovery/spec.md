## ADDED Requirements

### Requirement: Runtime initialization phase for nonfinal static fields

系统 SHALL 在普通类静态初始化整组的所有成员、类型、有序读写与来源证明通过时，将非final字段的常量RHS保留为运行时声明初始化器，不得仅因RHS为常量表达式而拒绝整组。系统 MUST 保持原字段flags、初始化次序、可见副作用和数组对象身份，并保留物理初始化方法及指令来源。

#### Scenario: Nonfinal constant RHS beside an ordered array initializer
- **WHEN** 非final静态字段以常量初始化，邻近数组字段的元素包含有序调用，完整静态组通过其他证明条件
- **THEN** 系统 SHALL 按原写入顺序呈现完整字段声明初值，重编运行与原程序一致，并不得使非final字段成为ConstantValue常量变量

#### Scenario: Final constant RHS changes runtime initialization phase
- **WHEN** 原字节码的final静态字段没有ConstantValue、在运行时以常量表达式赋值，而声明提升会使其成为常量变量
- **THEN** 系统 MUST 拒绝整组提升并保留原物理初始化方法，不得提前初始化该字段或删除其来源

#### Scenario: Existing whole-group refusal and stop contracts
- **WHEN** 初始化组存在漏写、重复写、未认领效果、前向读、ConstantValue混合、异常边或预算/取消停止
- **THEN** 系统 MUST 沿现有整组拒绝或受限输出契约处理，不得因非final准入改变而提交字段前缀
