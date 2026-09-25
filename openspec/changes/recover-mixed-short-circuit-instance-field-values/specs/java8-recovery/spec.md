## ADDED Requirements

### Requirement: 已证明的混合短路值可以赋给一个实例布尔字段

只有在证明图的 Region 所有权无重叠、true/false producer 与 Phi 唯一值使用成立，并且接收者/值操作数绑定到字段计划所声明的相同 SSA 值和精确字段身份后，闭合的 Java 8 混合 `&&`/`||` 值图 MAY 被发射为一个 `putfield Z` 赋值。接收者 SHALL 被证明在该字段赋值处只有一个表达式所有者，并 SHALL 在惰性 RHS 之前且只发射一次。生成的赋值 SHALL 保留 JVM 行为：RHS 的路径和效果先执行，接收者为 null 时随后才由 `putfield` 抛错。所有权、字段、接收者、使用关系或顺序任一证明失败时，整个候选 MUST 连同真实拒绝原因保留为引用。

#### Scenario: 带副作用的接收者赋入短路字段值
- **WHEN** `one(ZZ)V` 执行赋值 `target(nullReceiver).result = (a && b()) || c()`，BCI 25 的 `putfield result:Z` 消费栈深度 0 的接收者和栈深度 1 的 Phi，且接收者恰有一个所有者
- **THEN** 成功呈现结果 SHALL 可按 Java 8 编译，并保留 Runner 的全部 16 行：接收者非 null 时，字段值及 b/c 调用次数与原始类相同；接收者为 null 时，NPE 前也先发生相同的 b/c 调用次数，`receiverCalls` 保持为 1，且字段不写入

#### Scenario: 当前 Region 所有权重叠尚未解决
- **WHEN** 稳定版 Jarde 证据报告的规范 BCI 20 重叠尚未被证明可以安全解决
- **THEN** 候选 MUST 保持原子回退；实现 MUST NOT 仅为接纳实例字段消费者而绕过所有权核算

#### Scenario: 接收者或字段身份无法证明
- **WHEN** 字段不是精确认领的 owner/name/`Z` descriptor，接收者/值操作数与栈深度 0/1 不匹配，接收者 SSA 还有其它使用/所有者，接收者类型无法满足字段计划，或接收者表达式无法只呈现一次
- **THEN** 赋值 MUST 保持引用；不得移动或重复接收者调用，也不得将 RHS 分支单独发射

#### Scenario: Phi 有其它值消费者或写入包含额外结构
- **WHEN** Phi 还有另一个直接值使用，图存在额外的正常/异常所有者，或字节码是复合更新而非单条普通 `putfield`
- **THEN** 完整候选 MUST 被拒绝；`putfield` 的双操作数读取集合 MUST NOT 被误认为 Phi 有两个值使用
