## ADDED Requirements

### Requirement: Proven boolean array stores preserve the JVM low bit

对已证明元素为 boolean 的数组，系统 SHALL 在真实 `bastore` 消费可呈现整数值时生成可重编译 Java，并逐项保持原 JVM 写入后读回的最低位结果。该转换 MUST 只属于该数组元素写入位置，不改变整数值在其他消费位置的类型。

#### Scenario: Odd, even, negative and extreme values

- **WHEN** 合法 Java 8 class 把 0、1、2、3、-1、-2、`Integer.MIN_VALUE` 和 `Integer.MAX_VALUE` 经真实 `bastore` 写入已证明的 `boolean[]` 并读回
- **THEN** 完整恢复类 SHALL 通过 Java 8 编译和严格 JVM 验证，逐项与原 class 相同；2 MUST 读回 false，3 MUST 读回 true

#### Scenario: Array, index and value are effectful

- **WHEN** 数组表达式、下标表达式和值表达式各有可观察副作用，写入成功或随后发生 null、越界、值生产者异常
- **THEN** 恢复类 SHALL 各执行三者一次并保持原数组→下标→值的顺序、实际异常类型和写入状态；值生产者先抛错时不得提前执行数组存储检查

#### Scenario: Existing boolean evidence

- **WHEN** 写入值已由方法、字段、局部、数组读取或 0/1 常量证明为 boolean
- **THEN** 系统 SHALL 保留等价 boolean 写入，不加入不必要的整数转换，也不改变数组读取返回

### Requirement: Boolean array store conversion is proof-bound and source-bound

系统 SHALL 只以数组组件事实、实际写入指令和值的呈现类型决定转换，并沿现有来源、证据模式和停止契约发布结果。不能因缺少任一证明而猜测 boolean 数组。

#### Scenario: Ambiguous or incompatible store

- **WHEN** 数组组件未知或并非 boolean、opcode 与已证明组件不匹配，或值不能呈现为 B/C/S/I 与既有 boolean 证据之一
- **THEN** 本项 MUST NOT 生成整数到 boolean 的数组写入；未知或不相容情况 SHALL 保持可定位拒绝，已证明 `byte[]` 的 `bastore` SHALL 继续按 byte 语义处理

#### Scenario: Store and operand origins

- **WHEN** 同一已恢复的写入请求默认证据、完整证据和重放
- **THEN** 正文 SHALL 相同，完整来源 SHALL 同时保留数组、下标、值生产者和真实 `bastore` 的 BCI；合成转换不得冒充 class 中存在额外字节码

#### Scenario: Bounded stop

- **WHEN** 输出、分析或来源预算不足，或请求被取消
- **THEN** 操作 SHALL 按既有停止契约终止，不发布部分 Java 表达式、半张来源表或重复的生产者
