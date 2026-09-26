## ADDED Requirements

### Requirement: 已证明的词法外层 super 方法调用

在已完整装配的命名非静态成员家族中，系统 SHALL 仅当静态合成桥的完整方法体、桥内准确分派目标、每个调用点的捕获接收者和可见使用闭包均可证明时，把该桥调用呈现为 Java 8 合法的 `Outer.super.method(args)`。投影 MUST 保持执行次数、参数与接收者求值顺序、异常覆盖以及与成员自身 `this.method()` 不同的目标分派。证明不全时 MUST 保留物理桥与调用来源并拒绝完整家族投影，不能以同名普通虚调用替代。

#### Scenario: 与成员自身调用目标不同

- **WHEN** Outer 直接继承的 Base、Outer 覆写和 Member 继承的 MemberBase 分别提供可辨返回值，而桥内 `invokespecial` 精确调用 Base 方法，成员调用点实参是被捕获的 Outer
- **THEN** 家族源码 SHALL 写 `Outer.super.value()`，成员自己的调用 SHALL 保持 `this.value()` 或等价动态分派；Java 8 重编和执行结果 SHALL 与原 class 一致

#### Scenario: 同类型普通参数不能投影

- **WHEN** 桥调用接收者来自与 Outer 同类型的显式 `other` 或另一个实例，而非经捕获字段证明的词法外层实例
- **THEN** 系统 MUST NOT 写 `Outer.super`，也 MUST NOT 省略仍被该调用使用的 helper；输出 SHALL 明确拒绝该桥投影并保留调用点

#### Scenario: 桥体目标或完成关系不同

- **WHEN** 桥内存在额外效果、另一个返回/异常出口、不同父类或接口目标、参数重排，或处理器覆盖与投影语句不等价
- **THEN** 系统 MUST NOT 把桥视为本要求的词法 super；必须保留可定位的物理方法/调用来源和拒绝原因

#### Scenario: 使用闭包不完整

- **WHEN** 桥在已选输入中还有未被改写的调用者、method handle/bootstrap 引用、未解析的使用位置，或读取预算/取消令检查未完成
- **THEN** 系统 MUST NOT 从家族文本删除桥，完整家族投影 SHALL 保守拒绝；不得因已见的一个成功调用点宣称所有使用都已处理
