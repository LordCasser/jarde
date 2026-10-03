## ADDED Requirements

### Requirement: 构造器合成存重排仅在 super 不可能虚分派时进行

系统 SHALL 仅在构造器的 super 调用目标恰为 `java/lang/Object.<init>()V`（owner 与 descriptor 双匹配）时，把 pre-super 的合成字段直传存组移至 super 之后呈现；super 目标为任何其它类时 SHALL 保持既有逐字呈现与诊断，不得重排、不得改为整方法拒绝、不得丢弃该构造器。已重排的正例呈现 SHALL 逐字不变；系统 SHALL NOT 产出"可编译且行为不同"的构造器文本。

#### Scenario: 构造期虚分派反例不重排

- **WHEN** `AnonymousSuperDispatch$1`（`putfield val$captured` → `invokespecial Base.<init>`，Base ctor 内 `invokevirtual observe()` on `this`）经三方 Java 8 重编运行
- **THEN** 呈现保持捕获写入早于 `super()` 的原顺序，且该文本要么不可编译、要么运行保持 `visibleDuringSuper=true`——不得出现可编译且 `visibleDuringSuper=false`

#### Scenario: 任意非 Object super 不重排

- **WHEN** 输入为 super 目标非 `java/lang/Object.<init>()V` 的捕获型构造器（`anonymous-super-args/AnonymousSuperArgs$1`、`anonymous-capture/AnonymousCaptureCases$1`，及生成器构造的用户类 super 负例）
- **THEN** 呈现为捕获写入早于 `super()` 的逐字节序，带既有诊断（不可编译是如实登记状态，见 design 决策 2），且行为与原 class 一致

#### Scenario: Object super 正例仍重排

- **WHEN** 输入为 super 目标 `java/lang/Object.<init>()V` 的捕获型构造器（C1/C2 家族与 `capture_ctor_class` 生成的全部正例）
- **THEN** 输出与本变更前逐字一致（`super();` 首句、合成字段写入其后）
