## ADDED Requirements

### Requirement: 已证明的枚举构造器委托

当一个 Java 8 枚举的两个常量及隐式成员均可完整证明，且零源参数常量选择无源参数构造器、该构造器只委托到同类唯一的整数参数构造器时，类源码 SHALL 保留常量的不同调用形状和两条源码构造器声明；JVM 注入的 name/ordinal 参数与 `Enum` super 调用 MUST NOT 出现在源码声明或正文。委托值、用户调用、字段写入的顺序与次数 SHALL 与原 class 相同；重编后的构造器反射参数数目集合 SHALL 与原 class 相同。证明不完整时 MUST 不把零参数常量改写成直接带整数参数的常量，也 MUST 不删除无参构造器或其它可观察效果。

#### Scenario: 零参数常量委托，另一常量直接传参

- **WHEN** 两个常量按物理顺序为 `ZERO` 与 `ONE`，前者调用无源参数构造器，该构造器以整数 `0` 委托到整数构造器，后者以整数 `1` 直接调用整数构造器，且两条路径的其余效果都已证明
- **THEN** 完整类 SHALL 写出 `ZERO, ONE(1)`、无源参数构造器中的 `this(0)` 和整数构造器；Java 8 重编后的 `values`、name、ordinal、实例值、用户调用次数/顺序及反射构造器参数数目集合 SHALL 与原 class 相同

#### Scenario: 错误委托边或实参不能被猜测

- **WHEN** 无源参数构造器未委托到已选整数构造器，未原样传递隐式 name/ordinal，委托实参不是已证明的 `0`，或存在额外调用/异常路径
- **THEN** 类源码 MUST 不宣称已恢复此委托形状，MUST 保留原始构造器及常量的可追溯来源，不得把 `ZERO` 猜成 `ZERO(0)` 或移除其构造器

#### Scenario: 终端构造器和枚举辅助成员不完整

- **WHEN** 整数构造器的用户调用/字段写入次序未被完整恢复，或两个常量、隐式数组、`values`/`valueOf` 任一无法证明
- **THEN** 本规则 MUST 拒绝整组源级投影，不得只隐藏看似编译器生成的成员或吞掉用户效果

#### Scenario: 首片终端用户效果的边界

- **WHEN** 终端构造器只含一条对枚举类以外合法 Java owner 的静态 `(I)V` 调用、一次对精确 `private final int` 字段的实例写入和终结 `return`，且两次参数读取都由 Code slot 3 与同 BCI AST 证明
- **THEN** 这些效果只有在 Methodref/Fieldref、同轮 AST 次序、物理 Signature 与完整 Code 消费全部一致时才可成为私有证明事实；其它 helper 调用分派或字段形状 MUST 拒绝，且本阶段仍 MUST NOT 投影整组源码

#### Scenario: 原始成员与停止仍可检查

- **WHEN** 对同一输入请求默认/完整来源或 JSON 报告，或者证明与发射触发预算/取消
- **THEN** 成功请求的类源码正文 SHALL 不受来源选择影响，报告 SHALL 保留每条物理构造器、常量字段、方法的身份及原始运行结果与真实 Code 来源；停止 SHALL 如实呈现，MUST 不发布半个构造器委托投影
