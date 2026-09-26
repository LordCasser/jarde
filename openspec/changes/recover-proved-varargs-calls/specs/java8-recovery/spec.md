## ADDED Requirements

### Requirement: Proved source-level varargs invocation

系统 SHALL 仅在物理调用目标、其可变参数声明、末参数组的完整内联初始化以及展开后的 Java 重载绑定均已证明时，把该数组写作展开的调用实参。输出 MUST 保留原有求值顺序、次数、异常行为及每个原始指令的来源。证据不完整时 MUST 保留显式数组实参，不得凭方法名或数组形状推断可变参数。

#### Scenario: Inline array to a unique varargs target

- **WHEN** 调用目标唯一且声明带 `ACC_VARARGS`，末参为数组，调用末参是完整闭合的直接数组初始化，并且展开形式仍选中准确的目标签名
- **THEN** 源码将末参元素按顺序写成独立调用实参，Java 8 重编运行与原 class 的值、效果和异常顺序一致，物理数组及元素写入的来源仍可追踪

#### Scenario: Ordinary array target or explicit array value

- **WHEN** 目标不带 `ACC_VARARGS`、末参并非数组，或调用传入的是局部/参数持有的数组
- **THEN** 源码 MUST 保留一个数组实参及目标的物理声明语义

#### Scenario: Binding or element ambiguity

- **WHEN** 目标解析不唯一、可见的同名重载可能截获展开形式，或单元素 `null`/数组可能被 Java 当作整个可变参数数组
- **THEN** 源码 MUST 保留显式数组调用；预算/取消使所需事实未完成时 MUST 不发布部分展开

#### Scenario: Cross-check against JADX

- **WHEN** 使用与 JADX `TestVarArg` 相同的简单正例和普通 `int[]` 负例构造 Java 8 class
- **THEN** 原源码、JADX、Jarde 的正例调用均可重编，Jarde 负例保留 `new int[]{...}`，运行行为与原 class 一致
