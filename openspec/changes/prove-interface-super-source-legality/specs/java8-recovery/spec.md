## ADDED Requirements

### Requirement: Interface-qualified super calls require a legal source qualifier

当非构造 `invokespecial` 已证明以入口 `this` 为接收者时，恢复只有同时满足以下两项源码条件才 SHALL 写出 `I.super.m(...)`：调用 owner 在当前类的直接父接口名单中恰好出现一次且并不冗余；在请求选定环境的完整接口声明链中，该限定调用能唯一绑定到与原调用相符的可访问 default 方法。非冗余要求当前类的任何其它直接父接口均不等于、也不继承该 owner，且从当前类的直接父类沿完整父类链所实现或继承的任一接口也不等于、继承该 owner。父类链只在 JVM 固定根 `java/lang/Object` 终止；其它父类缺定义不得冒充该根或视为空接口。目标接口自己或其中途父接口的更近抽象声明 SHALL 遮蔽祖先 default，不能因更远祖先存在同名实现就放行。证据缺失、选择歧义、重载绑定未证、已证冗余或目标抽象时，恢复 MUST 保留包含原调用方法和 BCI 的缺口，不得改写为类 `super`、普通虚调用或其它接口调用。预算/取消停止 MUST 如实报告停止，且不得发布未完成的源码结论。拒绝 MUST 保留已延期接收者和实参生产者的来源及效果。

不涉及接口调用的 class-super、本类 private、构造器和普通调用 SHALL 保持其既有选择。报告的恢复质量、覆盖和执行状态 SHALL 如实反映缺口；JVM 可执行性本身不构成 Java 源码合法性证明。

#### Scenario: One direct interface declares its own default

- **WHEN** 当前类只有一个直接父接口 `I`，它声明唯一的目标 default，已证明的特殊调用引用它并以入口 `this` 为接收者
- **THEN** 恢复文本 SHALL 保留 `I.super.m(...)` 和原调用来源

#### Scenario: One direct interface inherits a default

- **WHEN** `Child` 是唯一直接接口、不声明目标方法而从选定的 `Parent` 唯一继承 default，原调用引用 `Child`
- **THEN** Java 8 恢复文本 SHALL 保留 `Child.super.m(...)`；完整源码重编执行 SHALL 与原 class 的 Parent default 结果一致

#### Scenario: An interface default delegates to a direct parent

- **WHEN** 当前声明本身是接口 `Child extends Parent`，其 default 方法用 `Parent.super.m()` 调用直接父接口唯一的 default
- **THEN** 恢复文本 SHALL 保留 `Parent.super.m()`；接口没有类父链这一事实不得使合法调用被拒绝，Java 8 重编执行 SHALL 与原 class 一致

#### Scenario: Unrelated direct interfaces preserve distinct dispatch

- **WHEN** 父类 `value` 返回 7，两个无关直接接口 `Left` 与 `Right` 的默认 `value` 分别返回 11、22，正文显式特殊调用两者
- **THEN** Java 8 恢复文本 SHALL 分别写出 `Left.super.value()` 和 `Right.super.value()`；重编后两个结果及组合结果 SHALL 与原 class 的 11、22、33 一致

#### Scenario: Redundant direct interface is refused

- **WHEN** 当前类同时直接实现 `Parent` 和 `Child extends Parent`，一条 verifier-valid 特殊调用的池项 owner 是 `Parent`
- **THEN** 恢复 MUST 拒绝写出 Java 8 不合法的 `Parent.super.m()`，并在相应方法/BCI 留下可追溯缺口；MUST NOT 改写成 `Child.super.m()` 或父类 `super.m()`

#### Scenario: Superclass makes a direct interface redundant

- **WHEN** 当前类直接实现 `I`，但选定的父类或其祖先已经实现 `I` 或其子接口；一条 verifier-valid 特殊调用仍引用 `I`
- **THEN** 恢复 MUST 拒绝写出 Java 8 不合法的 `I.super.m()` 并保留原方法/BCI；若父类或相关接口定义缺失、歧义或读取停止，MUST NOT 将未知当作非冗余

#### Scenario: Abstract declaration shadows an inherited default

- **WHEN** `Child` 是唯一直接接口、物理调用引用 `Child.value()`，其选定定义抽象声明 `value()` 而祖先 `Parent` 有 default
- **THEN** 恢复 MUST 拒绝写出不合法的 `Child.super.value()`，并保留原调用方法/BCI；MUST NOT 绕过抽象声明改写成 `Parent.super.value()`

#### Scenario: Intermediate abstract declaration shadows a more distant default

- **WHEN** 唯一直接接口 `Leaf` 不声明目标方法，它的父接口 `Mid` 抽象重声明目标，而更远祖先 `Parent` 有 default
- **THEN** 恢复 MUST 拒绝 `Leaf.super.m()`，不能绕过较近的抽象声明认定 `Parent` 的 default 可调用

#### Scenario: Missing or ambiguous hierarchy or method is not guessed

- **WHEN** 当前选定环境无法唯一、完整地证明其它直接接口、父类链及它们的接口闭包与调用 owner 的继承关系，或无法证明目标 default 的唯一可访问声明/源码绑定，或读取因预算/取消停止
- **THEN** 恢复 MUST 不发布该 `I.super` 源码结论，并如实保留来源与执行停止

#### Scenario: Refusal retains argument effects

- **WHEN** 冗余接口特殊调用的接收者或实参包含已延期且可能产生效果的调用
- **THEN** 缺口 SHALL 保留这些生产者的方法/BCI 来源，不能因拒绝外层调用而隐藏其求值
