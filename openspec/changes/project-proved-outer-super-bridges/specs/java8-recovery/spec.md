## ADDED Requirements

### Requirement: 已证明的词法外层 super 方法调用

在已完整装配的命名非静态成员家族中，系统 SHALL 仅当静态合成桥的完整方法体、桥内准确分派目标、每个调用点的捕获接收者和可见使用闭包均可证明时，把该桥调用呈现为 Java 8 合法的 `Outer.super.method(args)`。投影 MUST 保持执行次数、参数与接收者求值顺序、异常覆盖以及与成员自身 `this.method()` 不同的目标分派。证明不全时 MUST 保留物理桥与调用来源并拒绝完整家族投影，不能以同名普通虚调用替代。

桥内 `MethodRef` 的 descriptor 只证明字节码目标，不证明重新编译的 Java 8 调用仍绑定该目标。投影前系统 MUST 核对选定父类及相关继承声明中的同名候选、泛型签名和调用所需的 checked exceptions；无法证明生成源码的绑定和编译合法性时 MUST 拒绝，而不能仅凭原 class 可执行就删除桥。

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

#### Scenario: 源级重载或泛型绑定不明

- **WHEN** 桥的准确字节码目标已证明，但所选父类/继承链中还有同名候选、相关 `Signature` 改变源级参数化，或生成实参的转换可能改变重载选择
- **THEN** 系统 MUST 先证明 `Outer.super.method(args)` 在 Java 8 下仍选中桥内准确目标及参数求值；证明不了时 SHALL 保留物理家族文本并说明拒绝，不能把桥体证明等同于源级绑定证明

#### Scenario: checked exception 表面不明

- **WHEN** 目标方法声明的异常可能要求调用处有 Java 源码 `catch` 或 `throws`，而已发射的成员方法无法证明满足该要求
- **THEN** 系统 MUST NOT 发布省略桥的家族源码；即使原字节码能通过 verifier，也不能宣称生成源码可重编
