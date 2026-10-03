## ADDED Requirements

### Requirement: 构造器合成存重排仅在构造期可见性可证不变时进行

系统 SHALL 仅在满足以下之一时把 pre-super 合成字段直传存组移至 super 之后呈现：super 调用目标恰为 `java/lang/Object.<init>()V`；或 super 目标类在本快照有物理定义且其构造器可证明构造期无法到达子类覆写（既无 `this` 上的 `invokevirtual`/`invokeinterface`，也不把 `this` 作为实参传给任何调用）。其余情况 SHALL 保持既有逐字呈现与诊断，不得重排、不得改为整方法拒绝、不得丢弃该构造器。已安全重排的正例呈现 SHALL 逐字不变；系统 SHALL NOT 产出"可编译且行为不同"的构造器文本。

#### Scenario: 构造期虚分派反例不重排

- **WHEN** `AnonymousSuperDispatch$1`（`putfield val$captured` → `invokespecial Base.<init>`，Base ctor 内 `invokevirtual observe()` on `this`）经三方 Java 8 重编运行
- **THEN** 呈现保持捕获写入早于 `super()` 的原顺序，且该文本要么不可编译、要么运行保持 `visibleDuringSuper=true`——不得出现可编译且 `visibleDuringSuper=false`

#### Scenario: super ctor 无 this 分派时仍重排

- **WHEN** 输入为 `AnonymousSuperArgs$1` / `AnonymousCaptureCases$1`（super 类在快照内且其 ctor 仅含 Object.<init>、自身字段、新建对象的虚调用与 invokestatic）
- **THEN** 输出与本变更前逐字一致（重排仍发生，产物可编译且行为正确）

#### Scenario: Object super 正例不变

- **WHEN** 输入为 super 目标 `java/lang/Object.<init>()V` 的捕获型构造器（C1/C2 家族）
- **THEN** 输出与本变更前逐字一致（`super();` 首句、合成字段写入其后）
