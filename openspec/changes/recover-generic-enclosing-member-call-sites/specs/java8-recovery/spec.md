## ADDED Requirements

### Requirement: Proven generic enclosing member construction produces independently recompilable Java 8 callers

当选定环境唯一确定泛型外层类及其公开非静态成员类，外层泛型声明和成员关系与物理 class 一致，且调用点的封闭实例、构造器目标、源级参数及异常/效果顺序均获证明时，调用方源码 SHALL 将封闭实例写为限定创建表达式 `outer.new Member(args)` 或 `outer.new Member<>(args)`。调用方方法签名中所需的嵌套类型及每层类型实参、同一已选嵌套类上的静态调用限定符 SHALL 以 Java 源级名称呈现；物理合成外层参数 MUST NOT 作为源级构造实参出现。结果 MUST 能在原始目标 class 的 classpath 上由 Java 8 独立重编，不据此声称目标类族也已恢复。

#### Scenario: Raw generic outer and ordinary member

- **WHEN** 一个调用方参数为原始 `Outer.A`，返回 `Object`，创建 `A` 的公开非泛型非静态成员 `Plain`，所需类关系、构造调用与封闭实例证据完整
- **THEN** 调用方 SHALL 使用可解析的嵌套类型名和 `outer.new Plain(...)`；以原始类族作依赖独立重编及运行的返回类型、效果和异常次序 MUST 与原 class 一致

#### Scenario: Parameterized generic outer and ordinary member

- **WHEN** 调用方参数签名为 `Outer.A<String>`，返回 `Object`，创建公开非泛型非静态成员 `Plain`，签名的每层类型与其物理擦除一致
- **THEN** 调用方 SHALL 保留可编译的 `Outer.A<String>` 参数声明并使用已证外层实例创建；普通实参及物理来源 MUST 保持一致

#### Scenario: Parameterized generic outer and generic member

- **WHEN** 调用方参数签名为 `Outer.A<String>`，创建公开泛型非静态成员 `Generic<V>`，其构造器源级泛型参数与已证物理外层首参后的 descriptor 尾部一致，且返回类型为 `Object` 或 `Outer.A<String>.Generic<Integer>`
- **THEN** 两种调用方 SHALL 各自独立重编并保持其 Java 泛型声明；成员创建 SHALL 绑定原外层实例，且正常与 null 路径的返回值、参数效果次数和异常先后 MUST 与原 class 一致

#### Scenario: Ambiguous or contradictory nested evidence

- **WHEN** 外层或成员定义缺失/歧义、双向成员关系或静态性冲突、泛型实参数量与已选声明不符、签名擦除不符、构造器物理首参或源级尾部不符、封闭实例 SSA 身份不符，或空值检查晚于普通实参效果
- **THEN** 相应签名投影或创建站点 SHALL 保留可追源拒绝及物理 BCI，不得发布已证明的可重编限定创建，也不得猜测 `$` 名称的成员层级

#### Scenario: Evidence and stop integrity

- **WHEN** 同一已证调用点分别请求该完整类入口支持的必要/全部证据，或请求该入口不支持的 BCI 范围，或在目标/外层解析时发生预算耗尽或取消
- **THEN** 完整请求的源码决定 SHALL 一致，物理目标与实参来源 MUST 保留；不支持的范围请求及未完成解析 MUST 如实报告停止，不得把未证明调用作为成功输出
