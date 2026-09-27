## ADDED Requirements

### Requirement: Proven current-instance aliases may be presented directly

当 Java 8 实例方法中的非参数局部只有一次从当前实例接收者写入，且同次局部身份、SSA 来源、区域与全部读取均证明该局部始终是准确的 `this` 时，恢复 SHALL 能省略该局部声明及赋值，并在对应读取处直接呈现 `this`。输出 MUST 保留原有调用、字段写入、分支及异常行为；不得凭一个局部的名称或某一次赋值就消除其它来源。

#### Scenario: A single current-instance alias has closed uses

- **WHEN** 单一局部从 `this` 得到唯一值，并只作为已证明的调用接收者、字段接收者或 `Objects.isNull` 实参被读取
- **THEN** 恢复 SHALL 在全部这些读取处写当前实例且不写冗余局部，完整类 SHALL 以 Java 8 重编并在验证运行中与原 class 的字段状态和输出一致

#### Scenario: A local can hold a different object

- **WHEN** 同一变量还有新对象或其它来源写入、复赋值、合流值或身份不完整的读取
- **THEN** 恢复 MUST 保留原局部的源级值流或按原有规则拒绝，MUST NOT 把它的调用接收者或返回值无条件改成 `this`

#### Scenario: Proof or publication stops

- **WHEN** 用途普查、区域/绑定检查、预算或取消导致同次证明不完整
- **THEN** 恢复 MUST NOT 发布只省略部分声明/读取的源码，且 SHALL 保留可定位的物理事实和现有停止/拒绝状态
