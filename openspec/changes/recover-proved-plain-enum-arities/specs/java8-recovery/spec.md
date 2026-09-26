## ADDED Requirements

### Requirement: Proven ordinary Java 8 enum constants across arities

系统 SHALL 在所选 Java 8 物理枚举类的常量、隐式构造、值数组、标准辅助方法和其余成员使用关系均完整且一致时，将零个或多个普通常量呈现为合法 Java 枚举源码；该源码与必要的独立依赖在 `javac --release 8` 下 MUST 能重编，执行效果 MUST 与原 class 一致。系统 MUST 保留各物理字段、方法及其原始恢复结果和来源；只有整组证明通过才能在类源码中省去编译器脚手架。证据不全、预算停止或额外使用未证明时 MUST 保留物理呈现，不得只隐藏其中一部分。

#### Scenario: Empty enum with no user members

- **WHEN** 一个 Java 8 枚举声明零个常量，且隐式构造、零长值数组与标准辅助方法得到完整证明
- **THEN** 类源码 MUST 写成可编译的空枚举，不得把 `$VALUES`、隐式构造器或 `$values()` 当作用户声明；`values().length` MUST 与原 class 同为零

#### Scenario: Empty enum with a user method

- **WHEN** 上述零常量证明成立，类另有完整可呈现的用户方法
- **THEN** 类源码 MUST 在用户方法之前保留 Java 语法所需的空常量列表分隔，并保留该方法行为

#### Scenario: One or four plain constants

- **WHEN** 所选物理枚举分别有一个或四个普通常量，每个 name、ordinal、构造与字段写入一一对应，隐式无源参数构造器无用户效果
- **THEN** 类源码 MUST 按物理常量顺序写出恰好相同数量的常量；`values()` 顺序、`valueOf()`、身份及可观察结果 MUST 与原 class 一致

#### Scenario: Existing two-constant projection remains valid

- **WHEN** 输入满足已证明的两常量带源整数参数枚举或两常量匿名常量体模式
- **THEN** 本要求 MUST NOT 改变其已验收的常量参数、正文、用户初始化、物理报告和拒绝边界

#### Scenario: Hidden member has another physical use

- **WHEN** 某个未被本次投影消隐的物理方法额外读取拟隐藏的值数组、常量字段、构造器或标准辅助方法，或相关 Code/引用无法完整核对
- **THEN** 类源码 MUST NOT 发布部分枚举投影；公开物理成员结果和拒绝事实 MUST 仍可查询

#### Scenario: Stopped or ambiguous proof

- **WHEN** 类表、方法体、唯一引用关系或源码发射因预算、取消或歧义不能完成
- **THEN** 本次请求 MUST NOT 把未证的编译器成员隐藏成可编译枚举假象，也 MUST NOT 发布半个常量列表
