## ADDED Requirements

### Requirement: Bitwise expressions preserve integral and boolean meanings

系统 SHALL 恢复已有可接受区域内、操作数证据足够的int/long位运算及非短路boolean逻辑表达式，保持结果类型、数值宽度、每侧一次求值及分组。系统 MUST NOT 把boolean与整数混合操作数伪装成合法Java表达式，或把`&`/`|`改成会跳过右侧求值的短路运算。

#### Scenario: Integral operators retain boundaries and grouping

- **WHEN** 方法以int/long或受整数提升的byte/short/char操作数计算`& ^ |`，包括负值、极值、与-1异或及嵌套算术
- **THEN** 恢复正文 SHALL 保持原操作数宽度和分组；实际重编译执行的结果与原class一致

#### Scenario: Boolean expressions survive nested and local consumption

- **WHEN** 已证明boolean的参数、字段、调用、数组元素或局部组成嵌套`& ^ |`，结果经过局部复制、受支持分支内赋值、返回、实参或条件消费
- **THEN** 已恢复表达式与局部声明 SHALL 使用一致的boolean类型，0/1仅在被证明要求boolean的表达式位置呈现为false/true，实际完整类重编译执行与原class一致

#### Scenario: Calls retain eager evaluation and abrupt completion

- **WHEN** 左右操作数是有副作用或可能抛错的调用，且左值已足以确定boolean结果
- **THEN** 正常完成的左侧之后仍 SHALL 求值右侧一次；左侧抛错时不执行右侧，右侧抛错时保留已经发生的左侧效果

#### Scenario: Mixed primitive operands remain an explicit boundary

- **WHEN** 合法JVM字节码对已证明boolean及非boolean整数值执行相同int位运算，且没有合法直接Java表达式
- **THEN** 系统 SHALL 明示引用而非输出类型错误或转换语义后的表达式，并保留被拒绝消费者及未呈现生产者来源

#### Scenario: Changed locals and rejected consumers preserve evidence

- **WHEN** 操作数保存了后来已覆盖的局部旧值，或位运算消费/生产链不能被当前恢复规则表达
- **THEN** 系统 MUST NOT 用新局部值重算或丢失调用效果；已经呈现的语句保留，其余必要生产者与消费者通过真实物理来源引用

### Requirement: Bitwise recovery retains bounded work and artifact accounting

位运算类型判断、值恢复和来源生成 SHALL 遵循既有有界工作、停止及产物提交契约，不以新增表达式绕过预算、递归限制或证据选择。

#### Scenario: Evidence selection changes metadata only

- **WHEN** 同一位运算方法分别以默认及完整来源请求恢复，预算足够
- **THEN** 正文 SHALL 逐字相同，默认请求不发布来源表；完整请求的操作数、运算和消费者保留非空BCI及真实成员来源

#### Scenario: A source budget stop preserves committed text

- **WHEN** 正文已提交而来源预算耗尽
- **THEN** 系统 SHALL 保留已提交正文并报告不完整来源，不伪造未支付来源或重复计费/发射表达式

#### Scenario: Deep type propagation stops without process failure

- **WHEN** 位运算嵌套或局部传播超过可用深度、工作预算或收到取消
- **THEN** 系统 SHALL 返回可解释的停止或保守引用结果，不发生栈溢出、无界传播或将未完成证明误称为boolean
