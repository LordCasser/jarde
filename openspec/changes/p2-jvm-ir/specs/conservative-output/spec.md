## Purpose

让方法分析在结构可读但语义恢复证据不足时仍能返回可复核结果，并将 Bytecode、Conservative、Mixed、失败和未执行验证清楚区分。

## ADDED Requirements

### Requirement: Honest fallback output

系统 SHALL 为每个方法独立返回 `representation`（Java/Bytecode/Mixed）、`quality`（Structured/Conservative/Fallback）、`syntax_status`（Checked/Unchecked/NotJava）、`compile_status`（NotAttempted/Compiles/Failed）、`semantic_validation`（LocalInvariants/FixtureDifferential/Unproven）、`verification`、`coverage`、`diagnostics` 和 origin。`Mixed` 仅表示 representation，不得被当作 quality；无法证明结构化 Java 等价时 MUST 保留 bytecode 或 conservative 表示，不生成空 body、伪造 return 或成功标志。

#### Scenario: Unsupported normalization

- **WHEN** jsr/ret、不可约 CFG、异常区域或 IR 膨胀超出支持范围
- **THEN** 输出 `quality=Fallback`、fallback 原因、已分析范围和原始 BCI evidence；若请求范围已完整扫描，coverage 仍可为 CompleteWithinScope，而不是把质量 fallback 误报为 Partial（验收 A09、A13）

#### Scenario: One method fails

- **WHEN** 同一个 class 中一个方法解析失败而其他方法可以读取
- **THEN** 结果按成员隔离诊断；只有扫描/解析未完成时才标记 Partial，质量为 Conservative/Fallback 本身不得改变 coverage（验收 A13）

#### Scenario: Complete scan with fallback quality

- **WHEN** 目标方法的请求范围已读取完毕，但 Region 或 Java 呈现无法证明等价
- **THEN** 结果可为 `representation=Bytecode|Mixed`、`quality=Fallback`、`syntax_status=Unchecked`、`compile_status=NotAttempted`、`semantic_validation=Unproven`、`coverage=CompleteWithinScope`、`verification=NotPerformed`

### Requirement: Method-level demand boundary

单方法 IR 请求 SHALL 只物化目标 Body、必要 Header 和被记录的最小依赖闭包；不得因为一个方法请求预先构建全局 XRef、所有方法 Body 或 Java AST。

#### Scenario: Local decompile request

- **WHEN** 调用方请求单个方法的 Frame 或 Conservative 输出
- **THEN** 执行记录显示只读取必要闭包，并满足不构建无关 Body 的门槛（验收 A16）
