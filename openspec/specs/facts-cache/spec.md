# facts-cache Specification

## Purpose
为已证明有收益的 facts cache 或索引提供明确身份与失效边界，使缓存只加速读取而不改变结构、解析、恢复和降级结果。

## Requirements

### Requirement: Complete cache identity and invalidation

任何 cache/index entry SHALL 按所在缓存层绑定其实际依赖的语义维度：CP/Header 使用 class 内容摘要、parser/registry 版本和 parse policy；X1 使用 class/resource 摘要与 consumer/scanner schema；resolution 再加入 symbol/source context、view/domain/platform/dependency snapshot；IR/source 按需增加方法内容、analysis/recovery 版本、依赖事实、output level 与命名配置。物理 origin 在返回结果时单独绑定，MUST 不因内容缓存共享而合并；不完整结果不得覆盖完整项。输入或语义依赖改变时 MUST 失效或返回不命中。

#### Scenario: Dependency becomes available

- **WHEN** 缺失依赖导致的 negative/partial cache entry 之后补齐 Header provider
- **THEN** 旧 entry 不得覆盖新的可解析结果，系统必须重新查询或明确使用不同 key

#### Scenario: Runtime profile changes

- **WHEN** 同一物理 artifact 以另一个 RuntimeProfile 或 output level 查询
- **THEN** 不复用会改变选择/语法的旧 entry，结果携带新的 view/output key

#### Scenario: Cache key follows layer dependencies

- **WHEN** 同一 snapshot 的原始 CP/Header/X1 查询与 resolution、IR 或 source 查询分别建立缓存
- **THEN** 原始层 key 不因无关 RuntimeProfile、output level 或 recovery 变化而失效；resolution/IR/source 层按各自实际依赖加入 RuntimeView、platform、registry、IR 或 recovery 上下文

### Requirement: Cache is optional and transparent

cache/index 不得成为正确性前置条件；关闭、损坏或版本不兼容时系统 SHALL 在剩余预算内回退到同一语义的直接扫描，并报告 cache 状态。请求已取消或预算耗尽时 MUST 返回终止状态，不得重置预算重跑，也不得伪造 Complete。

#### Scenario: Corrupt cache entry

- **WHEN** cache entry 校验失败或格式版本未知
- **THEN** 丢弃该 entry 并在剩余预算内执行直接路径；相同配置的完整执行与无缓存路径语义一致，无法完整执行时返回 Partial（验收 A15）

#### Scenario: Index candidate requires verification

- **WHEN** 索引只提供潜在 candidate 而未证明 consumer/definition 关系
- **THEN** 查询仍需经过结构 consumer/解析验证，不把索引命中直接作为 XRef 事实（验收 A01）

#### Scenario: Cancelled or exhausted execution cannot reset the budget

- **WHEN** 直接路径或加速路径已经取消或耗尽预算
- **THEN** 系统返回已处理范围和终止原因；禁用或损坏缓存可在剩余预算内有界回退，取消或预算耗尽不得重置预算并自动重跑

### Requirement: Verified container facts are reusable within explicit bounds

系统 SHALL 支持显式启用的内存 container facts 复用，绑定不可变 snapshot、完整物理 container origin 和解析/验证版本。仅完整、已验证事实可作为完整目录发布；MUST 保留同 raw name 的全部物理 entries 及稳定顺序。关闭复用时仍可执行相同语义的直接路径。保留事实与 backing 的总量 SHALL 同时受显式 entry 数与 retained-byte 上限约束，并报告命中、驻留权重及容量拒绝；容量不足不改变分析语义。共享 store 中的 container、definition-read 与 CP/Header 已保留条目 MUST 全部计入同一总 entry 上限，与插入顺序无关。失效移除条目时 SHALL 同步扣减该 store 持有的权重，不把其他请求仍持有的引用声称为已释放内存。

#### Scenario: Different methods reuse the same container

- **WHEN** 同一 snapshot/container 的完整目录与 backing 已保留，后续足额请求访问其中另一个方法
- **THEN** 后续请求复用目录与 nested backing，不重新解析整个目录或重复解压已保留父容器，不因查名遍历或复制全部目录；只处理查询实际需要的候选和方法

#### Scenario: Duplicate physical entries survive reuse

- **WHEN** 同一查名位置含两个 raw name 相同、内容也相同的 entries
- **THEN** direct 和 warm 均保留各自 ordinal/origin，仍返回该位置的 Ambiguous，不按名字或内容摘要合并

#### Scenario: Snapshot or validation identity changes

- **WHEN** 输入内容、origin chain 或解析/验证版本改变
- **THEN** 旧目录/locator 不作为新输入的权威事实，系统重新验证或明确返回不命中

#### Scenario: Retained capacity is insufficient

- **WHEN** 新完整事实超过单项可容纳范围或剩余 retained-byte/entry 容量
- **THEN** 报告容量拒绝，当前请求继续消费已取得事实而不重做扫描；后续请求可在剩余预算内走直接路径，保留量不越限

#### Scenario: Partial construction and release

- **WHEN** container 构造中途停止，或持有复用事实的最后一个使用者释放该缓存
- **THEN** 未完成事实不冒充完整项；无使用者的保留 backing 与目录引用被释放，不能由隐式全局 store 无限保留

#### Scenario: Clearing retention preserves live request references

- **WHEN** store 被清空或释放，而活动请求仍持有已验证目录或 backing 的有效引用
- **THEN** 只释放 store 的持有关系，活动引用继续有效并允许该请求在自身剩余预算内消费事实；不得因此重建、悬挂或错误终止，不把仍被活动请求持有的内存报告为已经释放

#### Scenario: Mixed retained layers share one entry allowance

- **WHEN** entry 上限为二，store 已保留一个 container 和一个 definition-read，后续得到完整 CP/Header 事实；或同样三类事实以其他顺序到达
- **THEN** 第三个新条目被拒绝保留，总条目数仍为二，报告 entry 容量拒绝而非字节容量拒绝，当前请求仍可消费已取得的完整事实

#### Scenario: Incompatible facts release their store weight

- **WHEN** store 的 CP/Header 项因 format、registry 或 product 不兼容被丢弃，随后相同容量下完成一次兼容解析
- **THEN** store 的总条目及 retained-byte 权重只包含仍保留项；兼容结果可以再次保留，不因被移除项的旧权重永久拒绝，也不需要调用 clear 才恢复容量

#### Scenario: Removing one entry preserves other retention and live payloads

- **WHEN** 混合 store 中一个不兼容 CP/Header 项被移除，而其他层条目和一个先前请求的有效 payload 引用仍存在
- **THEN** 仅扣减被移除项的 store 权重，其他保留项的命中和权重保持正确，活动 payload 引用继续有效

### Requirement: Reuse preserves current request limits and evidence

复用 SHALL 使用当前请求的取消、时间和适用的结构/大小/深度限制；实际工作计费与复用证据 MUST 分开。命中不得伪造本次读取或重放旧 usage，不得重置已耗尽预算；当前返回结果仍受输出限制。两个足额完整执行的语义结果 SHALL 相同；工作量节省导致紧预算下完成程度不同可以报告，但不得伪造 Complete。

#### Scenario: Precancelled warm request

- **WHEN** 完整 facts 已预热但新请求进入时已取消
- **THEN** 返回 Cancelled，不因命中直接提交成功产物

#### Scenario: Warmed under a wider depth allowance

- **WHEN** 深层 nested container 在较宽限制下预热，新请求的 nested depth 限制不足
- **THEN** 新请求仍按当前深度限制停止，不绕过结构安全边界

#### Scenario: Physical metadata cannot be forged

- **WHEN** 调用方提交 snapshot、origin、ordinal 或 metadata 被篡改的 entry，且 store 中有相近事实
- **THEN** 拒绝不匹配身份；命中不能替代对物理 locator 和所选 entry 内容的完整性检查

#### Scenario: Verified backing saves work but does not grant a fresh budget

- **WHEN** 新请求命中已验证的 DEFLATED backing 和目录，随后仍需读取所选 class
- **THEN** 被复用部分不重复计入 archive_entries/read_bytes/entry_bytes，实际 class 读取按当前剩余额度计费；超过限额立即保持真实停止，不从旧请求借用额度或重新创建 Budget

#### Scenario: Expired or already terminated warm request

- **WHEN** 缓存可命中，但当前请求 elapsed 已到期，或先前操作已使该请求取消/预算终止
- **THEN** 命中不能恢复该请求或提交 Complete，报告实际终止原因
