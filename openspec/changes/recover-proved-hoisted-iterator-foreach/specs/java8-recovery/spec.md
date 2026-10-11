## ADDED Requirements

### Requirement: Proved iterable loop with a visible elevated declaration

系统 SHALL 在iterator初始化、循环测试、next取值与元素绑定的完整证据成立，且其局部声明有已证明可见的祖先词法归属时恢复增强for；声明与初始化处于不同语句列表本身不得成为拒绝理由。系统 MUST 保持每次初始化、hasNext、next和cast的求值次数、顺序与异常路径，并保留物理来源。没有完整变量身份、声明可见性或消费/effect证据时 SHALL 保留原可靠表示和具体拒绝原因。

#### Scenario: Inner iterator declaration belongs to an ancestor

- **WHEN** 嵌套循环的内层iterator声明已提升到可见祖先，初始化在每次进入内层循环的位置，完整循环和唯一消费证据成立
- **THEN** 系统 SHALL 呈现内层增强for，初始化仍每次进入执行，原元素cast和绑定语义保持；祖先空声明可以保留，外层语句和循环行为不变

#### Scenario: Slot identity or declaration visibility is incomplete

- **WHEN** 初始化/测试/next对应不同局部身份、缺可见owner、或声明归属不完整
- **THEN** 系统 MUST 拒绝该增强for投影，保留可靠原形和相关物理位置，不以同名或同slot推定一致

#### Scenario: Replaced Phi operands retain exact use ownership

- **WHEN** iterator除已证明的test/next load外，仅有已替换Phi保留的None-BCI输入记录
- **THEN** 系统 SHALL 按block核精确ValueId operand次数与use occurrence multiset相等，且每个匹配Phi输出和Value arm沿有界replacement链解析到同一iterator；未替换/仍有reader/mixed arm/计数不符 MUST 拒绝，不忽略所有None-use，不约束完全不含该值的其它Phi

#### Scenario: Executable statements separate the hoisted initialization and loop

- **WHEN** 新祖先声明路径的iterator初始化与循环之间有局部赋值或其它执行语句，即使RHS为literal
- **THEN** 系统 MUST 拒绝该路径的foreach投影；仅空声明可以留在间隔，不能使初始化异常观察到重排后的局部状态

#### Scenario: Observable iteration and cast failures

- **WHEN** iterator初始化、hasNext、next或元素cast可观察或抛异常，控制输入覆盖空列表、null及错误元素类型
- **THEN** 生成完整类的调用序列、次数、异常及先前副作用 SHALL 与原class一致；额外消费、读写或不同异常边未证实时 MUST 保留可靠原形

#### Scenario: Outer assignment and generic projection remain separate

- **WHEN** 外层元素取值仅为赋值而未满足本片元素绑定前提，或普通泛型源码证明不足
- **THEN** 系统 SHALL 保留其已有拒绝边界，不因内层声明证明成功而发布外层增强for或未证明的参数化类型

#### Scenario: Selection and stopping preserve the same decision

- **WHEN** default/all选择、可选规则详情或真实预算停止处理该输入
- **THEN** 已完成正文和物理来源 SHALL 在相同输入上保持一致；停止 MUST 记录真实dimension/usage/location且不发布当前方法的部分改写，不能因可选说明改变投影判定
