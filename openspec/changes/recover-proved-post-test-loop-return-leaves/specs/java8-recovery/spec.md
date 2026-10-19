## ADDED Requirements

### Requirement: Proved post-test loop return leaves preserve evaluation order

Java恢复 SHALL 支持普通后测循环中已证明独占的直接返回分支，保留每次体首取值、转换与赋值、返回分支及体尾测试的求值顺序；提前返回 MUST NOT 执行后续体尾条件或消耗额外输入。

#### Scenario: Original iterator return loop is executable

- **WHEN** 对完整迭代器原件的debug/no-debug class与完整JAR请求恢复，循环先读取并保存元素，空元素直接返回，否则测试是否继续，退出后返回最后元素，且所有路径获证
- **THEN** 目标方法 SHALL 输出结构化源码且完整生成类可重编；原件两组输入的返回、迭代调用次序、剩余状态、stdout与实际check结果 SHALL 按相同runtime一致，不要求等价循环采用同一种拼写

#### Scenario: Early return and throwing operations keep their boundaries

- **WHEN** 输入首元素为空且后方仍有元素、迭代器为空或缺失，或取值/后测操作实际抛出异常
- **THEN** 恢复源码 SHALL 保持原件的消费次数、短路顺序和异常类型，空值返回后 MUST NOT 调用后测条件，取值失败后 MUST NOT 继续消费或伪造返回值

#### Scenario: Unproved terminal ownership remains refused

- **WHEN** 返回路径有额外入口、跨越当前作用域、未闭合的非正常转移、非返回终结或与普通出口混淆
- **THEN** 本能力 MUST NOT 把该路径伪造为安全的提前返回或break，输出 SHALL 保留既有有界降级与可定位诊断

#### Scenario: Evidence and interrupted proof stay reliable

- **WHEN** 同一输入在default/all证据选择下恢复，或候选证明/组装期间发生预算停止或取消
- **THEN** 证据选择 MUST NOT 改变正文、身份、质量或有效物理来源；停止 SHALL 保留真实原因与用量，MUST NOT 发布半份循环、重复owner或通过退还用量掩盖停止
