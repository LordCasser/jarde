## Context

动机和范围见 [proposal](proposal.md)。本设计采用[最终架构](../../../JVM_Rust_Engine_Final_Architecture.md) §15–16 的同步可取消 API、显式有界 worker 和有序背压；其中已经说明预算中止的并行子集可能受调度影响。当前代码仍没有批量调度器，不能把架构图或本计划写成已实现能力。

2026-09-21 核对的代码事实（实施 G0 重新固定 SHA/dirty diff，不能用本工作树代替固定构建）：

| 位置 | 事实 | 本次落点 |
| --- | --- | --- |
| `src/facade.rs::Engine`、`artifact.rs::ArtifactSnapshot` | Engine 无状态；snapshot backing 为 `Arc<[u8]>` | 整个导出共享一次打开的输入 |
| `budget.rs::Budget`、`facts_cache.rs::FactsCache` | 每请求可附同一 cache handle；普通构造不附；store 有容量上限 | 导出持有句柄，worker 不各建一个 store |
| `facts_cache.rs::{FactsKey::of,take}` | 按 bytes 重哈希；命中在锁内 clone owned payload | 可信内容身份交接和不可变共享 payload |
| `classfile.rs::method_code_facts` | 重建类 reader 并完整重扫 methods | 类任务持有 parser/结构和多值方法定位 |
| `jarde-jvm/engine.rs::read_driver_method`、`facade.rs::read_named_callees` | driver、callee 使用独立 HeaderClosure 读取 | 在同一类准备上消费，但保留原需求与绑定检查 |
| `Engine::class_view` | 已共享 bytes 和成员列举；不是 Java 全类恢复 | 复用经验，不把 class_view 包装成已经存在的批量恢复 |
| `jarde-cli/task.rs::Session::deliver` | 一次文档完整序列化后写出 | 新 export 使用流式适配；旧命令保持原行为 |

用户给出的 8807fa5 数据作为调查线索：bcprov 40.2 s / 14,495 = 2.773 ms/体，S2-009 296.5 s / 53,516 = 5.540 ms/体；它们不同于请求 p50 的 2.65/5.10 ms。store-on 的 0.19/0.10 ms 是 p50，未提供同口径冷导出总账。分析步骤数相同只能说明那个计数没变。本设计不把这些数字写成基线验收结果或加速承诺。

## Goals / Non-Goals

**Goals:** 单次调用覆盖显式范围的全部方法声明；同类准备摊销；类间可并行；资源随 worker 窗口和声明容量受约束；方法产物持续交付；正常与中止均可对账；单类有可读的源码快捷视图。

**Non-Goals:** 保持 [proposal](proposal.md) 的能力边界。尤其不建立另一套 decompiler、公共通用 Session、全局 executor、全 IR 缓存或完整可编译工程装配器（用户追加的单类 `class-source` 视图只装配既有逐方法恢复文本，不引入新的解码/恢复实现，也不解析 imports 或 resources）；不以放大线程栈解决递归缺陷。跨调用的共享计算订阅/取消不进入首版。

## Decisions

### 1. 一个库操作，两个执行配置

根 `jarde` 增加 `Engine::recover_all`，接收 snapshots、`BulkRecoveryRequest`、调用方的 `&mut Budget` 和类型化 sink，返回 `BulkRecoveryReport`。名字为拟定 API，实施可按现有命名统一；以下数据及语义固定：

| 输入 | 含义 |
| --- | --- |
| physical view / scope | 一个主 snapshot 的显式物理导出范围；其它 snapshots 只能是声明依赖 |
| environment policy / profile | 沿用现有环境构造与校验，不自动推断 WAR/Boot layout；导出数量不证明运行时可加载 |
| workers | 明确的正整数；1 在调用线程串行，N 创建本次作用域内至多 N 个 worker |
| method_limits | 从每个方法开始分析到恢复完成的局部 Limits；不包含排队时间及已共享的准备成本 |
| max_class_bytes | 单个准备 class 的字节上限，超过即记录该类拒绝，不能无限等待 |
| max_result_weight | 单项结果的固定保留权重上限，不随 worker 数改变 |
| max_buffered_result_weight | 整个结果窗口的保留权重上限，至少能容纳一个 max_result_weight；权重模型见决策 5 |
| facts_capacity | 现有 entries/retained_bytes 限制；零容量关闭跨消费保留，操作正在持有的事实仍共享 |
| total limits | 由传入 Budget 提供；已经打开输入所付额度继续生效，禁止重新开始总账 |

库要求显式数值。CLI 提供有界默认值并在 Header 发布全部有效配置；`--jobs auto` 使用 `available_parallelism()`（读取失败取 1），再按可容纳的窗口数确定实际值。0/非法值拒绝；不能满足一个 worker 的配置在派发前拒绝。单方法 `recover` 和 Query 不获得隐式 worker。

sink 只由协调线程调用，不要求调用方实现线程安全回调。流以 Header 开始，每类依次 `ClassPrepared → Method* → ClassEnd`，允许夹入有物理位置的 `Diagnostic`，最后为 `Final`。事件复用现有身份/结果类型；不为每个事件建立独立报告框架。ClassPrepared 只交接身份和共享读取证据，不序列化完整 CP 或整份成员结果列表；所有普通数据事件都受固定单项容量限制。sink 确认接收后才推进 delivery cursor。返回的 summary 只保留计数、边界和有界活动窗口，不收集全包报告。

### 2. 类是调度单位，方法是交付单位

```mermaid
flowchart LR
  S[不可变 snapshots] --> D[有界物理遍历]
  D --> C[共享完整 container facts]
  C --> Q[至多 W 个类的执行窗口]
  Q --> W[W 个 scoped workers]
  W --> P[每类一次准备]
  P --> M[同类方法逐个分析与恢复]
  M --> B[每活动类一个有界结果槽]
  B --> O[协调线程按物理及声明顺序交付]
  O --> K[类型化 sink / CLI JSONL]
```

物理顺序沿用 snapshot/容器 entry ordinal 的确定性遍历：父容器按 ordinal 行走，child 在其 entry 处深度优先，成员按 class 声明 ordinal。worker 完成时间、展示类名和路径字符串排序都不改变这个顺序。完整 raw name/descriptor 与物理 origin 仍在记录中，ordinal 只作为本次序号，不成为跨 snapshot 身份。

发现使用已有完整目录和 locator，增加内部可停止 visitor/游标，使协调器最多派发一个窗口；不先构造全包类列表、所有类 Header 或所有 MethodIr。遍历栈受 nested_depth 约束。普通公开枚举 API 继续可收集其原报告，这个变化不改 Query 分页协议。

任务携带从同一已验证 container/backing 派生的物理 locator 和强引用，直接读取该 class，不再次从 roots 搜索已确定的目标。读取的 CRC/size、metadata 和 driver 的环境/物理绑定检查继续进行；不能用 prepared 输入绕过 loader/profile/版本校验。依赖仍只按原规则扩展，不进入导出目标集合。一次性扫描可以跨过 store 容量；正在遍历或被任务引用的 container 不因 cache 准入失败重物化，最后一个使用者释放后才允许以后重建。

选择类粒度是为了摊销 class 准备和避免同类竞争。方法粒度独立排队会让 CP/定位重复；全包共享可变 IR 会扩大状态范围。大类造成尾部拖延是首版接受的限制，B10 测出尾部占比后才能另立类内切分，不预先加入 work stealing/通用调度层。

### 3. Prepared class 是可信读取的生命周期，不是新缓存层

reader 提供不允许外部伪造的 prepared class 视图，包含可信 bytes/content identity、完整 CP/结构、成员表及 raw name/descriptor → 全部成员 ordinal 的定位。直接方法入口和批量入口最终调用同一个方法解码实现；只改变已有事实从哪里交接。

worker 先持有 backing，随后构造借用它的 parser 视图；解析器及方法引用不跨越 backing 生命周期，不把自引用结构放进 `Arc`，不要求第三方 parser 类型跨线程共享。可独立拥有的完整 CP/Header payload 才使用 `Arc`。已有读取返回的可信 digest 随事实传递，不接受调用方裸 digest 作为验证替代；不为每个方法重哈希同一 backing。

完整成员扫描时建立多值 locator，保留重复名称、descriptor、多个 Code 属性的错误和 Code span 校验。只用 Header shells 定位，不提前解码所有 Body。成员表不完整时禁止发布全表唯一性/不存在结论，沿用已有拒绝/前缀行为，不顺便扩展损坏隔离能力。

`jarde-jvm` 增加 prepared 输入的内部入口，复用 request/stage/environment 校验与原 pass 流水线。一次已准备类可提供 driver 和同类 accessor callee 的证据。持有的 MethodCodeFacts 可复用；完整 MethodIr/AST 不长期缓存。只为被实际要求的 callee 留有界事实，若因容量释放而日后再需要，允许重新解码并单独计费，不能承诺“全类所有 Body 无条件一次解码”而偷偷保留全部 IR。

共享 Header 读取发布于 `ClassPrepared`，方法通过准备序号引用同一物理证据；本方法真实新增的依赖读取继续进入自身记录。跨串行/批量语义对照先展开这个引用，再比较同一需求/来源，不复制 N 份 usage，也不伪造 N 次读取。无绑定、重复声明、未支持 dialect 或输出级别的原结果照常发布。parse、dialect validation、runtime selection、verification 和 source recovery 仍为独立事实。

### 4. 一份总账，方法有局部限制

不使用“每个 worker 一份完整总 Budget”，也不在整次方法恢复外面放一把锁。`jarde-reader` 的 Budget 复用现有 charge/poll 通路，增加操作作用域的共享总账；单请求仍可使用无共享协调的直接通路。

全局累计维度通过短临界区或等价原子准入执行：先校验局部额度/溢出、截止时间和取消，再原子取得全局额度，最后记录本地归属并执行工作。未取得额度不开始动作；锁不跨解析、解压、IR、恢复或 sink。某动作已取得许可后发生取消，它可能完成该动作，仍必须记账，在下一检查点停止。首版不做大块额度预借；避免预借余量制造假超限或隐藏实际费用。总账同步若成为热点，B10 单独计时后优化，不先牺牲额度语义。

归属分为 discovery/preparation、methods、delivery，都是实际工作分类，不新增预算维度：

`final cumulative usage = entry usage + discovery/preparation + sum(method usage) + delivery`

nested/dependency depth 分别取已接受最大值；总 elapsed 从父 Budget 的开始时刻算一次；方法局部 elapsed 从进入方法分析算到恢复结果形成，不把输出排队算作算法时间，总 deadline 仍覆盖排队/等待/输出。方法执行完毕后清理 IR，交付编码计入 delivery。cache miss 的准备由实际构造者计费，共享命中不重放计数。下游新的结构限制始终检查，不因引用复用放行。

全局终止原因只发布一个确定已发生的 primary stop，记录其实际归属/位置；后续观察不覆盖它，局部诊断仍各自保留。局部方法损坏、局部额度耗尽不取消其它方法；全局额度耗尽/取消关闭派发并通知所有活动 worker。API 返回前总账吸收它们的全部已执行工作；已执行但未交付也不能退费。所谓不退费不包括“只检查/保留但未执行”的容量许可，两者必须区分。

### 5. 窗口、内存和背压

首版最多 W 个活动类，每个活动类最多一个待交付方法结果，worker 必须等到该槽释放后才计算下一个方法。令 `W = min(requested_workers, floor(max_buffered_result_weight / max_result_weight))`，每个槽预先取得同样的 max_result_weight 额度；增加 jobs 不会缩小单项能力，也不会让最早类与后续类争最后一份输出空间。超大结果在形成/交接的检查边界返回带原因的局部停止，不能卡在“永远放不下”的等待中，也不截断 Java 文本后标成功。必要的停止控制事件有固定的小槽，与普通产物额度分离并计入窗口配置；控制槽只携带既有工作序号、停止码和计数/边界，不复制无界正文或诊断消息。

物理遍历不阻塞在向慢 worker 发送新类上：满窗口时先消费最早槽，空位出现再扩展发现。协调器持有最早类时可立即交付其方法，不必等它整类完成。即使其它 W-1 个槽全满，最早类仍有自己的槽；这给出无容量环路的进展条件。选择更小的窗口可能降低吞吐，但不会增加内存。

内存边界分别是：snapshot 输入限制；现有 FactsCapacity；至多 W 份 `max_class_bytes` 范围内的类准备；至多 W 份受 method_limits 限制的当前 IR；窗口结果权重；适配器一个有界编码/写出缓冲。容量权重统计拥有的 Vec/String capacity 及直接结构成本，Arc backing 在同一归属内去重；这不是进程 RSS 的硬上界。正在构造的结果由 method_limits 约束，进入等待窗口后再受结果权重约束，不能将后者宣传成所有构造分配的上限。真实 RSS、线程栈和 allocator 开销由基准单独测量。

不先引入通用内存分配器或跨所有 IR 容器的字节配额框架。若现有单方法限制无法给出可接受的工作集，作为阻塞默认启用的实测结果记录，单独修正该分配热点，不能把 retained weight 假装成 RSS。

### 6. 停止、交付与清理

| 事件 | 本次操作行为 | 结果 |
| --- | --- | --- |
| 某方法有正常 fallback / explanation_only，但原 execution 完整 | 交付原结果并继续 | 不升级 content/quality；可能范围 Complete，含语句覆盖仍不足 |
| 某方法损坏、Failed/Partial 或局部额度停止 | 交付其原状态并继续其它方法 | 最终 aggregate 至少 Partial，遍历到尾单独记录 |
| 总额度/全局取消 | 停止派发，通知全部 worker | Partial/Cancelled；保留真实执行和交付覆盖 |
| 某类太大/结构无法准备 | 类级拒绝，未读方法数未知，继续其它类 | 非 Complete，不能计完整方法分母 |
| worker 创建失败、可观察的 worker panic、sink I/O 失败 | 关闭所有邮箱/通道，停止并 join | Failed；不自动重试、不开第二次串行运行 |
| sink 主动不再消费 | 全局取消，不继续计算后缀 | Cancelled 与已确认交付前缀 |

派发后的类状态为 `queued → preparing → running → done/locally_stopped`，交付状态另记成员 cursor。执行 cursor 可以领先交付 cursor。终止后不为填平空洞继续分析；可交付的已准备结果也不得绕过取消/输出限额。活动窗口留下至多 W 个执行前缀和已执行未交付边界，已交付历史由流本身记录，不在 summary 中保留全包结果。

所有等待必须观察关闭/取消。复用现有 CancellationToken；协调器用有截止时间的等待检查 token，观察到后关闭/通知邮箱，使容量等待者退出。不得在持有 cache/总账锁时等待 sink 或 join。callback 的执行发生在协调线程；宿主 sink 必须协作返回，库不强制中断无限阻塞的外部 I/O。

内部字段以既有 ExecutionReport 为主；aggregate 的优先级是基础设施 Failed → 全局 Cancelled → 全局预算或任一方法执行不完整导致 Partial → 全部完成 Complete，保留 primary reason 和各方法原状态。遍历完成、分析完成、含语句产出、编译/语义验证和交付完成互不代替。

### 7. CLI 首版只增加一种流式导出形态

拟定命令：`jarde-cli export --input INPUT --scope SCOPE --policy POLICY --output OUTPUT.jsonl --jobs auto`，沿用现有 scope/policy 参数解析。输出是方法产物包，每个 Method 记录包含原恢复文本、source map 和结果平面；不是 `.java` 工程，也不复制 resources。不增加每方法子进程或每方法一个文件的 I/O 成本。

CLI 使用 `create_new` 创建目标，拒绝覆盖；编码经有界 writer/缓冲和现有 output_bytes 许可，超限不能无限分配。完整记录成功写入后才确认交付，I/O 部分写出的最后一行不计入 delivery。CLI sink 处理 Final 时完成最终 flush/close 后才确认该事件，库据此更新交付汇总；失败返回 Failed 并非零退出，即使文件中已可见 Final 行，也不以该行覆盖本次 I/O 失败。Final 确认后才可 exit 0；方法执行不完整为 4，输入/基础设施或 I/O 失败沿用 2，批量物理发现的单项歧义记录在项内并导致非完整，而不是把整包变成单目标选择的 3。

库始终返回可定位的 typed summary（输入验证前失败按既有 Result）；Final 流记录只在 sink 仍可写且额度允许时发送。不能为写 Final 绕过已用尽额度；若不能交付 Final，保留前缀并非零退出，消费者按“无 Final 不确认完成”处理。不承诺断电事务或恢复被截断的半行；断点续跑和多个文件的原子提交留作独立能力。

### 8. 确定性是明确的新并行契约

保留普通 API 与 bulk workers=1 的严格重复门禁，只去 elapsed_millis。bulk workers>1 的足额运行要求展开共享证据后的逐方法语义完全一致，按固定顺序发布；metadata/usage/cache 资源记录独立保存并核账。并行完整执行中资源归属可受准入顺序影响，不能用语义 fingerprint 声称完整原始报告字节确定。

紧预算/取消可能产生不同执行子集，设计主动承认这个边界，符合最终架构 §16；不通过强制静态切分总预算来制造表面确定却拒绝本可完成的方法。测试必须主动扰动调度，检查额度守恒、没有假 Complete、没有漏报已执行工作，以及语义诊断未被资源字段白名单隐藏。scope/identity/content/source map/规则/语义诊断不允许进入忽略列表。

这是新 bulk 模式的显式 contract delta；旧 warm/warm 串行字段对照不放宽。相关 [analysis-contracts](specs/analysis-contracts/spec.md) 与 [measured-execution](specs/measured-execution/spec.md) 同步描述适用范围，避免把输出排序当成并行停止集合确定性的证据。

### 9. 并发原语和依赖选择

首版选 Rust 标准库的 scoped threads、短时同步与有界消息；使用可返回创建错误的 `Builder::spawn_scoped`，不使用会隐式固定进程全局 pool 的入口。局部 mailbox 只承担“一个槽、关闭/唤醒、额度”三个本操作需要的动作，禁止发展成通用 executor。普通 worker 栈上必须通过深表达式回归，不能用 stack_size 提高掩盖 abort。

| 方案 | 复用能力及缺口 | 决定 |
| --- | --- | --- |
| Rust std | scoped 生命周期、创建错误、同步通道可用；关闭/背压与现有 token 需薄组合 | 采用，不新增第三方运行依赖，满足当前 MSRV |
| Rayon 专用 pool | 提供线程数配置及 work stealing；仍需总预算、按类顺序和输出背压 | 首版不引入；类内拆分若实测有价值再评估，不把并行集合 API 当完整调度契约 |
| async runtime | 不能自动加快当前 CPU 恢复；引入独立运行环境 | 不采用 |
| 进程池 | 增加输入打开、IPC 与共享事实交接成本 | 不作为全量主路径 |

第一方资料（2026-09-21 核对）：[`Builder::spawn_scoped`](https://doc.rust-lang.org/std/thread/struct.Builder.html#method.spawn_scoped) 提供 scoped 线程及创建错误；[`sync_channel`](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html) 提供有界消息；[Rayon ThreadPoolBuilder](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html) 为对比方案。Rust 标准库沿用工具链的 MIT/Apache-2.0 许可；Rayon 若后来准入，固定版本后重新核查 maintenance/MSRV/license 和 supply-chain 门禁，本文不预先批准新依赖。

### 10. 性能验证和交付判据

| 臂 | 变化 | 用途 |
| --- | --- | --- |
| A | 原同 snapshot 逐方法、无 store | 历史调用方式的可复跑锚点 |
| B | A + 共享 store，空 store 起跑 | 分离容器/CP retention 的收益 |
| C | bulk，workers=1，类准备共享 | 验证批量与 locator/ownership 的收益 |
| D | C，workers=2/4/6 | 只改变并发；共享同一输出包/schema |
| E | C/D，零容量、超容量、慢 sink、大类倾斜 | 容量与退化边界 |

这些臂只能按相邻变化归因，收益不相乘。按 class 一次使用而没有 IR/source 再访问的 sweep，不引入全方法结果 cache。user/system CPU、stage 互斥时间、worker overlap、总账同步等待、payload clone/hash、class/locator 构造、Body 重解码、目录/物化、排队与写出分别测量；仪表开关对照不把嵌套时间加成总时长。

主指标是冷进程开始到输出关闭的总时间，包含生成清单/准备、hash、解析、恢复、JSON 编码、write/flush/close；不额外要求 fsync，双方采用同一持久化口径。独立记录 OS page cache 初态，应用 cold 不冒充机器 cold。每配置至少十次交错重复，保存全部原始样本及预先确定的统计方法、退化/RSS 界限；方法 p50 和首次结果只是辅指标。

对 jadx 分别跑 `-j 1` 和明确 `-j N`，固定版本、JVM、相同 nested 输入集和 resources 策略。方法 join 使用完整物理 container/entry + name/descriptor，单列构造器、内联、失败、仅解释和无 Body。完整类源码与当前方法 JSONL 属于不同交付，单独呈现任务耗时；只有明确可比范围才能说接近/超过。每方法可编译/行为门禁仍由受控 fixture 独立提供，jadx 输出不是正确性 oracle。

验收分两层：B01–B09 是功能/资源交付；B10 是性能证据。足额产出语义保持是先决条件。目标默认策略是 CLI auto 加操作内有界保留，数值容量在任务 1.2 固定并由 B10 验证。达到全量导出功能不自动宣称超过 jadx；候选若没有超出噪声的整体收益，默认策略交付保持未完成，记录瓶颈继续修正。若决定改为仅显式启用，须先同步修订 proposal/spec/任务，不能实现时悄悄将 auto 变成恒定单线程。具体机器上的比例由真实数据填写，不在 spec 中许诺倍数。

### 11. 单类源码视图是装配，不是第二条恢复路径

用户追加需求：CLI 需要一个类似 jadx 的快捷功能，把一个类尽量恢复成可读源码。落点与边界：

| 项 | 决定 |
| --- | --- |
| 拥有者 | 装配在根 `jarde`（facade 新操作 `class_source`，逻辑放独立模块），逐方法 Java 文本仍由 `jarde-java` 既有恢复产出；CLI 只做薄适配 |
| 输入 | 一个类身份（现有 `ClassRef`：名字或物理定义）+ `EnvironmentRequest`；名字歧义沿用 `OperationOutcome` 候选分支，不猜 |
| 复用 | 类声明/字段/成员列举走现有 `class_view` 事实；每个成员体走既有单方法恢复链路（`analyze_method_ir` + `recover`）。不新增解码、成员扫描或第三份恢复实现 |
| 文本装配 | 类头（修饰符、名字、super/接口）+ 字段 + 逐方法签名与体；体文本按 Java 块重新缩进；描述符到 Java 拼写复用 `jarde-java` 既有能力 |
| 诚实性 | 无 Body（abstract/native）、未产出/拒绝、`ExplanationOnly`、带停止诊断分别用稳定标记出现在成员位置；成员不得被静默省略；单个成员失败不升级为顶层 Complete |
| 非目标 | 不装配可编译工程、不解析 imports、不复制 resources、不为每个成员重读类 Header 或重复列举成员表 |
| 确定性 | 同一输入/配置/预算形状下装配文本逐字节可重复；耗时与调度字段只进 usage，不进文本 |
| 与批量关系 | 首版两条交付形态并列：`export` 逐方法 JSONL 流（决策 7）与 `class-source` 单类文本。二者共用同一逐方法恢复结果类型；批量路径将来可按类复用同一装配函数，不预先引入类源码的并行装配 |

不带 `--jobs`：单类视图按需执行，不启动 worker，也不隐式触发批量发现（保持 A16/A17 的按需边界）。

## Risks / Trade-offs

- 大类占据最早位置 → 首版接受有序窗口的吞吐损失；用倾斜 fixture 和真实尾部阶段测量，不提前引入方法切片。
- 总账锁或 cache 复制抵消并行 → 类内借用/Arc，锁不跨计算，单独测等待；不能关闭计费换速度。
- 一次性扫描挤满 store → 顺容器/类消费、消费后释放；保留准入失败不撤销正在持有的事实，记录重建成本。
- worker 栈更小而暴露其它深度边界 → concat/deep-chain 原判据已由独立归档关闭；新增 worker 后仍在该真实入口做 debug/release 子进程回归，不以旧主线程或测试线程结果替代。
- 中止集合随调度变化 → 有序交付与真实执行覆盖分别发布；新模式修改契约，旧串行门禁保留。
- sink 不返回或外部 I/O 卡住 → 库只承诺内部等待可取消；CLI 用有界写出，记录协作取消边界。
- “更快”来自更少恢复或输出 → 固定方法清单、content/执行分类、受控编译执行和输出字节对账；不接受质量退化作为性能结果。

## Migration Plan

按 [tasks](tasks.md) 形成可独立 review 的提交：基线和契约测试 → prepared class / 共享 ownership → workers=1 的流式 bulk → 共享总账和多 worker → CLI → 真实 workload 与默认策略裁决。每步都沿用同一恢复实现。

实施时更新声称“当前没有 scheduler”的能力清点测试为实际生命周期/隔离检查，保留普通入口零隐式调度的门禁；不能仅删除旧测试。同步上游 reader 签名及调用者，不保留两套解码实现。并发状态出问题可选择 workers=1 或取消本次调用，但已经启动/中止的操作不得重置预算重跑。

父 [optimize-demand-workloads](../optimize-demand-workloads/tasks.md) 仍是调查专项；本 change 唯一拥有上述产品实现。三项恢复修正已归档且 T1–T4 原判据已关闭；实施前重读[当前完成复核](../../completion-review.md)、主 spec 及 T5/CLI 文档预算边界，避免覆盖其它 agent 的改动或误宣称全部转换正确。本次只交付设计、delta specs 和未勾选任务。


## 2026-09-21 implementation review 与修复顺序

候选 `e06fe14` 已实现主体链路，但尚未满足本设计的活动引用、串行窗口和共同总账约束。独立证据见 [verification §7–8](verification.md#7-独立-reviewe06fe142026-09-21)。不能通过降低本设计要求使实现变成“完成”。

1. **先修生命周期与预算。** 串行完成一个类时立即退役对应 slot（或根本不进入 worker 邮箱），保留同一计数/交付语义；正常完成不等待已经不存在的任务。以真实待处理槽数验证 O(W)，不以配置值 `active_classes` 冒充观测值。ScopeCursor 只有实际到尾且无未知子树才能报完整，首次 standalone 产出也先检查取消。Final 的记录许可与 usage 取样先后必须让流和返回值采用同一计费边界，编码/交付共用操作总账。
2. **再把操作内复用做进 crate。** 游标到任务交接现有可信容器事实/强引用和 locator；worker 的类读取及 prepared loader 绑定查询都消费仍持有的事实，不从 origin 重建目录；保留 loader 顺序、候选歧义及物理身份校验。retention 只决定最后消费者释放后的保留，不决定本次已取得事实能否继续用。零容量、超容量和 clear 都走同样的活动所有权链；不以提高 CLI cache 容量代替。`max_class_bytes` 在 class 物化前准入，库级方法局部限制与操作总量分列，CLI 不复制整包额度给单方法。
3. **按原权重契约完成窗口验证。** 当前 `result_weight` 的固定 64 字节/记录代理未计拥有的可变容量和未序列化 facts；它可作代理指标，不能作为原设计所要求的容量实现。保持归属去重和固定单项 ceiling，不把代理高水位或 RSS 当另一项已经通过。
4. **最后重建性能候选。** 修复后的 1/N 完整语义、JDK 执行对照、实际 worker 深链与资源门禁通过后再冻结。先把同一核心结果的丢弃 sink、JSON 编码到计数 sink、编码加写文件三臂分开；并行同时记录 CPU、锁/等待、第一条结果与类分布。三臂差值只是受控对照，流水线有重叠时不能直接当可相加阶段耗时。历史 3 次/2 artifact 表不能确定串行输出是唯一瓶颈。

bulk 是宿主显式要求全量时的一种调度，MCP 的普通导航继续由声明/成员/指定方法入口服务。把 prepared 复用扩到单点和选定方法集合、输出按需投影、查询续扫等安排在父专项，不混入本次缺陷修复；见 [MCP 优先级](../optimize-demand-workloads/design.md#15-mcp-宿主的按需边界与下一轮顺序)。
