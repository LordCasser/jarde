## ADDED Requirements

### Requirement: A named catch row whose clause rethrows its parameter is a user catch

异常表唯一具名类型行（`catch_type != 0`）到达某 handler，且该 handler 体以重抛入口存储的同一局部结尾时，恢复结果 SHALL 把该行呈现为普通用户 `catch` 子句，子句体末尾是重抛该子句参数的 `throw` 语句，且该子句之前的语句（如调用）按既有规则呈现。finally 拷贝候选检测 MUST 仅考虑 `catch_type == 0` 的行；任何具名类型行 MUST NOT 因 finally 拷贝候选被拒绝。呈现 MUST 保留 handler 入口存储与重抛 `athrow` 的物理来源。

#### Scenario: Precise rethrow with a logging call

- **WHEN** 方法在 try 内按条件抛出两类已检查异常，`catch (Exception e)` 记录后 `throw e;`，`throws` 声明两类窄类型
- **THEN** 恢复文本 SHALL 呈现 `try`/`catch (java.lang.Exception <参数名>)`，子句体含记录调用与 `throw <参数名>;`，不再整方法字节码引用；`throws` 拼写保持 `Exceptions` 属性的两类窄类型

#### Scenario: The synthetic finally copy is untouched

- **WHEN** 某行 `catch_type == 0` 且 handler 形如参数存储、体、参数读取、`athrow`
- **THEN** 该行 MUST 按既有 finally/TWR 规则处理，MUST NOT 被改写成用户 `catch`

#### Scenario: A handler reached by more than one named row uses existing catch recovery

- **WHEN** 同一 handler 入口被多条异常表行到达（多捕获编码），其中含具名行
- **THEN** 这些具名行 MUST NOT 被误判为 finally 拷贝；恢复 SHALL 沿既有多捕获路径逐项证明或引用，不新增并集类型推断，也不得将部分可呈现的语句虚报为整个子句已完整恢复

#### Scenario: Runtime comparison of the two throw modes and normal completion

- **WHEN** 受控执行对照分别触发两类异常与正常完成路径，原 class 与重编译后的恢复文本同输入运行
- **THEN** 异常类型、异常消息、正常路径返回值 SHALL 一致；该证据 SHALL 只陈述受控样例，不泛化为全面语义等价
