## ADDED Requirements

### Requirement: Proven Java 8 string switches recover their source selector and labels

当完整的 class 事实证明一个字符串分派与 Java 8 `switch (String)` 等价时，系统 SHALL 输出单个字符串 `switch`，并保留全部分支与可观察行为；缺少证明时 MUST 保留行为忠实的已有呈现或明确拒绝，不得猜测标签。

#### Scenario: Colliding string constants select different arms

- **WHEN** 两个 `case` 字符串具有相同 `hashCode`，但匹配不同的分支
- **THEN** 生成源码 SHALL 以两个不同的字符串标签表示它们，Java 8 重编译后各自返回与原 class 相同的结果

#### Scenario: Selector is evaluated once and null still fails

- **WHEN** selector 的计算有副作用，或其值为 `null`
- **THEN** 生成源码 SHALL 只计算一次 selector，且 `null` 的异常类别与发生顺序 SHALL 与原 class 一致

#### Scenario: Default, grouped labels and fallthrough keep their targets

- **WHEN** 原分派包含 default、多个标签共享目标或合法 fallthrough
- **THEN** 恢复源码 SHALL 保留每个输入进入的目标、执行顺序与退出行为

#### Scenario: Shape is incomplete or ambiguous

- **WHEN** 字符串比较、分派索引或任一标签到分支的映射无法完整证明
- **THEN** 系统 MUST NOT 输出推测的字符串 `switch`，并 SHALL 保留完整来源与已有安全呈现；预算或取消耗尽时 SHALL 沿既有停止通道结束

#### Scenario: Source origins identify both dispatches

- **WHEN** 一个被证明的字符串 `switch` 被呈现且请求来源证据
- **THEN** 选择器、比较、分派、标签与分支的实际字节码 SHALL 在来源映射中有对应锚点，默认与完整证据选择 SHALL 生成相同正文
