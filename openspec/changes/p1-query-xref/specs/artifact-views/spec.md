## Purpose

为嵌套归档、MR-JAR 与 Boot 布局建立可复核的物理和运行时选择模型，避免把文件路径启发式或单一 classpath 视为 JVM 的完整加载规则。

## ADDED Requirements

### Requirement: Physical and runtime views are separate

系统 SHALL 先保留选定范围内全部物理定义和 entry，再根据显式 RuntimeProfile、LoadDomain、版本和布局规则派生 RuntimeView。物理视图不得因运行时选择而丢失证据。

#### Scenario: Multi-release variants

- **WHEN** 同一 MR-JAR 含 root、versions/11 和 versions/17 变体
- **THEN** PhysicalView 列出全部变体，Java 8、11、17 的 RuntimeView 分别选择正确候选（验收 A06）

#### Scenario: Duplicate definitions in WAR

- **WHEN** WAR 或嵌套依赖中存在多个同名类
- **THEN** 系统保留每个物理 origin，并在 RuntimeView 中保留显式 loader/order 条件；P1 不做唯一解析或静默消歧（验收 A07）

### Requirement: Bounded nested and Boot layout traversal

系统 SHALL 仅在调用方显式请求 artifact-tree scope 时递归 nested JAR；普通 physical snapshot 枚举只保留 candidate，不把标准 JAR 内任意 archive 自动激活为运行时 classpath。Provider SHALL 将 nested JAR、`BOOT-INF/classes`、`BOOT-INF/lib`、WAR 的 `WEB-INF/classes` 和 `WEB-INF/lib` 表示为带 evidence 的物理布局节点，并 SHALL 允许通过 origin chain 从 root snapshot 逐层复核并重读 nested entry。递归展开 MUST 受 `nested_depth` 高水位、archive entry、read/entry bytes、elapsed 和取消预算约束，并且不得自动联网、执行 launcher 或推断唯一运行时选择。

`nested_depth` SHALL 以 root container 为 0、直接 child 为 1；它记录请求达到的最大已接受深度，不作为累加 charge。Root container 建立后的 child malformed、unsupported、budget 或 cancellation MUST 返回可靠前缀、定位到父 entry 的 diagnostic、已扫描/未扫描 coverage 与非 Complete execution；不得返回空 child 冒充成功。

#### Scenario: Nested candidate without explicit tree scope

- **WHEN** 标准 JAR 含任意 nested JAR，但调用方只请求普通 physical snapshot 枚举
- **THEN** nested entry 保持 candidate-not-scanned，系统不递归、不把它加入运行时 classpath

#### Scenario: Nested DEFLATED archive

- **WHEN** 调用方显式请求 artifact-tree，嵌套 JAR 使用 DEFLATED 且展开后不超过预算
- **THEN** 系统物化并扫描其内容，layout node 记录真实父 entry 与 child container，内层 entry 保留完整 origin chain，且后续可从 root snapshot 重读（验收 A08）

#### Scenario: Nested traversal exceeds budget

- **WHEN** 递归展开超过任一 `nested_depth`、archive entry、read/entry bytes、elapsed 或取消预算
- **THEN** 结果声明具体预算维度或 cancellation、已扫描范围和未扫描范围，保留已完成 container/entry 的可靠前缀，且不标为 Complete（验收 A14）

#### Scenario: Malformed child does not erase siblings

- **WHEN** root 已建立且一个 nested child 在物化、ZIP open 或 central-directory 枚举阶段损坏，或使用不支持的压缩，而其他 sibling 可读
- **THEN** 结果保留可读 sibling，整体为 Partial，并对失败父 entry 给出 Error/Unsupported diagnostic；已建立但枚举失败的 child 保留非 Complete container report，失败 child 不以空列表表示成功

#### Scenario: Terminal budget after a local child failure

- **WHEN** 一个 child 已产生可继续的 Error/Unsupported，后续 sibling 又耗尽非深度预算或收到取消
- **THEN** traversal 立即停止，aggregate execution 报告实际阻止继续扫描的预算维度或 Cancelled；此前 child diagnostic 和可靠前缀仍保留

#### Scenario: Nested entry replay is bounded and revalidated

- **WHEN** 调用方用 artifact-tree 返回的 nested entry 从 root snapshot 重读
- **THEN** 系统逐层验证真实父 entry、candidate 状态、派生 child container 和最终完整 metadata，并对每层执行 `nested_depth`、archive entry、read/entry bytes、elapsed 与取消检查；预算或取消错误保持其原始结构，不改写成普通 invalid-input

#### Scenario: Ordinary ZIP bytes cannot forge a child container

- **WHEN** 非 `.jar`/`.war` candidate 的普通 entry 恰好包含合法 ZIP bytes，调用方手工构造 origin chain
- **THEN** nested entry replay 拒绝该 chain，普通物理 entry 不因内容可解析为 ZIP 而成为 artifact-tree child
