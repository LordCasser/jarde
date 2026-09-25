## ADDED Requirements

### Requirement: Construction expressions keep interleaved call effects in order

普通 Java 8 构造站点 SHALL 仅在 `new` 与对应 `<init>` 之间的每个调用都属于构造器实参的实际求值链时呈现为一个 `new Type(args)` 表达式。独立调用无论是否返回栈值，都 MUST NOT 因其位于同一基本块就被移动到该表达式之前；缺少消费证明时 MUST 拒绝该构造站点、保留相关字节码来源并给出可定位的拒绝理由。呈现判定 SHALL 与证据详情选择无关。

#### Scenario: Independent void call follows allocation

- **WHEN** 一份 JVM 可验证的 class 在 `new Target; dup` 后、`Target.<init>` 前调用不返回值的 `Side.effect()`，且 `Target.<clinit>` 与该调用各有可观察效果
- **THEN** 构造站点 MUST 报告 `presented=false` 与该调用的 BCI/未满足前提，恢复文本 MUST NOT 把调用移到 `new Target(1)` 之前并宣称 Structured；拒绝产物 MUST 可追溯原 `new`、调用与 `<init>`。原 class 的运行次序 `CST` 不得被声称等价于重排后的 `SCT`

#### Scenario: Called value is a real constructor argument

- **WHEN** 构造区间中的调用结果确实沿值依赖链进入 `<init>` 的普通实参，且原有构造站点其余条件均满足
- **THEN** 该调用 SHALL 继续作为 `new Type(call())` 中的实参呈现一次，不能仅因它位于区间中就拒绝，并 SHALL 保持类初始化、调用和构造器效果顺序

#### Scenario: Evidence selection does not turn refusal into acceptance

- **WHEN** 相同独立调用站点请求必要证据、全部规则详情或 BCI 范围详情
- **THEN** 完整运行的正文与站点接受/拒绝结论 MUST 相同；详情选择只影响记录物化，不得把未证明站点报告为已呈现
