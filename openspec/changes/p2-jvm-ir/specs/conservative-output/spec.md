## Purpose

交付可复核的方法 Bytecode 分析，分别说明质量、已完成阶段、覆盖、执行状态和验证证据。

## ADDED Requirements

### Requirement: Honest fallback output

方法结果 SHALL 分别返回 representation、quality、syntax_status、compile_status、semantic_validation、verification、请求/完成阶段、coverage、execution、diagnostics 和 origin。P2 输出 SHALL 为 representation=Bytecode、syntax_status=NotJava、compile_status=NotAttempted；quality 按证据为 Conservative/Fallback。系统 MUST NOT 为恢复失败生成空 body、伪造 return 或 Java 成功标志。Java/Mixed 表示与 Structured 质量留给后续能力，不由当前管线产生。

#### Scenario: Complete bytecode with unsupported normalization

- **WHEN** 原始方法范围完整读取，但 legacy normalization 无法产出 CanonicalCFG
- **THEN** Bytecode 可按其声明 schema 返回 CompleteWithinSchema，quality=Fallback 且语法为 NotJava；未完成 IR 阶段和原因单独报告，不把 bytecode 覆盖当作 SSA 完成（验收 A09、A13）

#### Scenario: A later phase stops after canonicalization

- **WHEN** 已产出 CanonicalCFG，但 Frame 或 SSA 停止
- **THEN** quality=Conservative 只表示已保留规范化事实，execution 与对应阶段仍报告实际停止，semantic_validation=Unproven；不能将该质量等级解释为 Frame/SSA 成功

#### Scenario: Budget stop cannot be hidden by fallback

- **WHEN** 读取或构建过程中耗尽预算、取消或输入损坏
- **THEN** 保留可信前缀、实际 execution 和未完成范围，fallback 不重置预算、不重新扫描、不把 Partial 改成完整

#### Scenario: One method fails

- **WHEN** class 中一个方法损坏而其他方法可读，且剩余请求预算允许继续
- **THEN** 按成员隔离诊断和阶段结果；共享预算耗尽时剩余方法明确 skipped，不因某个方法质量降级而抹除其他方法成果（验收 A13）

#### Scenario: Abstract or native method

- **WHEN** 请求合法 abstract/native 方法且没有 Code
- **THEN** 返回声明及无 Body 的适用状态，不伪造空 Code，也不将其当 malformed 方法

### Requirement: Validation evidence is independent

系统 SHALL 独立报告 LocalInvariants、FixtureDifferential 或 Unproven 的语义证据及适用范围；未完整运行 verifier 时 MUST 返回 verification=NotPerformed。测试 oracle 的成功 MUST NOT 被推广到未经该验证的普通输入。

#### Scenario: SSA completes its local invariants

- **WHEN** 本次请求的 Ssa 阶段为 Completed，但未执行完整 verifier 或差分验证
- **THEN** 报告 LocalInvariants，verification 仍为 NotPerformed，compile_status 仍为 NotAttempted

#### Scenario: SSA has not completed

- **WHEN** 只请求 Frame、Ssa 停止或未调度，或 abstract/native 方法没有 Body
- **THEN** semantic_validation=Unproven，不因其他阶段完成或测试 oracle 成功升级普通请求

#### Scenario: Fixture oracle succeeds

- **WHEN** 固定样本通过 JVM 或其他差分 oracle，但生产方法请求未运行完整 verifier
- **THEN** 验证记录限定到该 fixture/profile，普通报告仍为 verification=NotPerformed；不能用测试通过升级生产 verifier 状态

### Requirement: Method-level demand boundary

方法分析 SHALL 只物化目标 Body、必要 Header 和有 reason 的最小依赖闭包，不要求预建全局 XRef、所有方法 Body、Region 或 Java AST。库与 CLI SHALL 使用相同结果和终止语义。

#### Scenario: Local method analysis

- **WHEN** 调用方请求一个方法的 Frame 或 SSA
- **THEN** 读取记录仅包含必要闭包，未请求的 Body 和更高阶段构造次数为零（验收 A14、A16）

#### Scenario: Structural query remains independent

- **WHEN** resolver/IR 已存在于同一库，但调用方只运行 P1 X0/X1
- **THEN** resolver、CFG、SSA、Region 和 Java AST 的构造次数均为零，原始 evidence、BCI 和 coverage 契约保持成立（验收 A17）

#### Scenario: CLI uses the library result

- **WHEN** 相同 snapshot、运行环境、方法与 budget 通过库及 JSON CLI 的 analyze_method 请求
- **THEN** CLI 返回 method_analysis，报告逐字段等同库结果（仅 elapsed_millis 可不同）；适配层不改写环境和方法身份，报告内 Partial/Cancelled 保留在成功 transport 中，不补造成功或重新解释 fallback
