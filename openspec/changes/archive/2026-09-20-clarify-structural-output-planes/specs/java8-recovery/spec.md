## MODIFIED Requirements

### Requirement: Recovery artifact content is observable independently

恢复报告 SHALL 返回 `content`，取值为 `not_produced`、`explanation_only`、`contains_statements`。该字段 SHALL 描述最终交付产物：Stopped 为 not_produced；Produced 且只有包装、理由、BCI 引用等说明为 explanation_only；Produced 且含至少一个实际发射的 Java 语句为 contains_statements。Produced MUST 仅表示产物已交付，不能解释为存在语句、语义完整、可编译或已验证。

声明、赋值、调用、return 和控制流语句均可构成 contains_statements；该分类不改变 representation、quality、syntax、coverage、execution 或验证状态。分类 MUST 不依赖注释剥离、token 相似度或调用方二次解析文本。库与 CLI SHALL 返回同一分类。

`Produced` 与 `content` MUST 分开统计与表述：同一轮 sweep 的 182,883 个请求中有 30,452 个是 Produced 而产物不含语句；报告、README、支持矩阵与 benchmark 协议 MUST NOT 把这类计数表述为语句恢复率或语义恢复率，MUST 分别命名「引擎 content 分类」与旧 token/启发式口径。content 与其余平面一样是结构性的，MUST NOT 被读作值等价或语义正确的证据。

#### Scenario: Explanation-only produced artifact

- **WHEN** 方法产物只有 BCI 引用、降级理由和成员包装说明
- **THEN** outcome 仍为 Produced，content 为 explanation_only，不因文本非空或存在 fallback 节点被统计为 contains_statements

#### Scenario: Partial structure has a statement

- **WHEN** 产物包含 `if (arg0) {}` 或已证明的 `return;`，同时仍有未恢复区域的说明
- **THEN** content 为 contains_statements；既有 Mixed/Fallback 与未证明语义保持原状，不标为完整 Java 恢复

#### Scenario: Text emission stops

- **WHEN** 构建已产生语句但输出阶段被预算或取消终止，最终未提交产物
- **THEN** content 为 not_produced，与 Stopped 及实际 text/source map 契约一致，不报告未交付 AST 的内容

#### Scenario: Formatting and literals do not change classification

- **WHEN** 同一结构改变注释措辞、空白，或字符串字面量包含注释分隔符
- **THEN** 内容分类不因这些文本变化而改变；公开库和 JSON CLI 对同一请求报告相同分类

#### Scenario: A produced count is not a statement count

- **WHEN** 报告或文档给出某组请求的 Produced 数量或比例
- **THEN** MUST 同时给出 content 分类（或明确声明该数不含内容分类），MUST NOT 把它标注为语句恢复率、语义正确率或行为等价率；两个口径分别命名，不合并成一个数字

#### Scenario: Produced without a statement stays visible as such

- **WHEN** 一组请求的产物只有包装、理由与 BCI 引用
- **THEN** 它们计入 Produced 且 content 为 explanation_only；报告与文档 MUST 能同时读出这两个数，不得因「有产物」就把它们计成含语句
