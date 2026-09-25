## ADDED Requirements

### Requirement: Wrapped array element binding preserves evaluation order

当数组计数循环已被证明与 Java 8 增强 `for` 等价，且当前元素读取位于循环体表达式的子树时，系统 SHALL 仅在把该读取移至每轮隐式元素绑定不会跨越任何可观察操作、异常处理边界或另一次元素读取时输出增强 `for`。系统 MUST 保持元素值、数组更新、调用及异常的原有顺序；证据不足时 SHALL 保留原有可执行计数循环或可定位的保守引用。

#### Scenario: Array element precedes a call

- **WHEN** 每轮首条体语句中唯一的 `array[i]` 读取先于同一表达式中的调用，索引仅用于循环测试、递增和该读取，数组和长度身份已证明
- **THEN** 系统 SHALL 将该元素绑定为增强 `for` 的变量，并在循环体保留读取之后的调用；完整类 Java 8 重编执行 SHALL 与原 class 的结果和调用轨迹相同

#### Scenario: Call or array mutation precedes element read

- **WHEN** 每轮在 `array[i]` 读取之前执行调用、写入或其它可观察操作，包括调用实参从左到右求值和前置循环体语句
- **THEN** 系统 MUST NOT 将该读取提前至增强 `for` 的隐式元素绑定；在调用修改该数组时，输出 SHALL 保留原 class 所读到的更新后元素

#### Scenario: Two reads of one element around mutation

- **WHEN** 同一轮对 `array[i]` 有两次读取且两次之间可能修改数组
- **THEN** 系统 MUST NOT 将两次读取合并为一个增强 `for` 绑定；输出 SHALL 保留两次各自的读取时点

#### Scenario: Null and throwing paths

- **WHEN** 容器为 null，或元素读取之后的调用抛出异常
- **THEN** 任何增强 `for` 投影 SHALL 保持此前已发生的副作用、异常类型和传播路径；不能证明时 SHALL 保留原普通循环

#### Scenario: Source and bounded refusal

- **WHEN** 表达式子树的来源、处理器边界、唯一消费或预算不能被完整确认
- **THEN** 系统 MUST NOT 部分隐藏旧读取或发布无来源的增强 `for`，且 SHALL 按既有停止／保守输出契约保留真实字节码位置
