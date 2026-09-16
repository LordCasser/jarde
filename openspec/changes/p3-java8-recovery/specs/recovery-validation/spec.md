## Purpose

建立以语料、重编译和受控行为对照为基础的 Java 8 recovery 质量门槛，明确语法、验证、可读性和语义等不同承诺。

## ADDED Requirements

### Requirement: Separate representation and validation statuses

每个 source result SHALL 独立返回 representation（Java/Bytecode/Mixed）、quality（Structured/Conservative/Fallback）、syntax_status（Checked/Unchecked/NotJava）、compile_status（NotAttempted/Compiles/Failed）、semantic_validation（LocalInvariants/FixtureDifferential/Unproven）、verification（Performed/NotPerformed/Failed）、recovery profile、coverage 和 diagnostics；`Mixed` 只表示 representation，禁止把它当作 quality。未实际编译时 compile_status MUST 为 NotAttempted；只通过局部不变量可记录 LocalInvariants，受控语料对照通过才记录 FixtureDifferential，无相应证据时记录 Unproven。生成 Java 文本不得自动意味着可重编译或语义等价。

#### Scenario: Structured output cannot compile

- **WHEN** 恢复结果可读但包含 Java 无法表达的名称或控制结构
- **THEN** 结果保留 representation=Java 或 Mixed、按结构化程度设置 quality=Structured/Conservative/Fallback，并设置 syntax_status=NotJava、compile_status=NotAttempted、semantic_validation=Unproven、verification=NotPerformed；只有实际执行编译且失败时才使用 compile_status=Failed，不宣称 Java 8 可重编译

#### Scenario: Complete scan with fallback quality

- **WHEN** 完整扫描成功但某些区域只能保留低级结构
- **THEN** 结果可为 representation=Mixed、quality=Fallback，同时 coverage=CompleteWithinScope；quality 不得改写为 Partial

#### Scenario: Controlled fixture equivalence

- **WHEN** 受支持 fixture 在隔离环境通过重编译和输入输出/异常对照
- **THEN** 仅该 fixture/profile 记入已验证覆盖，不能把单个样本推广成所有合法 JVM 方法

### Requirement: Published Java 8 support matrix

项目 SHALL 按 parse、X1、resolution、decompile-quality、output-level 发布 Java 8/历史 45–52 的支持矩阵，并明确代表性 javac/ECJ 与缺失依赖、混淆、无 debug、不可约 CFG 的降级。

#### Scenario: Unsupported compiler dialect

- **WHEN** 输入来自未验证的 Kotlin、Scala、Groovy、AspectJ、JSP 或字节码生成器模式
- **THEN** 使用 generic JVM/fallback 状态并注明 rule/profile，不根据 major 猜测唯一源码语言
