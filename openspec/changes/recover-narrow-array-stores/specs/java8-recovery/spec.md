## ADDED Requirements

### Requirement: Narrow integer array stores preserve instruction conversion

对于数组元素类型和整数操作数均有充分证据、其余结构已可呈现的 byte/char/short 数组写入，系统 SHALL 保留实际数组写入指令的窄化，不因源码表达式类型为 int 而拒绝这一已知转换。

#### Scenario: Arbitrary int values are stored in narrow integer arrays

- **WHEN** 合法 class 将负值、极值或边界外 int 值写入已证明的 byte[]、char[] 或 short[]
- **THEN** 恢复的完整正文 SHALL 原样重编译并产生与原 class 相同的数组元素值

#### Scenario: A narrow source local is presented as int

- **WHEN** 普通 javac 窄类型局部回读在恢复中呈现为 int，并被写入对应窄整数数组
- **THEN** 系统 SHALL 在写入位置保留窄化，无需假称恢复了原局部声明类型；现有合法常量写入也 SHALL 保持语义

#### Scenario: A producer fails before a null or out of bounds store

- **WHEN** 写入值的生产者可能抛错，且数组为null或下标越界
- **THEN** 正文 SHALL 保留实际操作数求值顺序、调用次数、异常身份和未完成写入时的原数组值，不能将数组检查移到生产者之前

#### Scenario: Integer storage does not infer boolean meaning

- **WHEN** bastore的数组类型或操作数类型不足以支持本项窄整数转换，或输入需要尚未支持的一般整数到boolean最低位转换
- **THEN** 系统 SHALL 明确保留拒绝与真实来源，不能将其猜成byte数组、非零判断或未经授权的普通赋值

### Requirement: Narrow array stores retain bounded and stable evidence

窄数组写入恢复 SHALL 遵守既有预算、取消和证据稳定性契约。

#### Scenario: Evidence requests share one write expression

- **WHEN** 同一窄数组写入请求默认与完整证据或重放呈现
- **THEN** 正文 SHALL 一致，完整来源同时标识实际写入指令以及数组、下标、值的实际来源，不伪造额外字节码位置

#### Scenario: Insufficient budget stops conversion work

- **WHEN** 转换、节点或来源构造时预算不足或收到取消
- **THEN** 系统 SHALL 沿既有停止通道终止，不提交未支付或丢失操作数来源的正常恢复正文
