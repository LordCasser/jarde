## ADDED Requirements

### Requirement: Proved single static member class source unit

当选定根类与唯一直接、无字段且非泛型的静态成员类的物理定义及双向成员关系准确一致，平凡无参构造器和其他方法正文均可完整呈现，根类从方法直接返回它的无参创建表达式时，根类源码 SHALL 写出嵌套的 `static class` 声明，根类中该方法的返回类型及构造表达式 MUST 使用可解析的成员源名。物理身份、构造次数、成员行为与原有求值顺序 MUST 保持一致；子类独立报告及原物理来源 MUST 继续可查询。关系、正文或预算证据不全时 MUST 保留物理类表示，不得只改名而遗漏声明。

#### Scenario: A directly nested static member

- **WHEN** 一个 Java 8 类有唯一已证的直接静态成员 `Leaf`，该成员无字段且非泛型，平凡无参构造器和方法均完整恢复，根类直接返回 `new Leaf()`
- **THEN** 根类源码含 `static class Leaf`、返回类型 `Leaf` 和 `new Leaf()`，使用该源码单元及其他独立依赖重编运行的值和效果与原 class 一致

#### Scenario: A dollar sign is part of a top-level name

- **WHEN** 同一输入还有独立顶级类 `Named$Top`，它没有指向该根类的准确成员关系
- **THEN** 它的独立声明及根类中对它的调用仍使用 `Named$Top`，不得将其放入根类或改成 `Named.Top`

#### Scenario: Relation or member body is unproved

- **WHEN** 子类缺失、有多个同名物理定义、双方成员行冲突、出现第二个直接成员候选、子类方法正文含引用缺口，或预算/取消使类级证明停止
- **THEN** 根类不发布半个嵌套声明或只修改的类型/创建表达式，保留可追溯的物理类源码和原停止状态
