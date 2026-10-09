## ADDED Requirements

### Requirement: 数组初始化器元素 SHALL 按本次可证明的赋值兼容关系恢复

当本次分析已经证明 fresh reference-array 分配、准确的分量类型、闭合且按物理求值次序的元素存储，并能由现有平台、数组形状或本次运行环境选中的 class-file header 证明元素呈现类型到分量类型的单向赋值兼容关系时，系统 SHALL 接受初始化器。系统 SHALL 保留分量类型、各元素表达式和求值次序，不插入改变数组兼容或异常语义的强制转换。

兼容范围 SHALL 包括已知平台类/接口、自有快照子类/接口及由这些已知引用关系证明的等秩数组提升；同型、null、Object 与 primitive 的已有处理 SHALL 保持原有行为。系统 SHALL NOT 引入 LUB 推断或通过其他位置的证据猜测当前元素的关系。

缺少兼容证明的合法输入 SHALL 明确保留未覆盖/拒绝状态。降向、无关类型、primitive 或数组秩不匹配且不能证明兼容的输入 SHALL NOT 因此项被无条件接受。

#### Scenario: 异构装箱与平台接口初始化器

- **WHEN** fresh Number[] 的元素为 Integer 和 Long，或 fresh 接口/父类数组的元素关系由本次可用平台事实证明
- **THEN** 系统接受相应元素，完整生成源码能够隔离编译，验证运行的退出码、stdout 和 stderr 与原程序一致

#### Scenario: 自有类和接口的准确存储位置证明

- **WHEN** 同一分析环境选中的源 class-file header 的有界继承/接口链证明 source 可赋值给 fresh 数组分量，且该证明对应实际元素存储位置及完整 source/component 类型
- **THEN** 系统接受该元素；直接被可信 header 声明的目标不必有物理目标 class，但缺失的中间 header 不得被外部猜测补齐

#### Scenario: 等秩引用数组元素提升

- **WHEN** fresh 数组分量与元素都是等秩引用数组且最深引用分量关系由本次平台或快照事实证明
- **THEN** 系统保留真实数组类型与元素次序恢复；primitive 不变性和不能证明的 rank 关系继续拒绝

#### Scenario: 缺失事实和错位证据保持拒绝

- **WHEN** 当前 source/component 关系无证明、层级读取不完整或预算耗尽，或只有其他调用/存储位置的证明
- **THEN** 系统不借用错位证据恢复该初始化器，并保留明确拒绝；不得将合法但未证明的 BigDecimal 到 Number 关系声称为非法 Java

#### Scenario: 效果次序及非 fresh 数组边界

- **WHEN** store 乱序、重复、数组逃逸或更新的是既有可能更窄的数组，未满足既有初始化器证明
- **THEN** 本项不得绕过该证明折叠初始化器或重排副作用

#### Scenario: 完整语义验收

- **WHEN** CT.cov 或家族 fixture 的生成源码能编译且 JVM 退出码为零，但方法被拒绝或 main 输出与原程序不同
- **THEN** 该案例 SHALL 被记为未恢复；系统验收不得删除成员、借用原 class 或只比较退出码
