## ADDED Requirements

### Requirement: Common instance array initializer projection

系统 SHALL 在普通类全部构造器与字段信息完整、且每个构造器直接调用super后拥有相同连续实例数组初始化前缀时，把已证明的一维primitive数组初值呈现为字段声明初值，并仅从各派生构造器正文移除对应赋值。系统 MUST 保持原字段类型/flags、每实例新数组、元素及构造器主体的求值和异常次序，保留原物理方法与指令来源。

#### Scenario: Identical effectful array prefix in every direct-super constructor
- **WHEN** 两个或更多已完整恢复的direct-super构造器立即赋值相同数组初值，元素包含同目标且同实参的static调用，其后构造器主体不同
- **THEN** 系统 SHALL 将共同数组初值写入一个字段声明，保留各super及其后不同主体，完整重编运行的每次调用数、数组值/身份和raw输出与原程序一致

#### Scenario: Literal and final arrays retain per-instance allocation
- **WHEN** 一个或多个direct-super构造器有可证明共同primitive数组字面量前缀，字段可为final，且字段顺序与写入顺序一致
- **THEN** 系统 SHALL 呈现数组初值并保持final属性，各次构造分别分配原程序对应的新数组，不得共享数组或产生标量常量变量阶段变化

#### Scenario: Equal call shape with different arguments
- **WHEN** 不同构造器的数组元素调用同一方法但实际参数不同，或数组类型/值/顺序不同
- **THEN** 系统 MUST 保留各构造器原赋值，不得仅凭opcode、目标名字或调用形状合并初值

#### Scenario: Constructor group is incomplete or unsafe
- **WHEN** 任一构造器this委托、信息不完整、漏写/重复写、依赖参数或this/field读取、存在handler、初值不紧邻super、不同前缀或声明顺序无法保留
- **THEN** 系统 MUST 不提交共同数组初值和相应构造器删除，沿既有呈现保留物理输入及明确的执行状态

#### Scenario: Stop and evidence selection preserve atomic presentation
- **WHEN** 共同初值分析/发射遇预算限制或取消，或同一输入分别选择默认与完整证据输出
- **THEN** 系统 MUST 不发布部分字段或仅部分构造器编辑，保持准确Stop；证据选择不得改变准入结果或完整恢复正文
