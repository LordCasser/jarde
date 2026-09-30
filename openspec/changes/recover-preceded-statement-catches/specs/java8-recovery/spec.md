## ADDED Requirements

### Requirement: 非 store 语句前置的具名 catch 按普通 try/catch 恢复

系统 SHALL 在具名异常行的保护范围之前、同一块内的最后一条指令不是 `Store`（如 void 调用）时，将该行交由普通 `catch` 呈现而不是资源头检查；`try` 前的全部语句、受保护正文与 handler 体 SHALL 完整恢复。前置指令是 `Store` 的行为 SHALL 保持不变：可证明的 try-with-resources 仍恢复为 `try (…)`，证明不了的继续以资源规则的理由拒绝，永不伪装成用户 `catch`。

#### Scenario: void 调用后接具名 catch

- **WHEN** 方法形如 `helper(); try { … } catch (E e) { … }` 且三方（原 class、固定 JADX Java-input、Jarde）源码以 Java 8 重编
- **THEN** 全部语句与唯一 `try/catch` 恢复，正常与异常路径 `java -Xverify:all` 运行结果与原 class 一致

#### Scenario: store 前置的降级拒绝保持

- **WHEN** 具名行之前是 `r = open();` 一类的 `Store`
- **THEN** 该行仍按资源规则检查，证明失败时维持既有拒绝码与语义，不得改判为用户 `catch`

### Requirement: 拼接链 split 拒绝归属链自身的值流消费点

`jre_concat_split` SHALL 仅在候选链自己的 builder 值（allocation/dup 起、经 append 返回值）被另一块中的同 owner `toString` 消费时出现，并 SHALL 命名该消费点 BCI。方法内存在更早的同 owner `toString` SHALL 不影响归属；链的准入门保持由同块 walk 决定。

#### Scenario: handler 块内的链不误配早期 toString

- **WHEN** 方法含多段拼接链且某链与其自身 `toString` 同处一个晚迭代块（如 `FinallyOnce.main` 第三条链）
- **THEN** 该链按普通拼接呈现，不产生 `jre_concat_split`

#### Scenario: 真分支切断的链仍拒绝且归属正确

- **WHEN** builder 跨块使用、其 `toString` 位于另一块（如手写跨块 `StringBuilder`）
- **THEN** 维持 `jre_concat_split` 拒绝并指向该 `toString` 的真实 BCI，链不呈现为单一表达式
