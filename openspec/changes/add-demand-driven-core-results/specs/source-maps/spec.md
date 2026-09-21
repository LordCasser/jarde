## MODIFIED Requirements

### Requirement: BCI-to-source evidence

系统 SHALL 为恢复的声明、表达式、语句、异常路径和降级片段提供 origin set，能够映射到原始 BCI/attribute/CP 位置；每个位置 MUST 绑定实际物理定义及适用的完整方法身份，复用既有身份词汇，不能把 BCI 当作跨方法全局键。派生 accessor/lambda 映射 MUST 标注派生关系和覆盖范围。

恢复内部 SHALL 始终保留构造正文、降级和证据判断所需的 origin；默认必要结果 MUST 不构造完整映射段表，并明确区分 `NotRequested`、已请求的 Complete 空映射、Partial、NotPerformed 和不支持。仅在序列化时删掉完整表不满足要求。选择交付映射时，只能物化所选范围及解释它所需的 origin 闭包，并将每个位置绑定到确切 artifact、物理方法、分析配置和当前文本产物；局部范围涉及 accessor/lambda/callee 时，跨方法来源也必须保留各自完整身份。

本 requirement 中公开映射的交付场景以已选择相应映射为条件；未选择时内部 origin 与核心拒绝位置仍保留，完整选择继续交付原有全部映射。

#### Scenario: Accessor-derived field expression

- **WHEN** Java 输出把 accessor 调用呈现为外层字段访问
- **THEN** 映射同时包含 caller 的物理方法/调用 BCI 与 callee 的物理方法/字段 BCI，并标记派生关系；两者 BCI 相等时仍为不同位置，CP index 也按所属定义解释

#### Scenario: Partial method output

- **WHEN** 一个方法只有部分 region 成功恢复
- **THEN** 成功区域和 fallback 区域各自有映射与诊断；representation 可为 Mixed，quality 按区域独立为 Structured、Conservative 或 Fallback；只有扫描未完成时整体 coverage 才为 Partial（验收 A13）

#### Scenario: Default recovery keeps internal origins without a full map

- **WHEN** 调用方请求源码和必要拒绝缺口但未选择完整 source map
- **THEN** 内部判断仍保留所有正文和拒绝所需的 origin，返回结果明确 source-map 为 NotRequested；不得为了证明内部事实而构造完整映射段表，也不得把省略映射误报为空 Complete

#### Scenario: Selected mapping binds the exact artifact

- **WHEN** 调用方选择某一方法或 BCI 范围的 source map
- **THEN** 返回的每个映射绑定当前 snapshot、物理定义、完整方法身份、分析/输出配置和确切正文身份；相同 class 名、签名或 BCI 但不同 artifact 的映射不得复用

#### Scenario: Local range carries cross-method origins

- **WHEN** 所选 BCI 范围的表达式由 accessor、lambda 或 callee 的事实解释，且相关来源位于范围外或另一物理方法
- **THEN** 只交付与所选位置相关的来源闭包，但保留 caller 与 callee 各自的物理方法和 BCI/CP 身份，并标记派生关系；不得把相同 BCI 合并成一个位置

#### Scenario: Mapping stop is separate from body quality

- **WHEN** 正文已经提交而所选映射构造因预算或取消停止，或已请求映射尚未开始便停止
- **THEN** 正文的 representation/content/quality 保持原语义，映射按实际进度单独报告 Partial 或 NotPerformed 及原因，整体 execution 保留真实停止；不能因正文存在把映射说成 Complete，也不能把已请求但中止改成 NotRequested
