## MODIFIED Requirements

### Requirement: Separate representation and validation statuses

每个 source result SHALL 独立返回 representation（Java/Bytecode/Mixed）、quality（Structured/Conservative/Fallback）、syntax_status（Checked/Unchecked/NotJava）、compile_status（NotAttempted/Compiles/Failed）、semantic_validation（LocalInvariants/FixtureDifferential/Unproven）、verification（Performed/NotPerformed/Failed）、recovery profile、coverage 和 diagnostics；`Mixed` 只表示 representation，禁止把它当作 quality。未实际编译时 compile_status MUST 为 NotAttempted；只通过局部不变量可记录 LocalInvariants，受控语料对照通过才记录 FixtureDifferential，无相应证据时记录 Unproven。生成 Java 文本不得自动意味着可重编译或语义等价。验收 SHALL 独立核对已知反例的返回值/可观察行为与 fallback 的生产者来源完整性；语义已知错误的输出不得因 Unproven 或既有门禁绿色被计为完成。

每个平面 SHALL 只被读作它自己的结构性问题：representation 说明表示形态，quality 说明区域结构的强度，syntax/compile/semantic/verification 各自说明对应证据是否存在，coverage 与 execution 说明工作范围与完成度，content 说明交付物是否含发射语句。任意组合——包括 quality=Structured、content=contains_statements、execution=complete、coverage=CompleteWithinSchema 且无诊断同时成立——MUST NOT 构成语义等价或值正确的声明；已知反例中存在这样同时成立、却计算了与字节码不同值的产物。需要语义证据的调用方 MUST 执行受控编译/执行对照，或把产物当作假设并保留其未证明状态；MUST NOT 用任一平面组合代替该动作。

#### Scenario: Structured output cannot compile

- **WHEN** 恢复结果可读但包含 Java 无法表达的名称或控制结构
- **THEN** 结果保留 representation=Java 或 Mixed、按结构化程度设置 quality=Structured/Conservative/Fallback，并设置 syntax_status=NotJava、compile_status=NotAttempted、semantic_validation=Unproven、verification=NotPerformed；只有实际执行编译且失败时才使用 compile_status=Failed，不宣称 Java 8 可重编译

#### Scenario: Complete scan with fallback quality

- **WHEN** 完整扫描成功但某些区域只能保留低级结构
- **THEN** 结果可为 representation=Mixed、quality=Fallback，同时 coverage=CompleteWithinSchema；quality 不得改写为 Partial

#### Scenario: Controlled fixture equivalence

- **WHEN** 受支持 fixture 在隔离环境通过重编译和输入输出/异常对照
- **THEN** 仅该 fixture/profile 记入已验证覆盖，不能把单个样本推广成所有合法 JVM 方法

#### Scenario: Determinism excludes only observed elapsed time

- **WHEN** 同一输入、profile 和 limits 的受控恢复重复完成，且未触发 elapsed 截止或外部取消
- **THEN** 确定性比较 SHALL 仅剔除观测的 elapsed_millis，保留其余结果、origin、diagnostics、rules、顺序和预算计数字段；0/1 ms 时钟抖动不能成为测试假红，也不能通过缩减比较字段掩盖真实差异

#### Scenario: Existing corpus passes but an independent counterexample fails

- **WHEN** 原有测试与 CI 全部通过，而独立编译输入证明生成结果值或可观察行为不同
- **THEN** 当前正确性验收 MUST 保持未完成，将反例纳入可重放回归；修正前不得把归档、Structured 或旧样本通过推广为该反例通过

#### Scenario: Fallback coverage names the observable producer

- **WHEN** 生成结果降级且原方法包含未呈现的可观察生产者
- **THEN** 验收 SHALL 核对该生产者确实出现在产物的可靠语句或低级引用中，并有对应物理 origin；仅证明原指令属于 CFG block，或字段识别记录中存在该 BCI，不能替代此项检查

#### Scenario: Structured is a structural claim

- **WHEN** 一份产物同时报告 quality=Structured、content=contains_statements、execution=complete 且没有诊断
- **THEN** 该组合 MUST NOT 被当作值正确或语义等价的证据；验收只能依赖受控执行对照或对相关操作数值/求值点的证明，已知反例表明这样的产物可以计算与字节码不同的值

#### Scenario: The caller's action instead of a plane combination

- **WHEN** 调用方需要语义等价或值正确的证据
- **THEN** MUST 执行受控编译/执行对照（仓库既有 P3 3.3 流程），或把产物记录为假设（semantic_validation=Unproven、verification=NotPerformed，并保留其结构化平面）；MUST NOT 以任一平面组合、诊断为空或样本曾经通过代替该动作
