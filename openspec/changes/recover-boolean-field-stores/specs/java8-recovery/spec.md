## ADDED Requirements

### Requirement: Proven boolean field stores preserve integer low-bit semantics

对于已验证的 `Z` 字段写入，系统 SHALL 在消费位置把已呈现的整数值转换为其最低位所代表的 Java boolean，保持实际字段值、生产者次数和异常顺序；此授权 MUST NOT 扩展到普通 boolean 赋值、参数、返回或数组元素写入。

#### Scenario: Odd, even and extreme integer inputs

- **WHEN** 合法 class 以 `putfield` 或 `putstatic` 把正负奇偶数、`Integer.MIN_VALUE` 和 `Integer.MAX_VALUE` 写入 Z 字段
- **THEN** 完整恢复源码 SHALL 可编译执行，并逐项产生与 JVM 原 class 相同的 boolean 字段值，不把非零偶数写成 true

#### Scenario: Value producer and null receiver

- **WHEN** receiver 为 null 且值生产者正常返回或先抛错
- **THEN** 恢复源码 SHALL 恰调用生产者一次；producer 抛错优先于 put 的 NPE，producer 正常后发生 NPE，字段旧值不被修改

#### Scenario: Existing proven boolean stores

- **WHEN** 字段写入值来自 boolean 参数/调用/字段、已证明的局部或 0/1 字面值
- **THEN** 恢复 SHALL 保留合法 boolean 拼写与相同运行值，不加入无必要的整数余数或猜测式转换

### Requirement: Boolean field narrowing is source-bound and conservative

系统 SHALL 只依据真实 Z 字段写入及已呈现整数值构造布尔结果，沿已有来源、重放与预算合同呈现；任何缺失字段或值证据 SHALL 来源完整地拒绝。

#### Scenario: Wrong value type or unproved accessor

- **WHEN** 字段描述符非 Z、值不能呈现为整数，或目标 accessor 未经现有规则验证
- **THEN** 本要求 MUST NOT 把该值放入 boolean 字段写入，也不得放宽一般 `meeting_position`

#### Scenario: Exact origins and output modes

- **WHEN** 同一真实 Z 写入分别请求默认与完整来源
- **THEN** 正文 SHALL 相同，直接写入的真实 put BCI 与值生产者 BCI SHALL 可追溯；已验证 accessor SHALL 保留调用处及被调用方法内 put 的来源；预算/取消不足时 SHALL 沿既有停止合同返回，不重复或丢弃生产者
