## ADDED Requirements

### Requirement: Proved single-site anonymous interface body

类源码视图 SHALL 在匿名实现的物理类、唯一的直接 `return new` 创建位置、无捕获且无可观察初始化效果的构造过程、单一接口和全部待呈现方法正文均得到完整证明时，把使用点写成 `new Interface() { ... }`。每个方法的修饰符、返回类型、参数和有序语句 MUST 与该物理实现一致；使用点原有的求值、异常与运行时类身份 MUST 不变。物理类及其方法的独立报告 MUST 仍可查询。任一证明缺失、扫描不完整或预算/取消停止时 MUST 保留未内联的物理类使用点，不得发布半个匿名体。

#### Scenario: One uncaptured interface implementation

- **WHEN** 一个匿名物理类只实现一个接口，其无参构造器仅调用 `Object()`，全部实现方法完整恢复，所选输入中仅有一处准确创建和构造调用，并且创建表达式被直接返回
- **THEN** 使用点包含 `new I() { ... }` 及全部实现方法，不再包含对该匿名物理类构造器的源码调用；原 class 与呈现出的入口类源码均可按 Java 8 重编运行且输出一致

#### Scenario: Two allocations in the same method

- **WHEN** 同一方法的两个不同字节码位置均创建同一匿名物理类
- **THEN** 两个使用点均保持物理类构造形式，不得生成两个运行时身份不同的源码匿名类

#### Scenario: Another selected class uses the identity

- **WHEN** 所选输入范围内另一类构造该匿名物理类、引用其类身份，或无法完整排除这些使用
- **THEN** 不得内联；调用者和物理类的独立呈现继续保留

#### Scenario: Constructor or body proof is incomplete

- **WHEN** 构造器有合成捕获参数、字段写入或实例初始化效果，匿名体任何方法存在引用缺口，或候选/所选范围扫描在预算或取消处停止
- **THEN** 使用点保持物理类构造形式，不得输出缺方法、空方法或已部分替换的匿名体
