## Purpose

为显式选定的 artifact 物理范围提供有界并行的全量方法恢复与持续导出，使容器和类准备工作能被复用，同时让调用方准确区分方法恢复结果、范围处理完成、实际交付、资源中止及尚未读取的部分。

## ADDED Requirements

### Requirement: Bulk recovery has an explicit physical scope

系统 SHALL 提供库级批量恢复操作，接收不可变 snapshot、物理范围、显式环境策略、worker 数、总预算和局部限制。范围中的类候选、可读声明及全部方法声明 SHALL 可对账；每个已读方法包括 abstract/native、失败或未产出的方法都 MUST 有明确处置。物理同名/同字节类不合并，重复方法声明不任取首项；未提供的依赖不自动下载或扩张为导出目标。操作 MUST NOT 把方法正文集合宣称为完整可编译 Java 类/工程。

#### Scenario: Export every method declaration in a nested scope

- **WHEN** 显式物理范围包含 standalone class 或带 nested 容器的 artifact，且运行足额完成
- **THEN** 每个已选类和方法声明可由真实 origin、成员 ordinal、raw name/descriptor 对账；无 Body、拒绝和未产出均有记录，普通 resources 不恢复为方法，子容器是否包含由 scope 决定（B01；A07/A08/A13）

#### Scenario: Duplicate definitions and incomplete discovery

- **WHEN** 范围内存在同名类/同签名声明，或发现过程在后缀停止
- **THEN** 保留各自物理来源和歧义；未读后缀的类/方法数保持未知，不把当前已发现数量当完整分母，不返回完整空集合（B01；A07/A14）

### Requirement: Bulk recovery shares preparation without changing recovery semantics

同一次批量操作 SHALL 复用仍持有的完整容器事实，以及同一类的可信读取、结构与方法定位。每个方法 SHALL 使用现有分析/恢复能力，并保留独立的 content、quality、coverage、execution、diagnostics 和 source map；只消费共享事实的请求 MUST 引用其准备证据，不伪造本方法重新完成了一次物理读取。普通单方法和查询入口 MUST 保留按需边界。

#### Scenario: Recover several methods of one class

- **WHEN** 同一类的多个方法和同类 accessor callee 被处理
- **THEN** 完整 class 准备一次，仍持有的结构和定位不按方法重建；每个需求 Body 在持有其事实期间只解码一次，未请求且未被 callee 规则要求的 Body 不额外解码；逐方法正文和来源与相同配置的独立恢复一致（B02；A12/A15/A16）

#### Scenario: Reuse does not start a bulk operation implicitly

- **WHEN** 调用方仅打开 artifact、列举成员、查询 X1 或恢复单个方法
- **THEN** 不启动批量恢复发现或 worker；只按本次请求及既有 consumer/callee 规则读取所需 Body，不额外构建请求未要求的 XRef/IR（B02；A16/A17）

### Requirement: Parallel execution is explicitly bounded

批量操作 SHALL 支持一个或多个 worker，公布请求值和实际启用值，运行中的类任务、准备字节、结果等待窗口和单方法资源都 SHALL 有显式上限。增加 worker MUST NOT 复制整份 snapshot 或隐式增大总额度。调用返回前 MUST 等待本次 worker 退出并释放其引用；创建线程失败或 worker 异常 MUST 明确结束操作，不能静默伪造串行成功。

#### Scenario: Multiple classes run concurrently

- **WHEN** 至少两个可独立处理的类被选中，worker 数大于一且资源足够
- **THEN** 可观察到类任务重叠执行，活动任务不超过声明上限；逐方法可变分析状态不被混用，完整恢复结果与单 worker 对照相同（B03；A15/A18）

#### Scenario: Too little capacity or worker creation fails

- **WHEN** 某类超过单项准备容量，或 worker 创建/执行失败
- **THEN** 超大类有带位置的拒绝且不永久等待无法取得的容量；线程生命周期失败返回 Failed，停止新派发并收回已经启动的 worker，不遗留后台任务（B03/B06）

### Requirement: A bulk operation cannot multiply its budget by its worker count

发现、准备、各方法执行和输出 SHALL 共同受一次操作的累计额度与截止时间约束；各方法另受其局部限制。每次实际工作 MUST 同时获得适用的全局/局部额度，失败的许可不得开始工作。共享准备只计一次；累计维度求和，高水位维度取最大，elapsed 为操作墙钟而非各 worker 时间之和。计费、容量驻留和 RSS MUST 分列。总额度耗尽或全局取消后 MUST 停止派发，已有任务协作停止；局部方法失败可继续其它任务，但顶层 execution 非 Complete。

#### Scenario: Two workers race for the last quota

- **WHEN** 两个 worker 竞争不足以容纳两次工作的剩余额度
- **THEN** 实际累计使用量不越限；未取得额度的工作不执行；已付工作、停止维度及所有 worker 最终状态均进入总账，不以每方法新 Budget 规避上限（B04；A14）

#### Scenario: One method stops locally

- **WHEN** 一个方法损坏、可靠拒绝或耗尽局部额度，而全局额度仍足够
- **THEN** 其它已选方法可继续；该方法保留原停止/拒绝结果，正常方法不被删除；只含解释的产物不计作含语句覆盖，部分执行不能被总批次 Complete 掩盖（B04/B07；A13）

### Requirement: Ordered delivery has bounded backpressure

系统 SHALL 按固定的物理遍历顺序及成员声明顺序交付，不能按完成先后改变结果身份。生产速度超过消费速度时 SHALL 停止继续生产或派发，使待交付结果数量/权重保持有界。单项结果容量 MUST 独立于 worker 数固定；总窗口容量不足以容纳所请求槽数时只减少有效并发并发布该值，不得通过缩小单项容量改变可接受结果。最早未交付任务 MUST 不因后续结果占满缓冲而失去发布机会。外部停止、取消、消费错误和超大单项 SHALL 能结束内部等待；库不承诺强制打断调用方无限阻塞的 sink。

#### Scenario: A slow early class and fast later classes

- **WHEN** 第一个类较慢，后续类迅速生成结果且 sink 很慢
- **THEN** 输出顺序仍稳定，缓冲不随全包方法数增长，早期任务能够继续发布，不出现等待容量的环路或整包结果收集（B05）

#### Scenario: Stop while producers wait

- **WHEN** worker 等待空闲输出位置时发生全局取消或 sink 拒绝继续
- **THEN** 内部等待被唤醒，不再生成后续结果，调用在协作边界收回所有任务，并保留已确认交付前缀（B05/B06；A14）

#### Scenario: More workers cannot shrink a result limit

- **WHEN** 只增加请求 worker 数，单项及总窗口容量保持相同
- **THEN** 不因重新平分窗口而拒绝原本可容纳的单项；有效 worker 可减少并显式报告；超过固定单项上限的记录返回可定位停止，不无限等待，也不截断正文后标成功（B05）

### Requirement: Execution coverage and delivery coverage are distinct

批量结果 SHALL 分开报告发现范围、执行范围和 sink 确认的交付范围，并区分已执行未交付与未执行。最终报告 SHALL 使用既有 Complete/Partial/Cancelled/Failed 执行词汇：方法错误或部分分析使 aggregate 非 Complete，即使遍历已经到尾。完整遍历、全部要求的执行完成且全部结果交付，才可发布 Complete。并行中止允许部分集合受调度影响；已交付数据仍必须有序、唯一且真实。缺少最终记录的流 MUST 被读作未确认完成。

#### Scenario: Cancel after a later method finished but before delivery

- **WHEN** 后序任务已经计算完成，前序位置尚未交付时发生取消
- **THEN** 后序工作进入实际 usage 和执行覆盖，未获 sink 确认的结果不计作交付；报告可有执行孔洞，不能把丢弃缓冲当作未做过工作，不能伪造全局 Complete（B06/B07）

#### Scenario: Full execution with different worker counts

- **WHEN** 相同输入和语义配置以 1 与 N worker 足额运行
- **THEN** 展开共享证据后的逐方法内容、身份、来源、规则、诊断和语义覆盖一致且输出有序；调度参数、实际使用量和耗时单独记录，不承诺跨调度相同资源 fingerprint（B07；A15）

### Requirement: CLI export is a streaming adapter

CLI SHALL 通过同一库操作提供 `export`，接收显式 input/scope/policy、输出文件和 `--jobs auto|N`，默认 auto；auto 的解析值 MUST 写入有效配置，0 或非法数值拒绝。首版交付逐记录 JSONL 方法产物流，包含开始配置、类准备、方法结果、诊断及最终汇总，不为每个方法启动进程。输出文件不得覆盖已有文件；编码和写出 SHALL 受输出额度限制，只把完整写出的记录计作已交付。最终事件 MUST 在 flush/close 成功后确认；没有最终 Complete 记录、最终确认失败或出现任何全局/方法执行停止，MUST NOT exit 0。I/O 错误保留可识别前缀，返回失败且不能自动重跑，即使已写出最终行也不得隐藏随后观察到的 I/O 错误。

#### Scenario: Auto and explicit worker settings

- **WHEN** 执行 `export --jobs auto` 或 `export --jobs 1`
- **THEN** 同一进程内完成请求；auto 在可用并行度和显式资源范围内选择 worker，1 使用串行路径，结果记录实际配置，不承诺一个请求一个新 cache（B08）

#### Scenario: Output stops in the middle

- **WHEN** 输出额度不足、文件写入失败或消费端中断
- **THEN** 停止生产并回收任务；已完整记录保持可读，残缺末行不计作交付；只有最终记录明确 Complete 才能确认全部完成，CLI 使用既有失败或执行未完整退出状态（B08；A14）

#### Scenario: Explanation is an outcome, not recovered statements

- **WHEN** 全量产物含 explanation_only 或 not_produced
- **THEN** 汇总分别计数，并保留原方法报告；遍历完成不升级其 content/quality，不宣称整包 Java 源码已经恢复或编译成功（B07/B08）


### Requirement: Active state ends with demand rather than retention admission

物理游标及已派发任务 SHALL 持有消费所需的可信容器事实，retention 关闭、满额或被清理 MUST NOT 使仍持有的事实重新物化。串行与并行路径 SHALL 在交付/处置完成后退役类状态；内存中未退役类记录不得随已完成类总数增长。单类物化 SHALL 先满足 max_class_bytes 及适用的读取额度；整包总额度不得隐式取代独立的方法局部限制。

#### Scenario: One worker finishes before the request deadline

- **WHEN** 单 worker 已交付所有成员和 ClassEnd，剩余操作仅需发布 Final
- **THEN** 已结束类不遗留待结束槽，正常清理不进入 worker 等待超时；不能由无任务可等待的固定延迟耗尽 deadline（B03/B05/B06）

#### Scenario: A current container cannot enter retention

- **WHEN** 游标仍持有一个 container，而 store 为零容量、超容量或刚被 clear
- **THEN** 同次操作派发该 container 的类继续使用其可信事实；目录和 nested backing 不因 cache miss 重建，最后消费者释放才结束活动引用（B02/B09）

#### Scenario: Discovery has only yielded a prefix

- **WHEN** ScopeCursor 尚未开始或仅返回部分候选，或者 standalone 首项前已取消
- **THEN** 前缀不报范围完整；已取消的游标不返回新候选，只有实际穷尽且不存在未知子树时才报告 CompleteWithinSchema（B01/B06）

#### Scenario: Source text and its encoded record each fit but their total does not

- **WHEN** 正文构造和 JSONL 交付各自小于 output_bytes 上限，但累计工作已超过同一操作额度
- **THEN** 第二部分不得通过独立 allowance 再花一次额度；真实停止进入总账，不能发布最终 Complete，Final 自身许可也不得在汇总取样后漏计（B04/B08）
