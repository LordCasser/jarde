## Why

JADX `TestArrayInitField` 的纯字面量完整类已经通过原/JADX/Jarde双JDK对照，但带副作用的数组字段组因 `static int trace = 0` 被当前静态证明误判为会改变初始化阶段，无法提升成声明初值。`trace`并非final常量变量；只需要修正既有阶段准入条件，继续复用有序字段写与原子投影。

## What Changes

- 非final普通类静态字段的常量RHS不再仅因常量表达式而拒绝整个初始化组；完整表、唯一写、有序效果/读取、类型和来源证明继续适用。
- final字段的常量表达式阶段变化继续拒绝，接口final约束和ConstantValue混合拒绝继续保留。
- 用已冻结纯字面量/有序数组完整类复跑candidate源码，核对副作用、数组内容与所有物理成员/BCI；增加非final正例与final阶段变化边界测试。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 区分非final运行时静态字段初始化与final常量变量的阶段准入。

## Impact

`src/facade.rs::prove_static_initializer_group` 的一项准入及现有静态投影测试；无新IR、AST、pass、依赖或公开API。前置普通类静态投影已实现，冻结CLI与产品6dd的确切CI已验收；本片新实现须另行完整类与CI验收，不能借旧CI。

非目标：实例字段提升/构造器共同前缀、部分静态组、ConstantValue混合、前向读、异常边、未显式写字段。实例数组仍在构造器的呈现差异另行设计，不扩张本片。
