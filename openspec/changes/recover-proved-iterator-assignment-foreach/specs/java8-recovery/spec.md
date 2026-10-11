## ADDED Requirements

### Requirement: Proved iterable loop preserves an existing element assignment

系统 SHALL 在完整 iterator 循环与唯一元素消费证据成立时，支持循环首句对已有局部变量的 inline 强转赋值并呈现增强 for。系统 MUST 保留原赋值的目标、声明作用域、强转、求值次数与顺序、异常路径和物理来源；不得把该变量改为增强 for 的局部绑定。

#### Scenario: Element variable is declared outside the loop

- **WHEN** 可靠循环体的首句是对已有局部的 `target = (T) iterator.next()`，完整 iterator 和 next→cast→store 证据成立
- **THEN** 系统 SHALL 生成 Object 增强 for 元素并在原首句执行相同 cast 和赋值，空循环时变量原值不变，最后一次赋值在循环外仍可观察

#### Scenario: Binding shape or consumption is not proved

- **WHEN** 首句不是所支持的 inline 强转局部赋值，或 next/cast 存在额外消费者，或 iterator 身份、初始化位置、间隔或异常范围证据不足
- **THEN** 系统 MUST 保留可靠原表示，不删除初始化或局部声明、不发布部分投影

#### Scenario: Iterator and cast failures preserve state

- **WHEN** 空列表、null iterable、错误元素类型或 iterator/hasNext/next 抛异常
- **THEN** 完整生成类的返回值、既有局部状态、副作用、调用序列及异常 SHALL 与原 class 一致，cast 成功前不得提前写入目标

#### Scenario: Evidence selection and stopping preserve the decision

- **WHEN** 相同输入通过 default/all/no-rule-details 请求，或处理遭遇真实预算停止或预取消
- **THEN** 完成结果的正文与物理来源 SHALL 一致；停止 MUST 报告真实计费维度、用量和位置且不发布当前方法的部分正文、来源或投影
