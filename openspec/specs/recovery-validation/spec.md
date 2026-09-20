# recovery-validation Specification

## Purpose
建立以语料、重编译和受控行为对照为基础的 Java 8 recovery 质量门槛，明确语法、验证、可读性和语义等不同承诺。

## Requirements

### Requirement: Separate representation and validation statuses

每个 source result SHALL 独立返回 representation（Java/Bytecode/Mixed）、quality（Structured/Conservative/Fallback）、syntax_status（Checked/Unchecked/NotJava）、compile_status（NotAttempted/Compiles/Failed）、semantic_validation（LocalInvariants/FixtureDifferential/Unproven）、verification（Performed/NotPerformed/Failed）、recovery profile、coverage 和 diagnostics；`Mixed` 只表示 representation，禁止把它当作 quality。未实际编译时 compile_status MUST 为 NotAttempted；只通过局部不变量可记录 LocalInvariants，受控语料对照通过才记录 FixtureDifferential，无相应证据时记录 Unproven。生成 Java 文本不得自动意味着可重编译或语义等价。验收 SHALL 独立核对已知反例的返回值/可观察行为与 fallback 的生产者来源完整性；语义已知错误的输出不得因 Unproven 或既有门禁绿色被计为完成。

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

### Requirement: Published Java 8 support matrix

项目 SHALL 按 parse、X1、resolution、decompile-quality、output-level 发布 Java 8/历史 45–52 的支持矩阵，并明确代表性 javac/ECJ 与缺失依赖、混淆、无 debug、不可约 CFG 的降级。

#### Scenario: Unsupported compiler dialect

- **WHEN** 输入来自未验证的 Kotlin、Scala、Groovy、AspectJ、JSP 或字节码生成器模式
- **THEN** 使用 generic JVM/fallback 状态并注明 rule/profile，不根据 major 猜测唯一源码语言

### Requirement: Coverage metrics distinguish artifact content and validation

恢复评测 SHALL 分别报告请求适用范围、有产物、含语句、只有说明和停止数量，并注明分母与分类版本；MUST NOT 把 Produced 比例称为语句恢复率或行为正确率。文本、调用序列、字符串相似度 SHALL 各自报告有效 pair 数和排除原因。行为对照只为实际执行的受控 fixture 提供证据，不能由相似度或多数投票推导语义等价。

#### Scenario: Produced includes explanation-only outputs

- **WHEN** 一组请求产生含语句、仅解释及停止三种结果
- **THEN** 报告可对账的三类数量与 Produced 的合计；没有 Code 的方法单列适用状态，不用纯解释结果抬高语句覆盖

#### Scenario: Reference decompiler folds initializers

- **WHEN** 对照工具折叠 `<clinit>` 或省略隐式 `<init>`，且无法建立方法级 pair
- **THEN** 报告表示差异/未匹配范围，不自动计为任一引擎失败或正确，并从相应 pair 指标分母中明确排除

#### Scenario: Historical heuristic differs from structural classification

- **WHEN** 旧报告用去注释 token 判内容，新报告使用引擎 content
- **THEN** 保留旧分类定义并标明新版本，记录差异样例，不静默重写旧覆盖数字或把两者视为同口径
