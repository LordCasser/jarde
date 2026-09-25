## ADDED Requirements

### Requirement: Narrow field stores preserve JVM instruction conversion

对于字段描述符、已呈现整数值与实际字段写入事实均已证明的 byte/char/short 存储，系统 SHALL 在该写入位置表达 JVM 截断并恢复字段变化，而不改变一般 Java 赋值/调用转换规则。

#### Scenario: Instance and static narrow field writes

- **WHEN** 合法 class 以 `putfield` 或 `putstatic` 将边界外或负的 int 写入 B/C/S 字段
- **THEN** 完整恢复源码 SHALL 可重编译，并与原 class 的字段值逐项相同

#### Scenario: Ordinary typed writes remain stable

- **WHEN** 普通 javac 用已证明的同型局部、合法零/一常量写入 B/C/S 字段
- **THEN** 系统 SHALL 保持每笔写入的原字段值，不凭局部 frame 的 int 外观引入多余或非法转换

#### Scenario: Producer precedes null receiver failure

- **WHEN** 实例字段值生产者可能抛错且 receiver 可能为 null
- **THEN** 恢复正文 SHALL 保留生产者调用次数、异常类别/身份及字段未被写入的状态；producer 抛错优先于该写入的 NPE，producer 正常后仍按原指令触发 NPE

#### Scenario: Boolean storage is not guessed from integer values

- **WHEN** Z 字段写入值没有现有 boolean 证明而要求 JVM 低位转换
- **THEN** 在本次 B/C/S 变更的验收范围内系统 SHALL 保持来源完整的原拒绝，不能改作非零判断、B/C/S 数值 cast 或无证据的合法 Java 赋值；后续独立 `recover-boolean-field-stores` 以最低位语义处理该边界

### Requirement: Field write recovery keeps bounded evidence

窄字段写入 SHALL 沿现有来源、重放与停止合同产生 AST 和拒绝证据。

#### Scenario: Successful store retains actual origins

- **WHEN** 默认与完整证据模式呈现同一 B/C/S 写入
- **THEN** 正文 SHALL 相同，字段写入、接收者和值生产者的实际 BCI SHALL 可追溯，且不得重复生产者

#### Scenario: Refused store retains producer origins

- **WHEN** 其他必要类型或表达式仍无法呈现
- **THEN** 拒绝来源 SHALL 含真实 put 指令及必要生产者，不以单个 put BCI 遮掉已观察的调用；预算或取消不足时 SHALL 沿既有停止通道结束
