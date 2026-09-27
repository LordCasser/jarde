## ADDED Requirements

### Requirement: 已证不适用重载下的词法外层 super 源码绑定

当一个命名成员类的 `Outer.super.m(args)` 桥、词法接收者和全部可见使用已按现有契约证明，且同名重载存在时，系统 SHALL 仅在生成源码的每个实参静态类型可证、所选目标可适用、所有竞争重载均可证不适用的情况下投影该调用。目标必须与桥内的物理调用定义一致，重编后的调用 MUST 保留原 class 的结果、求值次数和异常行为。无法完成任何一项证明时 MUST 原子拒绝家族投影并保留桥及物理来源。

#### Scenario: 严格子类参数重载不适用

- **WHEN** 直接父类同时声明 `pick(Arg)` 与 `pick(NarrowArg)`，且完整所选类层级证明 `NarrowArg extends Arg`；成员调用的生成源码实参静态类型准确为 `Arg`，桥的目标是 `pick(Arg)`
- **THEN** 系统 SHALL 输出 `Outer.super.pick(arg)` 并省略已闭合的 synthetic 桥；完整 Java 8 源码 SHALL 可重编，验证运行 MUST 与原 class 一致

#### Scenario: 实参源码类型不确定

- **WHEN** 字节码的桥参数为 `Arg`，但生成源码的实参可能被推断为 `NarrowArg`、`null`、原始类型装箱结果，或不能证明其静态类型与所需参数一致
- **THEN** 系统 MUST NOT 用桥 descriptor 猜测源码重载选择；家族投影 SHALL 拒绝并保留桥及调用点

#### Scenario: 竞争重载可能被选中

- **WHEN** 一个同名重载对生成源码实参可适用，或缺少可证明其不适用的类声明/层级事实
- **THEN** 系统 MUST NOT 把目标不明的调用写成 `Outer.super`；拒绝信息 SHALL 可定位到源码绑定证明

#### Scenario: 泛型与异常边界保持拒绝

- **WHEN** 目标或竞争者的泛型签名、varargs、桥方法、参数转换或 checked exception 使 Java 8 源码绑定无法在已证明子集内确定
- **THEN** 系统 MUST 保留当前原子拒绝，不得仅因同名方法的描述符不同就声称可重编
