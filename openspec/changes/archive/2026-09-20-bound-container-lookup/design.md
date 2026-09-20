## Context

动机见 [proposal](proposal.md)。源码证据为 `providers.rs::zip_candidates/tree_candidates`、`artifact.rs::read_nested_entry_with_accounting` 和 `facts_cache.rs`。当前两条路径重复：查名先枚举目录，读取 nested entry 又重放祖先链并物化父 JAR；只保存 `ArtifactTreeReport` 仍然留下第二条路径。现有 `FactsCache` 在字节物化及完整性检查之后跳过 CP/Header parse，且只按 entry 数限制容量。

benchmark 固定在 `cd6f2f0`。分析期间工作树出现 `fd0aae8` 的 R8/R9 修正；实现前重新固定 SHA 和直接路径基线，不把旧文本 fingerprint 直接要求为新实现输出。Atlas 的 `tree_candidates` 文件级定位已完成；其他目录级 Focus 尚未完成，相关结论来自直接源码核对，不声称获得完整调用图。

## Goals / Non-Goals

**Goals:** 把成本从“每个名字重走整树”改为“访问实际搜索的 container 与祖先链；完整 facts 保留期间重复查名只做定位”。先修直接路径与操作内复用，再量化跨请求保留的增量收益；约束驻留内存，用同工作负载的端到端证据决定是否继续投资。

**Non-Goals:** 不建预热全树的强制 session，不缓存 resolution verdict、IR 或源码；不改 root 选择语义，不以扩大默认预算换取成功，不建立新线程池/数据库。查询 class 共享、方法定位表、批量 API、细粒度分页、通用 lazy parser、CP/Header 深复制改造、预加载/预取、single-flight、mmap 与输出序列化均不在本次实现范围。

## Decisions

### 1. reader 提供单 container 事实，resolver 只声明所需位置

从 nested replay 抽出一个共用的定向 container 访问路径：验证 snapshot/root；逐层验证父 entry ordinal、raw name、candidate 状态与派生 child identity；最后枚举指定 container 的完整中央目录。`tree_candidates` 不再调用整树枚举；`zip_candidates` 也走同一目录事实路径。显式整树枚举仍负责发现所有 descendants，它和单 container 查名有不同的覆盖范围。

中央目录可能必须线性读取，冷成本不会凭空消失。对目录建立 raw-name → entries 的多值查找表，entries 以中央目录 ordinal 稳定排序；使用现有物理身份，不按类名合并重复项。命中只借用/复制匹配项，禁止每次 clone/遍历整份目录。跨请求关闭时仍使用相同定向算法，只不保留 facts。

单次操作内部将已取得的权威目录、locator 和 backing 传递到 entry 读取阶段；查名、按物理身份读取与绑定复核共享必要的 container 事实。无需先生成公共树报告再反向消费，也不应在 cache 未开启时重新建立同一份仍在使用的事实。操作内引用随操作释放，跨请求保留才进入现有 store；两种生命周期共用事实生产路径，不增加公开 session/manager。活动数据继续受请求额度约束，容量拒绝不能迫使当前读取丢弃并重建已经取得的事实。

### 2. 复用目录及其经验证 backing，避免只缓存 listing

扩展 reader 现有 `FactsCache`，不另建全局缓存管理器；内部增加 container 产品，key 为 snapshot 内容身份、完整 ContainerOrigin、目录/验证 schema。RuntimeProfile、loader 顺序和 prefix 不属于物理目录事实，不进入该 key；它们仍由 resolver 每次应用。启用仅允许保留真实请求构造的事实，不触发后台预热、自动全树索引或方法预分析。

STORED nested container 可引用已验证 backing 的区间，DEFLATED nested container 保留经过完整 CRC/size 检查的不可变 backing。目录记录的 locator 只在所属 backing 下有效。读取 class 时用该权威记录检查传入 metadata，再读取所选 class；不信任调用方反序列化的 report，不省略 class entry 自身的完整性检查。物理 origin 在每次结果中照实绑定。

采用完整 container 粒度发布；损坏、截断、取消和预算终止不写完整目录项，也不缓存“未找到名字”的 resolver 结论。已完成 ancestor 与某个失败 child 分开管理；失败 child 不能污染完整 sibling 或伪造空目录。首次版本不保存跨请求 Partial facts。

### 3. 同一 cache handle 统一限制驻留，不用 request Budget 承担生命周期

沿用显式 handle 注入，不把可复用状态隐式塞进不可变 snapshot，也不把开关默认为 on。扩展现有容量配置为 entry 上限与 retained-byte 上限；权重覆盖 backing、目录、名称表和现有 CP/Header payload，避免容器与 Header 各自合规而累计无界。API 若需调整直接迁移仓库调用点，不维护平行旧接口。

首版沿用“满则拒绝插入”的策略，不引入 LRU、驱逐队列或额外依赖。超过单项或剩余容量时释放待插入状态，当前请求继续使用已构造事实，后续请求走直接路径；不能为了 cache admission 失败重做本次扫描。共享 backing 去重计权，所有被 store 持有的 strong reference 必须计入；cache 清空/drop 后释放保留引用。活动请求的临时字节继续受既有预算约束，retained weight 是驻留代理而非 RSS 精确值。

### 4. 区分实际工作、复用证据和当前限制

命中先 poll，再检查当前请求的 `nested_depth` 等对象结构限制；高深度预热不能让低深度请求绕过边界。这里须与累计工作额度区分：`archive_entries/read_bytes/entry_bytes` 记录本次实际解析、读入与展开，复用免去工作后允许在较低累计额度内完成，不回放旧请求计费，也不能把复用宣称为本次 I/O。若校验策略另有对象大小限制，则必须按当前策略检查已验证 metadata，不能把这类限制当作“省去了工作”。当前查询返回的结果数量与输出字节继续计费；内部候选定位不先生成一个面向用户的全树报告。

报告复用命中、目录解析次数、nested 物化次数与字节、retained weight、容量拒绝。跨 direct/cold/warm 比较使用语义 fingerprint：cache stats 与实际 usage 不参与，定义、顺序、诊断、选择依据、source map 和逻辑 coverage 参与。它不能替代主规格的完整报告确定性检查：重复运行相同输入/profile/limits 和相同 cache 初始状态时，仍只剔除 elapsed_millis，保留其它字段及预算计数；cold 对 cold、相同预热步骤后的 warm 对 warm 分别核对。两种 fingerprint 使用不同名称并同时保存。

两个足额完整执行的语义结果必须相同；同一紧预算下热路径可能 Complete、冷路径 Partial，只要真实计费与停止语义成立。不得以“冷/热完全相等”为由假计费，也不得借命中重置取消或预算。额外用已校验 DEFLATED backing 验证：目录/backing 命中不虚增 archive_entries/read_bytes/entry_bytes；仍需进行的 class 读取不能免单；到期 elapsed、已取消或已经终止的 Budget 均不得因命中恢复运行。

### 5. 容器子项的分阶段证据

本 change 是 [性能专项](../optimize-demand-workloads/proposal.md) 的 O1。工作负载口径、阿姆达尔判断、仪表校准、统计方法、完整/语义 fingerprint 与停止投资原则以总专项为准；本项只实施容器访问与复用，不承担其它方案的调查或实现。

扩展现有 P5 harness，比较 B（固定原始路径）→ D（定向直接路径及操作内复用，跨请求保留关闭）→ C/W（同一 D 上的空/热 store）→ F（容量不足）。B/D 使用记录精确 revision 的基线/候选产物，固定语义与构建条件；C/W/F 使用同一候选。容器实验保持 CP/Header 产品策略一致，无法隔离时只报告组合收益，不把它全部归给 container。

以总专项 W1/W2/W5 为主，另测同方法重复控制组。分别记录准备/填充成本、每请求与全序列时长、目录解析及条目访问、nested 物化次数/字节、实际 class 读取和驻留权重；避免只测相同方法的最佳情况。小/大 flat JAR、STORED/DEFLATED nested WAR、多个 sibling、重复项与超容量 fixture 均可再生成，CI 不依赖 `/tmp/jarde-bench`。

确定性验收是未搜索 sibling 展开为零、同次操作中仍持有的目录/backing 不重建、保留期间后续查名不遍历整目录或重复解压父容器，以及足额完整结果语义一致。计数改善与时间收益分别报告，按总专项固定协议记录分布与不确定性，不预设亚毫秒/倍数。即使时间收益未证实，也不得省略已承诺的行为、资源和回退验证；默认保持关闭，不自动加入另一种加速机制。

### 6. 复用与层次

复用已准入 rawzip/flate2 的目录、locator、解压与校验；复用现有 `Arc`、`BTreeMap`、锁与 FactsCache 注入。所缺的是保存已验证物理事实及权重核算，不是 ZIP parser 或复杂缓存算法，无需新增库；现有依赖的版本、许可和维护判断沿用 `openspec/dependencies.md`。不持锁解析、解压或调用上层，竞争插入重新核算容量；本 change 不承诺并发 single-flight。

目录 parse 不证明 class dialect 合法；raw-name 候选不证明 runtime 选择正确；cache hit 不证明 JVM verification 或 Java 恢复正确。这些阶段沿用各自契约。

## Risks / Trade-offs

- nested backing 增大常驻内存 → 显式总权重、容量拒绝、drop 验证；默认关闭，不承诺 RSS 比基线更低。
- 缓存旧 execution/usage 导致计费失真 → 只缓存 immutable facts，当前请求重新构造报告与停止状态。
- 只测相同方法掩盖目录复杂度 → 同 snapshot 多方法、逐步增加无关 sibling/entry 的确定性计数对照。
- 定向查名不再观察无关坏 sibling → 这是收窄范围的预期行为；显式全树枚举仍报告其损坏，局部查询不得宣称整树完整。
- 热路径数字掩盖首次访问、填充和内存成本 → W1 与序列总成本单列；超过容量时测量直接退化。
- 阶段计时或并发工作树改动污染结论 → 计时在 harness 内校准，固定 revision/构建/语义 fingerprint，不能混合当前其它 change 的行为差异。

## Migration Plan

先固定 B 与工作负载分解；交付 D 并单独验证；再接入 C/W/F 做有界复用实验；最后重测、记录收益和后续停止/另案决定。缓存新增路径可通过不附加 handle 回退；定向访问是新的直接实现，使用受控旧/新对照证明选择语义。每个阶段单独提交；本 change 不修改其它 changes 的实现状态，不自动打开 cache、不自动进入候选优化。行为门禁与测量完成分别记录：证据不足可以得出“不发布耗时收益”，不能写成“已达到性能目标”。
