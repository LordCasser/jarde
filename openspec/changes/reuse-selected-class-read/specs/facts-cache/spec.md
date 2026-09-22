## ADDED Requirements

### Requirement: A verified definition read is reusable within explicit bounds

系统 SHALL 支持显式启用的**定义读取**复用：一次已经完成的、可信的 class 定义读取，可绑定其定义身份（不可变 snapshot、完整物理位置、digest、length、变体）、读取 schema 版本与当时的验证结果被保留，并在后续足额请求上按同一身份作答。只发布**完整**的读取：部分构造、被取消、被限额终止或校验未完成的读取 MUST NOT 作为可复用事实发布。该层 SHALL 与 container/CP/Header 层**共用**同两个显式上限（entry 数与 retained-byte），满则拒绝、不淘汰，并报告命中、驻留权重与容量拒绝。命中 MUST NOT 计入本次 `archive_entries`/`read_bytes`/`entry_bytes`/`class_bytes`，MUST NOT 重放旧 usage、MUST NOT 重置已耗尽的预算或借用旧请求的额度；关闭、未命中或被拒绝时 SHALL 在**当前剩余预算**内走直读，语义与关闭时相同。

#### Scenario: The second request for the same definition

- **WHEN** 同一 snapshot 上同一定义的第二次足额请求到来，其身份（位置、digest、length、变体、schema）与已保留读取完全一致
- **THEN** 本次请求不再读取该 entry，`archive_entries`/`read_bytes`/`entry_bytes`/`class_bytes` 不因这次命中共增加；解析、准备与分析工作照常执行，且结果与直读路径的 fingerprint 相同

#### Scenario: A different snapshot with the same coordinates

- **WHEN** 两个 snapshot 的内容 digest 与物理坐标相同但 snapshot 身份不同
- **THEN** 第二个 snapshot 上的请求 MUST NOT 命中第一个 snapshot 的读取

#### Scenario: A declaration that does not match the read

- **WHEN** 调用方声明的 entry、变体或 digest 与已保留读取的任一身份维度不符
- **THEN** 不命中，走直读；MUST NOT 以"内容相近"作答

#### Scenario: Capacity refuses before it evicts

- **WHEN** 新读取超过单项可容纳范围或剩余 entry/retained-byte 容量
- **THEN** 报告容量拒绝并继续以直读完成当前请求；保留量不越限，已有事实不被淘汰

#### Scenario: Hit under an exhausted or cancelled request

- **WHEN** 当前请求已到期、已取消或预算已终止，而读取可命中
- **THEN** 返回该请求的真实终止原因，不因命中提交产物或恢复已耗尽的预算

## MODIFIED Requirements

### Requirement: Complete cache identity and invalidation

任何 cache/index entry SHALL 按所在缓存层绑定其实际依赖的语义维度：CP/Header 使用 class 内容摘要、parser/registry 版本和 parse policy；**定义读取层**使用不可变 snapshot、完整物理位置（origin chain 与 ordinal）、内容 digest、声明 length、变体（multi-release 选择结果）与读取 schema 版本，并记录当时完成的校验（CRC/size/span）；X1 使用 class/resource 摘要与 consumer/scanner schema；resolution 再加入 symbol/source context、view/domain/platform/dependency snapshot；IR/source 按需增加方法内容、analysis/recovery 版本、依赖事实、output level 与命名配置。物理 origin 在返回结果时单独绑定，MUST 不因内容缓存共享而合并；不完整结果不得覆盖完整项。输入或语义依赖改变时 MUST 失效或返回不命中。

#### Scenario: Dependency becomes available

- **WHEN** 缺失依赖导致的 negative/partial cache entry 之后补齐 Header provider
- **THEN** 旧 entry 不得覆盖新的可解析结果，系统必须重新查询或明确使用不同 key

#### Scenario: Runtime profile changes

- **WHEN** 同一物理 artifact 以另一个 RuntimeProfile 或 output level 查询
- **THEN** 不复用会改变选择/语法的旧 entry，结果携带新的 view/output key

#### Scenario: Cache key follows layer dependencies

- **WHEN** 同一 snapshot 的原始 CP/Header/X1 查询与 resolution、IR 或 source 查询分别建立缓存
- **THEN** 原始层 key 不因无关 RuntimeProfile、output level 或 recovery 变化而失效；resolution/IR/source 层按各自实际依赖加入 RuntimeView、platform、registry、IR 或 recovery 上下文

#### Scenario: A read is reused only under its own identity

- **WHEN** 同一物理位置在不同 snapshot、不同变体选择或不同读取 schema 下被再次请求
- **THEN** 定义读取层 MUST 不命中，除非上述每一维度都与保留时逐项相同；MUST NOT 只按位置或只按内容摘要作答

### Requirement: Verified container facts are reusable within explicit bounds

系统 SHALL 支持显式启用的内存 container facts 复用，绑定不可变 snapshot、完整物理 container origin 和解析/验证版本。仅完整、已验证事实可作为完整目录发布；MUST 保留同 raw name 的全部物理 entries 及稳定顺序。关闭复用时仍可执行相同语义的直接路径。保留事实与 backing 的总量 SHALL 同时受显式 entry 数与 retained-byte 上限约束，并报告命中、驻留权重及容量拒绝；容量不足不改变分析语义。

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

- **WHEN** 新请求命中已验证的 DEFLATED backing 和目录，随后仍需取得所选 class
- **THEN** 仍须在该目录上**定位该 entry 并核对变体与调用方声明的身份**（内容、origin chain、解析/验证版本），该核对不得跳过；读取本身可由该定义的已保留读取作答。被复用部分不重复计入 archive_entries/read_bytes/entry_bytes，实际发生的工作按当前剩余额度计费；超过限额立即保持真实停止，不从旧请求借用额度或重新创建 Budget

#### Scenario: Expired or already terminated warm request

- **WHEN** 缓存可命中，但当前请求 elapsed 已到期，或先前操作已使该请求取消/预算终止
- **THEN** 命中不能恢复该请求或提交 Complete，报告实际终止原因
