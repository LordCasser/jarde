# Spec Delta

## MODIFIED Requirements

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
