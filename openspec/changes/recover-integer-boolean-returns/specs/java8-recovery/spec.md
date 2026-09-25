## ADDED Requirements

### Requirement: Integer operands returned as boolean preserve the JVM low bit

对已可完整呈现的整数操作数，系统 SHALL 在实际 `ireturn` 与方法 `Z` 返回描述符共同证明转换时，生成可重编译且与原 JVM 的最低位结果一致的 Java。该规则 SHALL 只作用于返回消费位置，不改变操作数在普通赋值、调用、字段或数组位置的类型。

#### Scenario: Direct integer return

- **WHEN** 合法 Java 8 class 的 `(I)Z` 方法以 `ireturn` 返回已呈现的 int 参数，输入包含 0、1、2、3、-1、-2 和整数极值
- **THEN** 生成的完整类 SHALL 编译、验证执行并逐项返回与原 class 相同的 boolean；输入 2 MUST 为 false，3 MUST 为 true

#### Scenario: A single effectful operand

- **WHEN** 返回值由一次可呈现的调用或字段更新产生，且最终由实际 `ireturn Z` 消费
- **THEN** 输出 SHALL 保留该操作数一次求值、写后字段完整整数值及其异常/同步顺序，仅将返回结果转换为 boolean

#### Scenario: Structured return destinations

- **WHEN** 已恢复的同步返回或 switch 臂下推把一个已呈现整数值交给共同的 `ireturn Z`
- **THEN** 每条可证明的返回路径 SHALL 应用同一最低位语义，保留原分支、监视器和表达式求值位置，不把臂值的来源冒充共同返回指令

#### Scenario: Proven boolean and raw numeric boundary remain distinct

- **WHEN** 返回值已由签名或局部事实证明为 boolean，或为可直接拼写的 0/1 常量
- **THEN** 系统 SHALL 保留等价的 boolean 返回；它 MUST NOT 因整数返回规则而把 `(Z)B/C/S` 非规范原始载荷改写成 0/1

#### Scenario: Missing integer evidence is refused

- **WHEN** 返回操作数不能完整呈现为 B/C/S/I，或实际返回指令与本项准入不符
- **THEN** 系统 MUST 保留可定位拒绝及相关生产者/返回来源，不得凭 `Z` 描述符推断任意值是 boolean

### Requirement: Boolean return adaptation keeps bounded and repeatable evidence

整数到 boolean 的返回转换 SHALL 沿用现有有界 AST、来源、停止及证据选择契约；不能因证据详略改变正文或在预算不足时发布半个转换。

#### Scenario: Operand and return have separate origins

- **WHEN** 对同一已恢复返回请求默认、完整证据和 replay
- **THEN** 正文 SHALL 相同，完整来源 SHALL 同时指出原值生产位置与实际 `ireturn`，包括共享返回与字段更新形态

#### Scenario: Budget or cancellation interrupts conversion

- **WHEN** 输出、分析或来源物化预算不足，或请求在转换期间取消
- **THEN** 操作 SHALL 按现有停止契约终止，不发布部分 Java 表达式、丢失可观察生产者或改变再次请求的转换结果
