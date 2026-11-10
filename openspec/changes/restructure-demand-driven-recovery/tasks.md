> **当前执行（2026-10-26）**：S1已按D12有限验收，S2a已按A2.5有限域验收，主线33/65；S3a的4.1/4.2/4.5/4.6组件完成，4.8阶段门待验；S3a及全部架构尚未验收。见[S2a收据](results/s2a-variable-legalization-v1/acceptance.json)和[当前检查点](results/s2a-variable-legalization-v1/reference-constraints-checkpoint.md)，历史收据保持当时口径。

> 历史 CG 局部验收（规划修订前口径）：[instance capture需求与持有期（2026-10-26，CG）](results/s1-instance-capture-demand-policy-v1/README.md)。当前489/0/0、83旧控制及2469全JSON/408核心保持；实测proof后2 IR/2 ClassFacts归0，恢复47，2.9/2.10开放，17/59，S2未进入。



> 历史进度说明（规划修订前）：2026-10-20 S0 的 9 项任务已按声明域验收，结果与失败原流见 [冻结报告](results/s0-observed-v3/README.md)。用户已授权继续 S1，已完成 2.1–2.8，见 [职责/evidence 验收](results/s1-core-v1/README.md)、[需求复用验收](results/s1-demand-v1/README.md)与[状态/停止验收](results/s1-demand-state-v1/README.md)及[源码目标验收](results/s1-source-target-v1/README.md)及[环境/loader 验收](results/s1-environment-v1/README.md)及[语义绑定验收](results/s1-semantics-v1/README.md)及[输入来源验收](results/s1-trust-v1/README.md)；合计 17/59。2.9–2.10 仍未完成，S1 整体尚未验收，S2–S6 未启动。验收 ID、审计映射与独立债务 E1–E6 见 [design.md](design.md)。测量基线通过不代表产品合同全部通过。

**语法问题处理时机（2026-10-26）：** 按用户要求，具体语法恢复问题先登记到[design D13](design.md#d13-语法恢复问题先记录架构变更后复核)，等当前架构变更完成、职责与共享接口稳定后再复现和修复，避免先补旧实现后再迁移。已记录字段声明alias/正文引用不一致，以及变量合法化和求值顺序的待验证风险；后两项不算新增已确认缺陷。S2a/S3a机制任务和原验收照常，当前不追加语法修补、任务复选框或退出前置。身份/origin与2.10停止传播问题不并入语法暂存；进度仍17/65。

## 1. S0 — 机制基线与验收输入（已验收）

前置：用户明确恢复 S0 后，核对当前源码、handoff、主规格与相关 change。产物：可复现输入、失败分布、机制矩阵和验收门。阶段内依赖写在各任务中；本阶段已验收，冻结原记录保持不变。

- [x] 1.1 冻结引擎 SHA/二进制、授权 JAR/WAR 原件、nested/MR/duplicate-origin、编译器/flags、完整 runtime/loader/provider 声明、有效源码配置与预算；验证 **A0.1**：摘要可校验，缺失项明确，版本/环境配置可重建，当前无独立 SourceTarget 的限制如实记录，未复现假设不标为已复现。
- [x] 1.2 依赖 1.1，建立物理 class/method、Code、runtime 可绑定、实际尝试与完整类编译的分母及失败分布；验证 **A0.1**：NoBody、unbound/shadowed、输入失败、停止分列，每条记录有物理身份和原始输出，首个/后续阻塞及重叠诊断不冒充根因分区，类抽样不外推全量。
- [x] 1.3 依赖 1.1–1.2，冻结 S2–S5 双问题族矩阵、拒绝负例、多机制组合与按来源/lineage 隔离的 holdout；验证 **A0.2**：编译/debug/类型/消费者/结构轴及适用域明确，同源改名/常量/flags 变体不分置为独立保留集，被用于调规则的样本标记污染。
- [x] 1.4 依赖 1.3，复用完整类编译及隔离执行流程对照原 class、Jarde 和固定 JADX；验证 **A0.2**：生成源码原样编译，无 stub/手改，运行 classpath 不含原始目标后备，返回/效果顺序/异常及适用的 cause/suppressed 实际核对，JADX 分歧单列，无 oracle 不计行为一致。
- [x] 1.8 依赖 1.1，补齐 reader/JVM/Java 的 test-support 观测；验证 **A0.3**：在实际 read、IR 构建和拥有对象的构造/clone/drop 位置计数，分开流读取字节、逻辑计费与 OS I/O；facts 按共享 payload、AST 按节点、候选按已声明对象类型计实际 live/peak，不将对象数声称为堆字节；normal build 不保留观测状态，不改变恢复判断、身份、序列化和预算。
- [x] 1.9 依赖 1.8，以聚焦生命周期测试及观测开/关的相同输入对照确认端口；验证 **A0.3**：共享 Arc 不重复计 payload、深 clone 计新对象、drop 后回落，热身后只在静止点 reset peaks，多 worker 原子峰值可读；充分预算下源码/身份/核心状态相同，停止路径逐项保存，端口不持有产品。
- [x] 1.5 依赖 1.1、1.8–1.9，先声明采样及阈值公式，再完成单方法、类、重复查询、bulk 各 10 次交错资源基线；验证 **A0.3**：保存噪声、cache 禁用/冷/热/容量与 evidence/并发配置，分列实际读取、Body/IR、逻辑计费、时间、RSS、cache/live handles/候选与输出保留；中断批次不补写成功、不混配凑次数。
- [x] 1.6 依赖 1.1 和 1.5 的测量协议，分别收集 token 取消、budget/elapsed stop、sink stop、慢 sink 和 worker panic 的现有行为；验证 **A0.3**：取消置位到返回的延迟独立保存，默认栈与硬资源停止条件明确，Prepared/Refused 两类前缀按协议核对，无法观测和既有失败如实列出，不在 S0 修产品。
- [x] 1.7 依赖 1.1–1.6、1.8–1.9，汇总 S0 验收与审计/旧 change/独立债务承接；验证 **A0.1–A0.3**：每阶段输入、分母、阈值和未覆盖域可引用，记录 E1–E6 条件前置，历史任务不批量改状态，部分运行与缺项不能宣布冻结完成。

## 2. S1 — 恢复职责、按需事实、身份与源码目标

前置：S0 已验收且获准进入 S1。已完成的 2.1–2.8 与收据原样保留；剩余仅按 2.9–2.14 / D12 收口。纯职责整理归独立 E7，不再作为 S2a 前置；S1 未实际退出前不实施 S2a。

- [x] 2.1 梳理并迁移 facade 的 Java 源码证明与拼写到 Java 层，保留门面事实供给和停止编排；验证 **A1.1**：职责与真实调用逐项对应，crate 方向不变，无新增 presentation crate/Session，纯迁移不扩大接受域，旧有效产物与 origins 保留。
- [x] 2.2 依赖 2.1，收拢核心结果与按需候选，分开词法/投影语义配置和可选 evidence；验证 **A1.1**：等价上下文的入口决定一致，类级合法展示差异有投影来源，interface-super 现有能力保留，可选 evidence 不改正文，未请求候选不保留整套 AST。消费 [S0 SourceMap 耦合反例](results/s0-observed-v3/evidence-selection/analysis.md)：essential/source_map-only/all 的必要类族决定、完整类编译和受控行为一致；覆盖有/无 projected emission 和真实缺 origins 的拒绝，保留 all 当前有效成功，不能靠强制 all 或删除类族能力通过。
2.1–2.2 收据：[职责与验收报告](results/s1-core-v1/README.md)、[983 项回归及 136×3 比较](results/s1-core-v1/acceptance.json)。核心证明在同次 emission 生成，公开 evidence 单独发布；真实缺 origin 继续拒绝。本检查点未单独宣告完整 A1.1；2.3 的补充验收见下方。资源采集 480/480 返回，但同期时间门 2/24、冻结 S0 时间门 5/24 超门，RSS 均通过；[资源问题](results/s1-core-v1/resources.md)由 2.10 承接，不更改阈值。

- [x] 2.3 依赖 2.1–2.2，复用 HeaderClosure/ReadReason/PreparedClassRead 形成有界需求与请求内去重；验证 **A1.1**：类型需求零依赖 Body、两候选共用 helper 只解码一次、无候选零额外 Body，cache 关闭或容量不足仍能复用 live facts，不合并不同 origins。
2.3 收据：[按需事实复用](results/s1-demand-v1/README.md)、[334 项回归及恢复对照](results/s1-demand-v1/acceptance.json)。请求内完整 Body 复用独立于 ArtifactCache；普通 owning-read 与 Prepared 路径均验证。2.4 的状态/循环/全局停止合同随后按下一项验收；2.10 全阶段资源门仍未通过。

- [x] 2.4 依赖 2.3，统一需求结果和循环/停止传播；验证 **A1.3**：Missing/Ambiguous/Unknown/Stop 与进行中需求分开，平台契约不覆盖输入定义，局部拒绝不污染独立候选，全局停止后零新 Body，incomplete 不写成负缓存。
2.4 收据：[状态与停止机制](results/s1-demand-state-v1/README.md)、[1140 项回归/136×3 恢复对照](results/s1-demand-state-v1/acceptance.json)。两个实际反例转绿；类级阶段与多候选重恢复都在全局停止后截断，局部损坏仍允许独立需求。原始失败、测试输入/成本假设修正及源码后继分别封存。后续验收按下列收据推进；2.10 阶段门仍未通过，S2 不进入。

- [x] 2.5 依赖 2.2，将 SourceTarget 与 RuntimeProfile 分离，迁移恢复请求/报告、绑定及薄 CLI；验证 **A1.2**：目标 7/8 不改变 MR/共同符号解析，目标 7 lambda 边界及未注册目标拒绝，纯 IR/物理解析不因源码目标失效，库/CLI 一致。
2.5 收据：[源码目标与运行环境分离](results/s1-source-target-v1/README.md)、[28 目标验收](results/s1-source-target-v1/acceptance.json)。SourceTarget 仅注册 7/8；运行时/MR 选择、平台证明和纯 IR 保持独立，方法/类/类族/批量及 CLI 传递一致，binding schema 2 区分源码配置与环境。1145 passed / 0 failed / 4 既有 ignored；136×3 正文不变，完整报告经明确的字段迁移后 136/136 等价。源码 7 的受控完整类用 JDK 8 生成 major 51 并在 JVM 8/23 对照通过。后继 2.6–2.8 验收见下方；2.9–2.10 未验收，S2 不进入。

- [x] 2.6 依赖 2.4，复现并补全全部参与 domain/provider 的环境身份；验证 **A1.4**：同 content/caller/loader IDs 下交换父 roots、prefix、delegation 仍可区分环境与选择，同名不同 loader 不合并，旧 expected artifact 不误认；改变仅展示标签不改变物理选择。
最新指定范围复核：[2026-10-24 / v25](results/s1-environment-recheck-v25/README.md)。实际重现旧 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项与相邻回归合计 288 passed / 0 failed / 1 ignored（11 已含在 288 中）。2729 当前来源/构建前清单/归档匹配，七个二进制来自同次 V R1 构建，target 缺席。核验 V 5895 冻结 payload，引用同源码/CLI 的 136×3、6 编译/12 JVM、真实两类 4 重编原流，本轮未重跑 CLI/JDK。无新增产品/测试体修改或 Cargo；2.6 保持通过，2.9–2.10/S1 未完成，S2 未进入。此前 v24 为其各自历史验收范围。
当前工作区复核：[2026-10-21 A1.4 / v7](results/s1-environment-recheck-v7/README.md)实际重跑旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全部 11 项通过，相邻回归合计 286 passed / 0 failed / 1 默认 ignored。当前 534 项源码与 source-name-boundary R1 归档/构建及 42 个二进制匹配，target 已清理，无新增 2.6 产品修改。本轮重跑 136×3 正文/核心决定零差异、6 编译/12 受控 JVM、真实两类 4 重编。此前 [v6](results/s1-environment-recheck-v6/README.md) 仅引用旧 oracle 的证据边界保留；本次不授予已有 2.9 overlay 或 S1 整体验收，S2 未进入。
后继本构建 A1.4 的 11 项保持通过，见 [raw member 检查点](results/s1-raw-member-paths-v1/README.md)；该局部恢复迁移不改变 2.6 身份机制或授予 S1 整体验收。
本次指定 2.6 的 [v8 复核](results/s1-environment-recheck-v8/README.md)重现旧构建 2+7 失败，当前 11 项及相邻回归 286 passed / 0 failed / 1 默认 ignored。545 项源码/资产匹配 raw-member R4，核验其全部 8,382 个证据 payload；恢复保留引用同构建既有 oracle，本轮未重跑 Cargo/恢复矩阵/JDK，无新增产品修改。展示标签措辞仅有同内容路径变更的可表达测试轴，API 无独立标签字段，边界已说明；2.9–2.10 不额外勾选，S2 未进入。

本次指定 2.6 的 [v9 复核与测试补齐](results/s1-environment-recheck-v9/README.md)重跑旧二进制 2+7 失败；当前 A1.4 的 11 项及相邻回归 286 passed / 0 failed / 1 默认 ignored。补齐 bridge admission 缓存的跨 loader、准备 facts 快捷路径、零 Header 热命中和取消热命中定向断言，只改 cfg(test)，不改恢复语义。555 项源码/资产与本轮两条成功构建、六个二进制匹配，target 清理；中断重建收据保留。既有恢复 oracle 仅引用生产源码相同的 raw-casts R3 并核验 5737 个 payload，本轮未重跑矩阵/JDK。2.9–2.10 与 S1 整体不额外验收，S2 未进入。

本次指定 2.6 的 [v10 当前源码复核](results/s1-environment-recheck-v10/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 的 11 项及相邻回归合计 286 passed / 0 failed / 1 默认 ignored。577 项源码/资产重新构建六个目标，两条命令成功，target 已清理。生产源码与 raw-projections R2 逐文件相同，仅待验收匿名类测试/资产不同；本轮无产品或测试体修改。核验其全部 6786 个冻结 payload，恢复保留引用既有 oracle，本轮未重跑恢复矩阵/JDK。止于 2.6，不继续 2.9，不授予 S1 整体验收，S2 未进入。

本次指定 2.6 的 [v11 当前来源复核](results/s1-environment-recheck-v11/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全 11 项与相邻回归 286 passed / 0 failed / 1 默认 ignored。复用进入本轮时已启动的 R3 构建，589 项源码/资产与 56 二进制匹配，七构建命令成功且 target 清理；本轮无生产代码或测试体修改。实际重跑 136×3 正文/核心、6 编译/12 受控 JVM、真实成功两类 4 重编，全部保持，未执行真实目标。此前已启动的 2.9 回归在两项新增 enum-switch 测试失败后终止，原流保留，不宣称完整回归或 2.9 验收；相关事项仍归 2.9/2.10。本次止于 2.6，S1 整体未验收，S2 未进入。

本次指定 2.6 的 [v12 当前源码验收复核](results/s1-environment-recheck-v12/README.md)重新运行旧构建，实际复现 2 个 loader / 7 个身份失败；当前 601 项源码/资产重新构建六个目标，两条命令成功，A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 既有 ignored**。本轮未改生产代码或测试体；与 enum-switch R4 差异仅此前待验收的 Outer.super 两测试/12资产，未继续 2.9。核验 R4 全部 9537 个冻结 payload，恢复保留引用其既有 oracle，本轮未重跑矩阵/JDK。target 已清理，2.6 保持通过；2.9–2.10/S1 未验收，S2 未进入。

本次指定 2.6 的 [v13 当前构建复核](results/s1-environment-recheck-v13/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 既有 ignored**。当前601项源码/资产、源码包及56二进制匹配Outer.super R3，核验12121个冻结payload，target当前不存在；本轮复用该固定构建，无新增Cargo、生产代码或测试体修改。恢复保留引用同源码/二进制既有oracle，本轮未重跑矩阵/JDK；未继续2.9，2.9–2.10/S1仍未验收，S2未进入。

本次指定 2.6 的 [v14 当前构建验收](results/s1-environment-recheck-v14/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 既有 ignored**。三处 memo、defining loader 父链、caller IR/prepared 快捷路径与热命中取消保持。本轮实际重跑136×3正文/核心、6编译/12受控JVM、真实成功两类4重编，全部通过，未执行真实目标。601源码/资产及56二进制匹配源码职责迁移R1，核验其10529个冻结payload；target缺席，无新增Cargo、产品代码或测试体修改。本次止于2.6，未继续2.9；2.9–2.10/S1未验收，S2未进入。

2.6 收据：[环境身份与 defining loader/cache key](results/s1-environment-v1/README.md)、[逐目标验收](results/s1-environment-v1/acceptance.json)。A1.4 新增 11 项通过；父 roots/prefix/delegation/parent/uncertainty、provider 内容、旧 expected artifact 及同物理定义的多 loader memo 均验证。身份保存完整有序 domains/providers，binding schema 3；后继从 defining loader 查询，层级图按绑定节点区分。34 目标 1497 passed / 2 既有 failed / 4 既有 ignored，新增退化为零；136×3 与完整报告迁移对照、6 编译/12 JVM、真实两类 4 重编通过。两项失败已用进入 2.6 前的源码重跑确认，原流保留，见下方 2.10。S2 不进入。

本次指定 2.6 的 [v15 当前构建复核](results/s1-environment-recheck-v15/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全 11 项和相邻回归 **288 passed / 0 failed / 1 既有 ignored**。完整环境声明、defining loader 后继、三处缓存/caller 快捷路径及热命中取消保持，无新增 2.6 产品修改。613 项源码/资产及 57 二进制匹配 metadata R2，核验 8,452 个冻结 payload；target 缺席，本轮无新增 Cargo。实际重跑 136×3 正文/核心、6 编译/12 受控 JVM、真实成功两类 4 重编均保持，未执行真实目标。首次旧脚本数量断言失败与修正后完整复跑分别保留。2.6 保持通过，2.9–2.10/S1 未验收，本轮止于 2.6，S2 未进入。

本次指定 2.6 的 [v16 当前构建复核](results/s1-environment-recheck-v16/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全部 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。616 项当前源码/资产、归档及 57 二进制匹配 metadata-refusal R1，5079 个冻结载荷逐项校验，target 不存在；无新增 Cargo、产品代码或测试体修改。恢复保留引用同源码/二进制的既有 oracle，本轮未重跑矩阵/JDK。2.6 保持通过；本次止于 2.6，2.9–2.10/S1 未验收，S2 未进入。

本次指定 2.6 的 [v17 当前源码验收](results/s1-environment-recheck-v17/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。当前 618 项源码/资产与归档、三成功构建命令和 7 二进制匹配，target 清理；136×3 正文/核心零差异、6 完整类编译/12 受控 JVM、真实成功两类 4 重编全部实际重跑通过，未执行真实目标。5079 前继冻结载荷校验通过。既有 2.9 overlay 的 R4 构建中断、无 receipt，未重启或追认通过，仅清理废弃 target；本轮未改生产源码或测试体。2.6 保持通过；本轮止于 2.6，2.9–2.10/S1 未验收，S2 未进入。

本次指定 2.6 的 [v18 当前源码验收](results/s1-environment-recheck-v18/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。三处 memo、defining loader 后继、caller 快捷路径及取消热命中保持，本轮无生产代码或测试体修改。进入本轮时在途的 Signature R1 正常完成，621 源码/资产、58 二进制与七成功构建匹配，target 清理；仅借该构建验收 2.6，未执行 Signature 产品矩阵或追认 2.9。136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持，8405 前继载荷核验，真实目标未执行。本轮止于 2.6；2.9–2.10/S1 未验收，S2 未进入。

本次指定 2.6 的 [v19 当前源码验收](results/s1-environment-recheck-v19/README.md)先实际重现固定旧构建 2 个 loader / 7 个身份失败；当前源码新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。四个环境/loader/facade/身份测试文件与 v18 归档逐字节一致，未发现新增可复现 2.6 缺口，本轮未改生产代码或测试体。624源码/资产、归档及7二进制匹配三成功构建，target已清理；136×3正文/核心、6编译/12受控JVM、真实成功两类4重编实际重跑保持，未执行真实目标。前继Signature 10122冻结载荷核验；既有2.9 generic physical overlay的R2新测试4/5及后续未验收改动保留，不授予2.9通过。本轮止于2.6，2.9–2.10/S1未完成，S2未进入。

本次指定 2.6 的 [v20 当前源码验收](results/s1-environment-recheck-v20/README.md)实际重现固定旧构建 2 个 loader / 7 个身份失败；当前新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。环境/provider/loader 机制与身份测试不变；只修复已有 2.9 迁移的两个旧 emitter 调用以排除编译阻断，另 6 项既有 enum 正负/停止/受控行为回归通过，未扩大恢复规则或修改测试体。R1 两处 E0061 原流保留，R2 三构建成功；678 源码/资产及 7 二进制匹配，target 清理。117 个字面量 include 输入补充归档明确为编译后/测试前采集，非全量预构建冻结；关键 A1.4 输入完整字节与测试二进制匹配。136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持，真实目标未执行。本轮止于 2.6，未继续 2.9 恢复迁移，2.9–2.10/S1 未验收，S2 未进入。

本次指定 2.6 的 [v21 当前构建验收](results/s1-environment-recheck-v21/README.md)引用已实际复现并核验原流的旧 2+7 失败（本次不重复计数）；当前 A1.4 全 11 项通过，六目标 288 passed / 0 failed / 1 既有 ignored，另 6 项既有 enum 回归通过。环境/provider/callee/facade/身份测试五文件与 v20 逐字节相同，无新增 2.6 产品或测试体修改。借用在途 enum R1 构建的 9 条成功命令，2,640 项来源与归档、67 二进制匹配，target 清理；136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持，未执行真实目标。已有 2.9 enum 草稿未取得产品矩阵/负例/evidence 验收，不借本次结果追认；2.9/2.10/S1 未完成，S2 未进入。

- [x] 2.7 依赖 2.2、2.5、2.6，明确并接入解析/恢复语义版本与产物绑定；验证 **A1.5**：同包版本/同文本但证明语义变化的构建拒绝旧 evidence，相关物理依赖和词法/投影配置可追溯，evidence 选择不改主体，格式/cursor/语义版本域不强行合并，构建摘要可复现。
2.7 收据：[语义版本与方法绑定](results/s1-semantics-v1/README.md)、[A1.5 验收](results/s1-semantics-v1/acceptance.json)。schema 4 分开解析/恢复合同、源码实现身份与包版本/major 元数据，保留实际物理依赖与词法/投影 handoff。实际同版本/同文本语义变体仅 Java ID 改变，拒绝旧 expected 并保留正文；normal/test-support 与独立目录 CLI 的身份一致。34 目标 1501 passed / 2 既有 failed / 4 既有 ignored，新增退化零；136×3、完整报告、6 编译/12 JVM 和真实两类 4 重编通过。后继 2.8 验收见下方；2.9–2.10 未验收，S2 不进入。

- [x] 2.8 依赖 2.1、2.6、2.7，收窄内部 proof 构造并校验公开底层入口的同次 facts/subject/方法一致性；验证 **A1.6**：真实消费位置的假 proof、错方法/facts/subject 被拒绝或隔离为不可用于可信绑定的来源，Engine 路径无新增注入面，不为每条 proof 重读同一事实。
2.8 收据：[输入来源与可信边界](results/s1-trust-v1/README.md)、[A1.6 局部验收](results/s1-trust-v1/acceptance.json)。公开 Java 输入明确为 caller-supplied 预览且 unbound，Engine 在私有读取交接中验证一致性并提交 binding；外部 callback 不能授予来源，元数据构造器不作认证。真实消费位置的假构造器及错 facts/subject/runtime 过去绑定、现在隔离，预览保留；五个公开入口与私有混用负例通过。41 目标 1532 passed / 2 既有 failed / 4 既有 ignored，新增退化零；136×3、完整报告、6 编译/12 JVM、真实两类 4 重编通过。恢复合同 revision 2，schema 仍为 4。2.9 raw 名称与 2.10 资源/旧失败未完成，不宣称完整 A1.6 或 S1 已通过，S2 不进入。

- [x] 2.9 固定 D12 / CG 入口可达的 raw 名称/descriptor 消费者、method/loader/BCI/origin 与测试清单，完成该有限闭包审计及必要修复；验证 **A1.6**：合法 supplementary、源码不可表达、损坏 MUTF 分组，全部 raw 判断与来源可核对；纯所有权迁移列 E7，不按剩余 root 判据扩张。

2.9/2.10当前有限验收：[逐项审查](results/architecture-completion-v1/finite-exit-review.md)。R01–R16与R06/R07后继闭合，固定raw集18目标实际81/81；历史90误计已明确更正，旧收据不改写。当前Java598、root普通294/观测310、JVM251，来源/环境及直接停止控制保持；Code原Error/query/callee后继截断、owner原Report、elapsed首因及预取消未搜索范围均有确认反例和修复证据。136×3及408完整报告、6编译12受控JVM和真实两类4重编保持。2469份中108项差异正在2.11逐项审查，2.12正式资源仍未验收；本轮只关闭2.9/2.10，不宣告S1或S2进入。

2.9 有限清单检查点（2026-10-26）：[结果](results/s1-raw-consumer-closure-v1/README.md)、[清单](results/s1-raw-consumer-closure-v1/inventory.json)、[逐行核对](results/s1-raw-consumer-closure-v1/consumer-review.md)。R01–R16 与118生产源在运行前固定；CG收据19个payload全部bytes/SHA-256匹配，40个目录引用存在，历史局部结论未升级为本轮验收。确认R06把MUTF owner转标准UTF8导致字段T误降Object，三行raw/拼写分域修复及恢复47→48；未迁职责。新公开回归先红，最终15目标71项通过；两owner×source7/8×三evidence共12对CLI，ASCII全文6保持、supp投影0/6→6/6，物理field/method item保持。R07 synthetic字段过滤的必要origin线索尚缺公开入口对照，保留未决；2.9不勾选、17/65、2.10及S2/S3未执行。

2.9 R07后继（2026-10-26）：[synthetic字段来源](results/s1-synthetic-field-origin-v1/README.md)确认N1的合法supplementary捕获字段被标准UTF8过滤，必要写入origin丢失；仅改既有严格MUTF解码，恢复48→49，其余合同保持。新增公开回归先红；最终7目标39项通过。当前源码基线/修复后52对CLI中supp N1根类6份恢复折叠，其余46份正文及52对物理成员保持；隐藏写锚点的child owner/构造器/BCI2与raw字段核对。6次完整类空classpath编译、12生成JVM及4原夹具对照通过。DBS两来源的supp变体原本可投影，保留为无影响控制；不可表达/损坏组的旧边界分列，未修2.10状态传播。R07这一来源缺口已闭合，旧清单收据保留历史判断；未扩E7、未重跑整体验收，2.9仍未勾选、17/65。

局部进展：[raw member/callee/call](results/s1-raw-members-v1/README.md)、[raw 签名](results/s1-raw-signatures-v1/README.md)、[DynamicSite/SAM](results/s1-raw-dynamic-v1/README.md)及[Field](results/s1-raw-fields-v1/README.md)之后，当前[类名/嵌套关系检查点](results/s1-raw-class-names-v1/README.md)将类/包/父类型拼写改为严格 MUTF-8，InnerClasses 物理比较和 static fold descriptor census 改用 raw/canonical 编码。21 名称输入由上个检查点 9 编译增至 13 编译、26 次行为通过；四个子类独立查询由 0/4 变为 4/4 编译，但四个 RawDescriptor 根类仍失败，同源 ASCII 控制也失败，不计为根类恢复收益。4 损坏仍拒绝。当前 31 目标 1431 passed / 8 基线已有 failed / 3 默认 ignored；两项新增根库测试通过，原失败集保持。136×3、6 编译/12 受控 JVM、真实两类 4 重编与字段四组/8 JVM 保持。A1.4 全部 11 项通过；schema 7、恢复合同 8、解析合同 3、输入摘要域 v4。后继 [raw alias 检查点](results/s1-raw-alias-v1/README.md)已打通 DebugLocal/SlotEvidence/RenderedName、reuse、MethodParameters 与输入摘要：3×3 raw evidence/摘要可区分；6 参数组 18 编译/36 JVM，21 名称仍 13 编译/26 JVM；136×3 与既有成功保持。R3 产品 + R4 单一测试后继/reader 合计 40 目标 1697 passed / 8 基线 failed / 8 默认 ignored；两项旧碰撞丢名断言的原失败与加强编译后的迁移分别保留。schema 8、恢复合同 9、输入域 v5，解析合同 3；其余精确消费者及名称语法仍待闭合，不勾选整项。

后继[共享源码名称边界](results/s1-source-name-boundary-v1/README.md)将类/类型/包名收拢到严格 MUTF-8 与冻结 Java 8 Character 门，producer census 比较 raw descriptor；Frame 与 catch 不经 lossy 决定类型，普通引用拼写保留数组/Lfoo，修正非法字段 owner 被报 Structured。17 名称由 7 组非法或身份不保真输出、2 组合法包名被拒，变为 8 组双编译器通过、9 组明确拒绝；受控 catch 由 0/2 编译变为 2/2、4 JVM 对照通过。41 目标 1706 passed / 8 基线 failed / 8 默认 ignored；136×3、既有名称/字段/alias/参数名及真实成功类保持。当前 R2 534 源码/归档/42 二进制匹配，target 清理；schema 8、恢复合同 10、解析合同 3、输入域 v5。class literal/array raw 消费、member-inner/bridge 等精确链路仍未闭合，2.9 不勾选，S2 未进入。

后继 [raw 数组检查点](results/s1-raw-arrays-v1/README.md)迁移 class literal、anewarray/multianewarray 和 Named array 两处消费者，直接读取 raw descriptor/object leaf，保留嵌套准入、opcode、rank 与拒绝规则。实际基线 Java 572 passed / 3 新增 failed，当前同套测试 575 全通过；12 组同源变体的原样完整类编译由 8/24 到 24/24，48 次生成类 JVM 对照通过。41 目标 1709 passed / 8 基线 failed / 8 默认 ignored；A1.4 11 项、136×3、21 名称及真实两类 4 重编保持。534 Rust/manifest + 3 embedded test 资产与源码包/构建/42 二进制匹配，target 清理。member-inner/bridge 等精确交接和 2.10 门仍未闭合，2.9 不勾选，S2 未进入。

后继 [allocation/成员源码路径检查点](results/s1-raw-member-paths-v1/README.md)沿 CP raw owner/descriptor、enclosing/capture/构造器、source path 与声明投影保留 JvmBytes；Allocate/Site/NewRecord/census 及相关消费者同步迁移，最后由既有严格 MUTF-8/源码名称门拼写。两项真实 IR 回归从 0/2 到 2/2 通过；32 同源输入完整类编译由 16/64，经 R1 的 56/64，最终到 64/64、128 JVM 与 32 raw NewRecord 对照通过；2 个 varargs 补充输入达到4编译/8JVM，从R3两项假partial转为当前complete，保留ellipsis。R1/R2 的 supplementary package 8 次编译失败保留；R4 修正原始解码、完整类型 token/已投影路径计数及 varargs ellipsis 边界，R3 probe 与 release 中断分列。最终 51 目标 1755 passed / 8 基线 failed / 9 默认 ignored：42 根层以 R4 重跑，9 个无根库依赖的下层目标按相同源码/依赖输入复用 R1，不重复计数。A1.4 11 项、136×3、旧数组 24 编译/48 JVM、21 名称与真实两类 4 重编保持。545 项当前源码/资产与 R4 构建相符；选定 43 个 R4 + 9 个 R1 二进制有各自收据，全部 11 个 target 清理。schema 9、恢复合同 11、解析合同 3、输入域 v6；NewRecord.class JSON 改为 byte array，外部报告格式合同仍由 E3/2.10 承接。Bridge 与其余精确消费者及 2.10 门未完成，2.9 不勾选，S2 未进入。

后继 [cast/type-test 与 bridge 检查点](results/s1-raw-casts-v1/README.md)将两类 CP 操作、BridgeParameterCast、canonical/recheck 和 bridge 目标声明定位保留 raw；generic interface Header 严格解码 MUTF-8。身份 helper 与整个被调用描述符的源码门分开，保留对象类型 bridge 域。永久整类反例 0/1→1/1，24 同源输入原样完整类编译 40/48→48/48、96 JVM 与 24 raw descriptor 对照通过，不称为 holdout。当前 R3 全部 52 目标重跑：1764 passed / 8 基线 failed / 10 默认 ignored；3 Java + 1 整类新测试，另 5 passed/1 ignored 来自本次纳入的既有引用 cast 目标。A1.4 11 项、136×3、成员 64 编译/128 JVM、varargs 4/8、数组 24/48、旧名称与真实两类重编保持，真实包未执行。549 项源码/资产和 53 二进制绑定，7 构建命令成功，四个 target 清理；两次审查中断与修正保留。schema 9/恢复合同 12/解析合同 3/输入域 v6；raw 方法跨度与静态初始化等剩余精确消费者仍待复现，2.9 不勾选，2.10 与 S2 不进入。

后继 [raw 初始化/字段投影检查点](results/s1-raw-projections-v1/README.md)修正实例数组 helper 的 raw owner/name/descriptor 精确交接，access bridge 的源码字段名复用字段声明 written_name，物理证明仍按 raw FieldRef 与字段表索引。两项实际旧失败的永久测试转绿；28同源变体预期投影12/28→28/28，完整类编译48/56→56/56，112受控JVM一致。数组只计投影误拒绝修复，8次新编译成功来自supplementary字段组，不称holdout。R2全部53目标1770 passed / 8既有failed / 10默认ignored；新增2测试，另4通过是扩入既有N1。A1.4的11项、cast/成员/varargs/数组/名称、136×3正文核心和真实成功两类重编保持，真实包未执行。R1 census失败由旧reader实际复现，R2只更新新增两class/八Code的cfg(test)精确人口，全部目标重新构建。555源码/资产、54二进制与R2来源匹配；本phase三个target清理。schema9/恢复合同13/解析合同3/inputs v6；匿名捕获、enum switch和Outer.super剩余消费者仍待复现，2.9–2.10/S1不勾选，S2未进入。

后继 [匿名类 raw 身份检查点](results/s1-raw-anonymous-v1/README.md)在现有 tuple/nested site 保存已验证 candidate.class，让 typed rows、resolver、capture/xref 与扫描对位消费 raw，最终接口/outer/super 拼写复用严格 MUTF-8。完整 census/唯一verified/BCI/descriptor/形状和原子提交守卫保持；super 两处 UTF-8 消费同时修正，旧两次 CLI panic 消失。三项永久反例0/3→3/3；48同源变体投影20→48、完整类编译40/96→96/96、192JVM一致。super16组内联2→8、编译4/32→16/32、32JVM一致，8组javac8原shape拒绝保留并由S5/6.9矩阵承接，不称holdout。R1全部55目标1783 passed / 8既有failed / 10默认ignored；新增3测试，另10通过来自扩入既有super目标。A1.4的11项、136×3、旧cast/初始化/成员/varargs/数组/名称与真实成功两类4重编保持，真实包未执行。577源码/资产、56二进制与7成功构建匹配，两个target清理；schema9/恢复合同14/解析合同3/inputs v6。enum-switch、Outer.super及2.10门仍未完成，2.9/S1不勾选，S2未进入。

后继 [enum-switch raw 检查点](results/s1-raw-enum-switch-v1/README.md)让 IndexCall、enum owner、values field/factory 与 descriptor 直接传递 JvmBytes；用同次IR的anewarray CP raw类名核对元素，最终label严格MUTF解码，保留ordinal/对象/构造器/闭合helper/handler/BCI守卫。28同源输入投影8→14、完整类编译16/56→28/56、56生成JVM一致；12个raw拒绝中仅6个取得整类恢复，另6个显露后继canonical拒绝，当前14组javac8原形状缺口归S5/6.9。两组private metadata名字变化在JVM8/23先校验，再由旧0/2投影、0/4编译到当前2/2、4/4和8JVM一致；不称holdout。R3两项末尾断言失败原流保留；R4修正注释显示与可执行源码断言，并用修订后的同一测试源码在旧实现重建得到0/2、当前2/2。统一R4全部55目标1785 passed / 8既有failed / 10默认ignored；A1.4 11项、4×3 enum evidence、136×3、既有保留矩阵与真实成功两类4重编保持，真实包未执行。589源码/资产、56二进制及7构建相符，六个phase target清理；schema9/恢复合同15/解析合同3/inputs v6。IndexCall报告raw数组格式门归E3/2.10；Outer.super、annotation/default、generic/constructor等仍有静态raw边界待实际复现，2.9–2.10/S1保持未验收，S2未进入。

后继 [Outer.super raw 交接检查点](results/s1-raw-outer-super-v1/README.md)把目标 owner/name/descriptor 直接作为 JvmBytes 交给 Java，validator/参数/返回消费 raw，最终方法名与 needle 严格 MUTF-8 拼写；共用完整 descriptor 与规范类名门，保留 physical/SSA/BCI/闭合/overload/异常/事务守卫，无 String/raw 双轨。六源码文件，schema10/恢复合同16/解析合同3/inputs v7。两项同测试源码旧0/2→当前2/2；effects28完整类编译24/56→56/56、112生成JVM一致；补齐实际分析 Header 的reference12完整输入8/24→24/24、48JVM一致，12raw descriptor/physical method及UTF-8跨度核对通过。首次误查Member及随后缺Parent/Token的旧探针原流保留，不充作完整环境基线；合法`_`父类复用规范拼写后4组8编译/16JVM已含effects。统一R3全部55目标1788 passed / 8既有failed / 10ignored，A1.4全11、7×3 evidence、136×3及既有保留矩阵/真实两类4重编保持，未执行真实包。601源码/资产、56二进制和7成功构建匹配，四target清理；R1类型错误与R2中间收据保留。两矩阵是开发输入，不称holdout。其他raw边界及facade内剩余源码判据职责继续承接，2.9–2.10/S1未验收，S2未进入。

后继 [interface-super raw 检查点](results/s1-raw-interface-super-v1/README.md)删除String候选实体，CP三元组到Java消费保留JvmBytes，最终拼写复用canonical类名和共享完整method门；default/层级/loader/entry-this/停止守卫保持。两项同源码永久反例0/2→2/2；56同源开发输入qualified-super32→56，完整类编译64/112→112/112、224生成JVM一致；JVM合法bad-name的错误Structured改为带raw/alias/BCI的拒绝，孤立surrogate和损坏MUTF拒绝保持，后者未执行。55目标1790 passed / 8既有failed / 10默认ignored；A1.4全11、24新evidence、80旧Outer编译/160JVM、136×3及真实两类4重编保持。首次比较器混入可选详情失败原流保留，五门补跑后独立核验九门；不称原流水线全绿。606源码/资产、56二进制和七成功构建匹配，两个target清理，峰值958433241 bytes。六生产文件，schema11/恢复合同17/解析合同3/inputs v8；facade其余源码策略、annotation/default/generic/constructor等与2.10门继续承接，2.9–2.10/S1不勾选，S2未进入。

后继 [metadata字面量职责/raw检查点](results/s1-raw-metadata-literals-v1/README.md)将既有MemberDefault、ConstantValue/default与注解元素纯解析/拼写归Java，root保留同一属性读取/停止/隔离/类组装；复用JvmString和emitter转义表，区分名称标识符门与字符串UTF-16值、损坏MUTF的Err。两项同源码永久反例0/2→2/2，另8旧项保持；56双目标同源开发输入编译96/112批次→112/112（224完整source units）、行为32/56→56/56、224生成JVM一致。修复NoBody损坏default被替换字符且Complete；合法不可表达enum名不写非法token，raw属性保留，但外层default Option的“缺省/源码拒绝”合并仍是确认盲区，须由本项后继闭合，不能宣布A1.6完成。56目标1801 passed / 8既有failed / 10默认ignored；42evidence、112raw来源/普通body-map、136×3、6编译/12受控JVM及真实两类4重编保持。613源码/资产、57二进制与R2七成功构建匹配，三个target清理，峰值974082806 bytes；R1缺导入失败原流保留。schema11/恢复合同18/解析合同3/inputs v8；2.9–2.10/S1不勾选，S2未进入。

后继 [metadata 拒绝状态检查点](results/s1-metadata-refusal-v1/README.md)让 Java 的已存在值 resolver 返回 Result<MemberDefault>，root None 仅表示属性缺失；方法复用 Refused，字段复用标记/诊断并隔离局部错误，全局预算/取消仍截断。四生产文件，无新 DTO/Session/cache/crate；恢复合同 19，schema11/解析3/inputs v8。相同测试源新两项旧0/2→当前2/2，原10项保持；24开发输入的16个错误Complete改为Partial，8损坏仍Failed且保留邻接成员。56目标1803 passed / 8既有failed / 10ignored，A1.4全11、既有metadata112编译批次/224JVM、42evidence与112raw/body-map、136×3、6编译12受控JVM及真实两类4重编保持，未执行真实目标。616源码/资产、57二进制与七成功构建匹配，三个target清理；四紧预算探针通过，只声称逻辑Body。首次测试API错误、状态验证器摘要算法假设及预算成本假设原流保留，纠正后六门独立验收，不称首次流水线全绿。2.9–2.10/S1仍未验收，S2未进入。

后继 [metadata 类型路径检查点](results/s1-metadata-type-paths-v1/README.md)将已存在 ResolvedInnerClass 迁入 Java naming，借用同次 raw rows 为普通 descriptor 声明、注解使用及递归默认值提供统一路径；root 只读取 raw default、供给上下文并保留停止/隔离/类族事务。五生产文件，无新增resolver/read/cache/Session/crate；恢复合同20、schema11/解析3/inputs v8。20同源输入的40完整编译批次0/40→40/40（80 source units）、80生成JVM一致；保留真实 `$`、深层及supplementary边界，40 raw报告/30 evidence对照保持，60处type annotation spelling逐项验证。两root旧反例转绿，另两Java unit；新重复/矛盾row拒绝与BodyProbe逻辑Body4→3通过，不声称actual IR。R5七构建成功，56目标1807 passed / 8既有failed / 10ignored；78完整类文本、24旧状态/四预算、旧metadata112编译/224JVM及42 evidence、136×3与真实两类4重编保持，真实包未执行。618源码/资产、57二进制匹配，六target缺席，5079前继载荷核验。R1/R2错误、R3新增失败、R4中断及验证器错误原流保留；R5首流水线因raw/spelling DTO比较退出，修正后补跑七门并独立验收十门，不称首次全绿。Signature/generic、Body类型上下文、constructor/private owner及2.10总门继续按[next-boundaries](results/s1-metadata-type-paths-v1/next-boundaries.md)承接，2.9–2.10/S1不勾选，S2未进入。

后继 [Signature raw 名称检查点](results/s1-raw-signature-names-v1/README.md)将递归 Signature 拼写及 body TypeVariable 出口迁入现有 Java 模块，全部类型变量声明/引用共享严格 MUTF-8 标识符决定；root 保留 raw scope/erasure、成员事实、停止与类事务。四生产文件，无新读取/resolver/cache/crate；恢复合同21，schema11/解析3/inputs v8。四项同源码永久反例0/4→4/4；48同源javac输入泛型32/48→48/48、反射行为128/192→192/192，96/96编译保持，不称编译净收益或holdout。57目标1813 passed / 8既有failed / 10ignored；36 evidence、48 raw/body/map、完整报告预期差异及逻辑Header/Body零增量通过，12负例拒绝泛型/损坏Failed且未执行。旧metadata112编译224JVM、20类型路径40批次80JVM、78完整源码、24状态/四预算、136×3及真实两类4重编保持，真实包未执行。621来源/资产、58二进制与七成功构建匹配，baseline/product target清理，8405前继载荷核验。首负向门因验证器误要求损坏输入exit0失败，原流保留并独立核对exit4后补跑八门，不称首次全绿。后续物理field/method名、constructor/private owner、Body类型上下文及2.10门沿[next-boundaries](results/s1-raw-signature-names-v1/next-boundaries.md)承接；2.9–2.10/S1不勾选，S2未进入。

后继 [泛型物理名称检查点](results/s1-generic-physical-names-v1/README.md)让同类调用key/库存保留raw三元组，field/method词法及六处AST连接共享严格MUTF标识符出口，constructor规范owner连接保留同次SSA/完整使用/停止守卫。五生产文件，无新增读取/resolver/cache/Session/crate或序列化字段，恢复合同22，schema11/解析3/inputs v8。五项同源码永久反例0/5→5/5；60输入泛型反射40/60→60/60，120重编/240正常运行均保持，收益为元数据恢复。61目标1869 passed / 8既有failed / 10ignored，其中补入三个既有generic目标49项全通过；45evidence、60raw/body/map、108套绑定预期更新及完整报告差异门通过。六负例保留raw/显式别名/泛型拒绝或准确Partial/Failed前缀，未执行变异输入。Signature96编译192JVM、metadata40/80及112/224、78源码/24状态/四预算、136×3与真实两类4重编保持，真实目标未执行；field-write/setter两组R2已绿控制48编译96JVM保持。674来源/资产、62二进制和八成功构建匹配，target清理，10122前继载荷保持；准确同字节CLI复用已实际运行的60组矩阵，无重复计数。R1 check/R2 native及R3检查器误判Partial的失败原流保留，补跑12门后覆盖18门，不称首次全绿。后续Body类型上下文与2.10总门承接，2.9/2.10/S1仍不勾选，S2未进入。

[Body 类型上下文检查点](results/s1-body-type-names-v1/README.md)将完整 raw InnerClasses rows 沿 MethodDeclaration/Java facts/Declaration/retained AST 共享，commit/replay/fragment 复用同一 SourceClassNames，删除成员名列表双轨；具名边核验后投影，local/anonymous/无行保留物理拼写。两项永久反例0/2→2/2，60开发输入重编48/120→120/120、240隔离JVM精确匹配；65目标1897 passed / 8既有failed / 10ignored，R3两项匿名回归已在R4恢复且原流保留。300绑定/60raw与map/120额外evidence通过，既有泛型、Signature、metadata与136×3/真实两类4重编保持，真实包未执行。2,624源码/资产和66二进制匹配，target清理；schema12/恢复23/解析4/inputs v9。v1原流遗失与v2历史generator pin失配已单列，不冒充全部输入历史可核验。嵌套enum child-context待实际复现，逐方法解析/资源总门归2.10；2.9/2.10/S1不勾选，S2未进入。

[嵌套enum上下文复核](results/s1-nested-enum-context-v1/README.md)关闭先前未闭合的可到达假设：terminal body Some在当前Engine唯一生产点必带delegation与constructor field index，nested render在二次emit前拒绝；正式8输入的实际拒绝一致，两个javac23物理家族重编和4次JVM保持。248个pins匹配，历史harness/空stdout核验错误已保留并分列，只有closure-v3作正式结论。不新增行表或放宽委托域；死分支清理归2.10。沿链发现活跃enum名称连接的11类非BMP失败，进入[enum raw名称实施](results/s1-enum-raw-names-v1/README.md)，当前旧源码四测试1通过/3失败，产品未验收；2.9/2.10/S1不勾选，S2未进入。

本构建补充：[enum R3 局部检查点与 A1.4 保持](results/s1-enum-raw-names-v1/README.md)、[收据](results/s1-enum-raw-names-v1/acceptance-r3.json)。A1.4全11/相邻288通过；四文件及facade的2.6片段保持，无新身份补丁。66目标1902 passed / 8既有failed / 10ignored；原四永久反例1/3→4/0，第五R2失败→R3通过；132矩阵R1源码/报告限定差异保持，十个`_`20生成编译/40JVM通过，14evidence/15负例与168旧完整源码+78声明保持。136×3、6编译12受控JVM及真实两类4重编通过，真实目标未执行。初evidence比较器缺漏RuleDetails.fields的失败原流保留，v2补验通过；2643来源/67二进制和九步构建匹配，target清理。其它raw候选未实际复现，2.9/2.10/S1仍不勾选，S2未进入。

R3时点的2.9待补（已由下方调用名称检查点关闭）：R3负例实际确认terminal helper的dash/surrogate名称仍以原样调用文本出现，缺结构化raw bytes/alias；enum投影拒绝不能充当方法源码合法性。见[逐报告分类](results/s1-enum-raw-names-v1/negative-review-r3.md)。本次2.6不扩大修复，后续先复现该实际消费者的完整证明链与普通控制，再迁移共享命名门；不得用Complete或无enum投影验收全类。

调用名称局部检查点：[共享方法拼写](results/s1-invocation-name-boundary-v1/README.md)、[收据](results/s1-invocation-name-boundary-v1/acceptance-r1.json)。Java names统一普通声明/普通调用及特殊调用拼写，保留raw身份和Java7/8 `_`，不可表达调用走raw/BCI fallback；五生产文件，恢复24→25，schema12/解析4/v9保持。四永久旧1/3→当前4/0；四invoke×四合法名称的完整类32编译30→32、生成JVM60→64；target7 `_`另4编译8JVM。12负例24错误Structured→Mixed/Fallback，12损坏报告仍Failed，变异不执行。67目标1906 passed / 8既有failed / 10ignored；A1.4全11/相邻288保持。R132完整报告、十个enum `_`源码、15enum负例、168旧完整源码+78声明及136×3/6编译12JVM/真实两类4重编保持，真实目标未执行。初harness空源码、基线排序、构造器比较器及收据解析错误均保留并明确排除/补验；2669来源/68二进制和9构建匹配，target清理。原terminal helper非法token缺口已在本范围修正；lambda/MethodHandle与generic/enum独立名称门及其它raw消费者未闭合，2.9/2.10/S1仍不勾选，S2未进入。

MethodHandle/helper 局部进展（2026-10-23）：[MethodHandle/helper 检查点](results/s1-lambda-handle-names-v1/README.md)、[收据](results/s1-lambda-handle-names-v1/acceptance-r4.json)：六生产文件复用 Java 方法名门与声明别名，具名非lambda$ MethodHandle拒绝不可表达名称，闭合helper的inline/rename按严格MUTF连接；声明和暂存别名冲突不得合并不同raw名，原同raw重载保持。恢复25→26，schema12/解析4/v9不变。相同测试源码旧2/4→6/0；16合法开发输入完整类编译28/32→32/32、生成JVM56→64；68目标1912 passed / 8既有failed / 10ignored，A1.4全11/相邻288保持。45负例、声明/暂存碰撞与重载保留、旧完整报告/源码及136×3、6编译12受控JVM、真实两类4重编通过，真实目标未执行。2700来源及归档匹配，显式选择12个R1下层+57个R4根层/CLI，不称同一次69构建；所有target清理。

**开放门不能省略**：captured-this kind7/ASCII helper原编译1/原JVM2成功，旧生成2编译失败，当前all完整源码逐字相同且无投影，诊断为空。本轮只实际运行一个输入；其它预想输入/证据未运行。13个独立门通过，instance门失败，不能宣告2.9通过。工具javap文本/目录冲突与真实缺口的原流分别保留，成功oracle未重复计数。未闭合helper、独立方法及碰撞caller fallback、其它raw消费者和2.10总门继续承接，2.9/2.10/S1未完成，S2未进入。

captured-this需求门检查点（2026-10-23）：[captured-this共享需求门](results/s1-helper-demand-v1/README.md)、[收据](results/s1-helper-demand-v1/acceptance-r1.json)关闭前继kind7需求遗漏：header不再要求static，精确raw身份、kind/flags、完整使用、单site、同次AST及停止/提交守卫保留。两生产文件，恢复26→27；schema12/解析4/inputs v9不变。相同七测试旧6/1→当前7/0；ASCII/supplementary×inline/rename四开发输入完整生成编译0/8→8/8、16生成JVM一致，非holdout；首旧oracle按完整源码精确相同引用并独立核验，不冒充重新运行。固定输入essential/all的actual Body解码4、IR六阶段各4、entry读取2次833字节与红基线相同；scan/AST保留和逻辑成本增加，不外推完整S1资源门。五变异15完整拒绝报告及明确IR预算Partial无提交通过，变异未编译/执行。68目标1913 passed / 8既有failed / 10ignored，A1.4全11/相邻288保持；L110完整报告、旧普通/enum/generic/Signature/Body/声明及136×3、6编译12受控JVM、真实两类4重编保持，真实目标未执行。

初构建被系统中断，39前缀产物仅有五步Cargo成功事件，退出码/峰值/独立rustc记录缺失；接续四步实际exit0生成30产物，69项显式来源选择、2701来源/归档匹配，target清理。不称九条正常退出或完整S1资源通过。11门和finalize通过；未闭合/helper独立方法与碰撞后caller的最终fallback、其余raw消费者及2.10总门仍未完成，OpenSpec17/59，2.9/2.10/S1不勾选，S2未进入。

[lambda 最终消费检查点](results/s1-lambda-final-consumption-v1/README.md)、[收据](results/s1-lambda-final-consumption-v1/acceptance-r3.json)（2026-10-23）：五生产文件让 raw Member、同次 BCI envelope、公用名称终结与私有 provisional 类候选闭合；root 复用精确 ExplanationOnly marker 及现有原文/span 门。未闭合引用不输出非法调用，整方法 quote 同步撤销丢弃的 lambda/accessor 状态；保留闭合 helper 和 captured-this。正式旧十项7/3，R3原十项加新增合法 sibling 共11/0，另一个root marker/span单元通过；R1有效投影退化和R2合法sibling误报的实际失败原流保留，不能称首次全绿。R3同次九命令成功，2708来源/归档、69二进制匹配，target清理，恢复27→28/schema12/解析4/inputs v9。68目标1918 passed/8既有failed/10ignored；27拒绝、三组合9控制/18变异、合法sibling三evidence和16公共预算stop/8零预算拒绝通过。组合原3编译6JVM、生成6编译12JVM在R2已成功，R3核验完整源码和产物/原流后引用，不重复计数；变异未编译/执行。122 lambda/captured-this（9旧非法文本改quote）、84 invocation、132 enum、adapters、168旧源码/78声明、当前136×3/6编译12受控JVM及真实两类4重编保持，真实目标未执行；固定captured-this实际Body4/IR六阶段各4/read2/833bytes保持。其它raw消费及完整职责/停止/资源门按[next-boundaries](results/s1-lambda-final-consumption-v1/next-boundaries.md)承接，2.9/2.10/S1不勾选，S2未进入。

[数组 helper raw 交接检查点](results/s1-array-helper-raw-v1/README.md)、[收据](results/s1-array-helper-raw-v1/acceptance-r2.json)（2026-10-24）：删除Site三份helper String，以同次IR的BCI/CP连接raw handle，最后统一命名；Java层接管整个声明/返回类型匹配，facade只供选中事实和原事务。四合法开发输入数组引用投影1/4→4/4，旧生成8编译/16JVM原本成功，不算编译成功率收益。R1新native实际失败后补齐遗漏门，R2新测试1/0、旧数组16/0；12报告、6新编译/12受控JVM和六变体36恢复报告通过，变异未编译/执行，损坏编码保留明确失败/可靠前缀。70目标1935 passed/8既有failed/10ignored；A1.4全11及相邻288/0/1、旧能力与当前136×3/6编译12JVM/真实两类4重编保持。2713来源/归档与71产物匹配，九命令同次正常成功、target清理；恢复28→29/schema12/解析4/inputs v9。比较器三类错误和原流保留，成功oracle核验后引用，不称首次全绿。其余raw消费者仅静态清点，按[后续边界](results/s1-array-helper-raw-v1/next-boundaries.md)继续2.9/2.10；OpenSpec17/59，S1未完成，S2未进入。

[write-accessor raw descriptor 检查点](results/s1-write-accessor-raw-v1/README.md)、[收据](results/s1-write-accessor-raw-v1/acceptance-r1.json)（2026-10-24）：复用 reader descriptor components 和现有 source_names.current_class，将已有 key/expected_descriptor 保留为 JvmBytes，字段/header/frame/成对发射共同核对 raw 身份；仅源码 Type 走共享严格命名，无新增读取/缓存/身份实体。两个生产文件，恢复29→30，schema12/解析4/inputs v9保持。四合法开发控制 helper Structured 1/4→4/4，完整生成编译2/8→8/8；三个变化源码新增6编译12受控JVM，ASCII同字节引用旧2编译4JVM。12控制报告、六变体36选定负例通过，变异未编译/执行；损坏字段按Partial可靠前缀/skipped fields[0,1)交付。71目标1936 passed/8既有failed/10ignored，A1.4全11/相邻288/0/1、旧能力、当前136×3/6编译12JVM和真实两类4重编保持，真实目标未执行。2718来源/归档与72二进制匹配，九命令同次正常成功、target清理。Partial/Failed与enum嵌套合同比较器失败原流保留，续验复用成功控制，不称首次全绿；冗余负例重跑不累加。剩余六行/五机制静态消费者及2.10资源/停止总门见[后续边界](results/s1-write-accessor-raw-v1/next-boundaries.md)，ASCII WA三项旧失败未修复；2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

[泛型消费者源码拼写检查点](results/s1-generic-source-consumers-v1/README.md)、[收据](results/s1-generic-source-consumers-v1/acceptance-r2.json)（2026-10-24）：raw receiver/List元素共享严格名称出口，同次AST raw InnerClasses闭合真实成员路径，字面$不猜嵌套；四生产文件，恢复30→31/schema12/解析4/inputs v9保持。六开发控制的泛型反射8/24→24/24，完整生成编译12/12保持；当前新8编译16JVM，两个同字节ASCII复用旧4编译8JVM，非holdout。两个dollar类保留旧Partial/anonymous_typed_nesting_unproved，不能称Complete。R3错误class<T>控制与R1新test2/1原流保留，R2修正测试3/0；补入旧List目标19/0/1。73目标1958 passed/8既有failed/11ignored；18正报告、七变异42报告、15局部保留及旧能力门满足。A1.4全11/相邻288保持；136×3/6编译12JVM/真实两类4重编核验同CLI的v24冻结原流后引用，未重复执行，真实包未执行。2726来源/两归档匹配，74binary显式选R1排除错误oldtest+R2testfix，不称完整R2同次构建，target清理。实际新复现overload raw cast：class-scope T/E的ASCII反射4/4、supplementary0/4，当前CLI原文保持缺口，下一步只迁移AST拼写并保留raw语义binary_name；其它消费者及2.10总门按[next-boundaries](results/s1-generic-source-consumers-v1/next-boundaries.md)继续。2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

[显式bound cast raw身份检查点](results/s1-generic-cast-source-v1/README.md)、[收据](results/s1-generic-cast-source-v1/acceptance-r1.json)（2026-10-24）：AST Type复用共享严格名称出口，后续SignatureType::Class.binary_name直接保留raw MUTF；候选/explicit bound/most-specific/flags/census守卫不变，两个生产文件，恢复31→32/schema12/解析4/v9。原两输入2编译4JVM、旧生成4编译8JVM核验，relay泛型反射4/8→8/8，生成编译4/4保持；supp新2编译4JVM，ASCII同字节引用旧2编译4JVM，非holdout。六正报告/七变异42报告通过；前六保留泛型拒绝/擦除，截断Failed可靠方法前缀1/4、skip[1,4)，变异未编译执行。74native1959 passed/8既有failed/11ignored，新永久test1/0；U正负39、T/memberpath15及旧能力门保持，A1.4全11/相邻288保持；当前32 CLI实际重跑136×3/6编译12受控JVM/真实两类4重编通过，真实包未执行。2729来源/归档、75binary/74cleanup与九命令同次构建匹配，target清理，U4952冻结payload核验。root generic void/nested bound源码策略按[职责输入](results/s1-generic-cast-source-v1/responsibility-audit-input.md)归既有2.10/A1.1；nested bound/raw span/needle及资源退出门仍待，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

- [x] 2.10 依赖 2.9，收口冻结需求链的 read/IR/proof/emission/提交停止，包括 shared emitter Interrupted 与原 Report/Error 消费；验证 **A1.3**：token、预算、elapsed 分别保存首因/diagnostics，局部拒绝与全局停止分开，停止后实际 Body/IR 为零、无半提交，合法前缀保留；未触达的 bulk 协调故障归 E2/S6。

2.10 职责清点：[generic void/nested bound](results/s1-generic-cast-source-v1/responsibility-audit-input.md)记录真实消费和已有输入类型。该链由W完成路由/严格准入/拼装，X完成三专门策略，Y承接同类generic body声明；root保留事实、subject、停止合并与提交。ordinary/no-body/constructor/class-scope及退出门仍待下方后续清单，不能因这些局部迁移关闭A1.1总门；原过窄审查稿与修订均保留。

2.10 职责复核输入（Outer.super raw 检查点时）：该次审查确认，`facade::prove_outer_super_source_binding` 仍包含 Java 源码重载/泛型/checked-exception 准入，相关参数类型证明亦与事实供给混合。词法门已计划统一到 Java 层，但不足以证明所有类族源码决定的所有权闭合。收尾须逐项核对这些真实消费者，将纯源码判据归 Java、保留 root 的同次物理事实供给/停止/事务，并用同一正负矩阵验证，不以 2.1 旧收据或本轮局部 raw 修复代替当前状态验收。该项承接已有 A1.1 职责要求，不另开 S2 或放宽 bridge 域。

[Outer.super 源码职责迁移](results/s1-outer-super-source-policy-v1/README.md)完成本项 A1.1 局部承接：泛型/Exceptions、源码重载、已发布参数和 SSA direct-load/严格子类判据归 Java；root 仅保留物理一致性、选定事实供给和原 execution 合并，census/事务保持。只改两生产源码，原测试体与接受域不变；55目标1788 passed / 8既有failed / 10默认ignored。40完整输入80编译/160受控JVM保持；40报告及36开发输入的42调用在校验两个确切实现摘要变更后，其余字段及逻辑usage一致，7×3 evidence、136×3与真实两类4重编保持，真实目标未执行。首次比较器未处理源码实现身份变化的失败原流保留，补跑后验收；schema10/恢复合同16/解析合同3/inputs v7不变。601源码/资产、56二进制与七成功构建匹配，target清理；实际Body/IR复用和所有职责的总盘点仍归本项后续，2.9–2.10/S1不勾选，S2未进入。


2.10 新增承接（本轮未修复）：[旧产品对照](results/s1-environment-v1/adversarial-review.md)确认 EM11 cap2 停止测试预期已失效，新旧都完整恢复；另有预取消的 skipped 范围遗漏，违反既有范围合同。两项继续计入回归失败分母，取消缺口不能靠改空范围预期换绿。完整声明复制/报告字节的新增成本亦沿原资源协议验收，不调整阈值。

2.10 测试合同承接：[raw 签名逐目标对照](results/s1-raw-signatures-v1/test-comparison.json)保留两项检查 `prepared.method_code` 的旧源码字符串断言及一项 prepared/direct 来源字段比较失败。后者应分别断言公开 Java 为 CallerSupplied/unbound、Engine 为 SelectedReads，再在等价呈现字段上比较；不得删除产品来源区分。三项在本轮改动前的 raw-members R2 与当前产品均实际失败；旧测试源不变不等于已用 2.8 原构建复现，证据等级见检查点。

2.10 扩大回归承接：[Field 对抗复核](results/s1-raw-fields-v1/adversarial-review.md)保存成员类/outer-super 两项断言和 WA 整类 main 三项失败；迁移前独立构建实际得到同样五项，不声称在 2.8 原构建复现。PreparedFold 与 Prepared 的断言差异须按真实来源/投影合同核对；WA main 仍是恢复缺口，不能全部改成通过预期。若根因在尚未完成的 2.9 名称链，先由该项修复，再执行 2.10 总回归；不混入当前 Field 身份迁移，也不免除 S1 退出门。

[泛型声明职责与raw sibling检查点](results/s1-generic-declaration-boundary-v1/README.md)、[第三轮收据](results/s1-generic-declaration-boundary-v1/acceptance-r3.json)（2026-10-24）：Java统一generic-void的Option<Result>选择/严格生成及共享generic方法声明拼装，root保留事实/原候选顺序/停止/提交；三生产文件，无新DTO/Session/cache/read。source_class_name/source_identifier替代三个标准UTF8边界，raw唯一sibling/同outer/binary重建/physical $拼写门保持，恢复32→33/schema12/解析4/v9。两开发输入selected-leaf泛型反射2/4→4/4，JDK23生成编译2/2保持；supp新1编译2VM、ASCII同字节引用旧1编译2VM。JDK8 leaf编译exit4旧限制保留，不称完整生成族或holdout收益。R3六门正常通过：14native957/0/1，6正报告/六metadata变体36报告、168完整源码/78声明保持，当前136×3/6编译12受控JVM/真实两类4重编通过，真实目标及变异未执行。2732来源/构建前/归档、15二进制与四正常命令同次匹配，target清理；R1编译失败、R2成功但route职责遗留、误选R2归档的审查原稿均保留，R3已闭合本链。其余泛型专门策略及2.10资源门按[next-boundaries](results/s1-generic-declaration-boundary-v1/next-boundaries.md)继续，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

[泛型专门策略迁移](results/s1-generic-declaration-policies-v1/README.md)、[收据](results/s1-generic-declaration-policies-v1/acceptance-r1.json)（2026-10-24）：Java接管method-local empty-void throws、instance null-return、static local throws三条策略的Option<Result>路由/严格准入/完整声明，root保留事实、原顺序、停止与提交；复用共享header，无新DTO/Session/cache/read。实际以W33 CLI复现只改Signature CP #11的interface-only bound导致exit101/panic，Code不变；Java显式要求class-bound且无附加interface-bound后局部拒绝，保留物理方法/正文/Exceptions，合法控制保留泛型。恢复33→34/schema12/解析4/v9。16选定native961/0/1，新增永久反例1/0；静态throws既有3测试扩入cohort，不称新增恢复收益。当前原/变异6报告、W正6报告/负18报告通过；对W33的18负例源码/正文/完整coverage/前缀严格保持，对W32另18比较仅3个bridge旧拒绝文案不同并显式记录，不称完整JSON一致。168完整源码/78声明及当前136×3/6编译12受控JVM/真实成功两类4重编保持，真实目标及变异未执行。2735来源/构建前/归档、17产物与四成功命令同次匹配，target清理，构建峰值755250698bytes不冒充S1资源验收。W6218冻结payload全部核验保持；剩余generic body/ordinary/no-body/constructor/class-scope策略及bridge/needle/2.10资源停止门按[next-boundaries](results/s1-generic-declaration-policies-v1/next-boundaries.md)承接，八项既有失败未修复或重测，本次非全仓门；2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

[同类generic body声明迁移](results/s1-generic-body-declaration-v1/README.md)、[收据](results/s1-generic-body-declaration-v1/acceptance-r1.json)（2026-10-24）：完整bounds/formals/return sources/throws准入与header组装归Java，root只核对certificate↔physical member、供事实/原顺序/停止/提交；复用descriptor/严格命名/共享header，无新DTO/Session/cache/read。两生产文件纯迁移，合同34/schema12/解析4/v9保持。现有预算sweep加入method-local T/U的MethodShadow，不新增test function；扩入既有same_class_generic_binding16项，17选定native977/0/1。四开发context×三evidence的12报告真实进入本route，当前源码/方法/execution/完整coverage/logical usage与旧X一致（仅测量elapsed排除）；旧完整生成8编译16VM及原oracle16VM已绑定，当前同字节复用，非holdout/恢复域收益。旧/当前同130cutoff全匹配，32个双body完整停止点均0或2泛型头。普通UTF8非MUTF name只改CP #8，旧/当前各3报告均Reader Failed，methods[1]拒绝且保留init前缀/完整coverage，未进genericroute，变异未编译执行；任意raw helper域差异不能冒充产品可达域变化。168源码/78声明、当前136×3/6编译12受控JVM/真实两类4重编保持，真实目标未执行。2735来源/归档/构建前与18产物、四正常命令匹配，target清理，峰值770427606bytes仅是构建守卫。target对象审查、elapsed比较、复制stem与tuple序列化四类工具失败原稿/原流保留；续验复用成功native、3个CLI前缀和130预算原流，不称首次七门全绿。剩余ordinary完整路由/no-body/constructor/class-scope、bridge/needle与实际read/IR/资源/停止门见[next-boundaries](results/s1-generic-body-declaration-v1/next-boundaries.md)；八项既有失败不在本轮分母，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

[普通参数化声明策略迁移](results/s1-ordinary-parameterized-policy-v1/README.md)、[收据](results/s1-ordinary-parameterized-policy-v1/acceptance-r1.json)（2026-10-24）：Java接管完整ordinary准入/返回来源/throws/声明与proof分类及exact-null选择，root保留同次事实、物理身份、有序路由、停止和提交；Signature/descriptor/varargs现有实体迁入Java并统一generic解析，无新DTO/read/cache。三生产文件纯迁移，合同34/schema12/解析4/v9保持。六门有效验收：22native1023/0/2，9开发context的27evidence+2预算逐mode源码/方法/execution/coverage/logical usage保持；实际19方法成功/4旧拒绝，Generic族单列。继承body12/130cutoff/name3保持，65compact error完整JSON核对，32个双body完整partial点无半项泛型头；168源码/78声明及当前136×3、核心6编译12受控JVM/真实两类4重编保持，真实目标与变异未执行。2735来源/归档/构建前、23产物与五成功命令同次匹配，target清理；峰值769858021bytes仅是构建护栏。旧CLI误选和name验证器KeyError原流保留；修正后复用成功门与all前缀，不称首次全绿。NoBody误称AST/SSA的旧文案、未覆盖的List参数primitive结果、临时return拼写成本和其余职责/资源门按[next-boundaries](results/s1-ordinary-parameterized-policy-v1/next-boundaries.md)承接；八个旧失败未重测或修复，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。


[NoBody 泛型策略迁移](results/s1-no-body-generic-policy-v1/README.md)、[收据](results/s1-no-body-generic-policy-v1/acceptance-r1.json)（2026-10-24）：Java 接管 NoBody 的层级源码准入、method-local 泛型完整声明和 slot/annotation/varargs/header 拼写，root 保留原 erasure 后预检位置、同次物理事实、CP 费用、调用绑定、停止和提交；没有新 DTO/read/cache。三生产文件与两现有测试增强，恢复 34→35，schema12/解析4/inputs v9 保持。纠正 ordinary NoBody 误称 AST/SSA 的说明，新旧均39字节，原 method-local 说明保持。有效六门：22 native 1023 passed/0 failed/2既有ignored；7上下文21报告、2实际预算stop与1零值输入拒绝保持；继承body12/130cutoff/name3，65 compact error完整JSON和32原子partial点通过；168全文/78声明保持。当前136×3对Z完整JSON除物理NoBody说明、实际engine tuple和usage.elapsed外相等；20上下文72次物理marker，其中16上下文48份顶层文本改变，声明/raw/费用/limits/来源/coverage保持。当前核心6编译12受控JVM、真实两类4重编通过，真实目标与变异未执行。2735来源/构建前/归档、23产物及五成功命令同次绑定，target清理；770145514bytes仅为构建磁盘护栏。首core driver全文断言exit1与合同preflight字段错误均保留，R2复用已成功原流离线验收，没有重复CLI/JDK或重跑native；不称首次全绿。后续构造器、family source policy、member descriptor与普通声明拼写及bridge/raw/资源退出门见后续清单；八项旧失败不在本次分母，2.9/2.10/S1仍未验收，OpenSpec17/59，S2未进入。

后继职责按 [next-boundaries](results/s1-no-body-generic-policy-v1/next-boundaries.md) 的真实消费者与事实/策略界面推进；已迁为薄适配的 generic 函数不重复计为 root 策略。


[泛型构造器源码策略迁移](results/s1-generic-constructor-policy-v1/README.md)、[收据](results/s1-generic-constructor-policy-v1/acceptance-r1.json)（2026-10-24，AB）：Java接管class-scope构造器选择、完整准入、candidate/body-proof回退、InitRecord/参数槽/作用域/字段写入证明、raw字段唯一匹配及声明/proof分类，root保留同次facts、subject身份、erasure/CP/同类调用与委托物理门、停止和完整提交。两生产文件纯迁移，无新DTO/read/cache/test函数/产品资产，恢复35/schema12/解析4/inputs v9保持。有效六门通过：24选定native1034 passed/0 failed/2既有ignored；80上下文240报告与8非零预算边界全文、marker、声明、物理身份、费用/limits、来源/coverage保持；52上下文有成功构造器投影，28没有，RawNewHold旧Partial保留。NoBody24/body12/130cutoff/name3、65compact error与32原子partial点、168全文/78声明保持；当前136×3全文及408完整JSON除已验证engine与usage耗时外匹配，核心6编译12受控JVM/真实两类4重编通过，真实目标未执行。2735来源/归档/构建前、25产物及五成功命令同次匹配，target清理。六份旧GenericConstructor缓存报告与重新生成的JAR身份错配已保留原流，以现存冻结JAR补采AA旧6份，当前235前缀复用+5续跑，不改snapshot、不重建或重跑成功native/JDK，不称首次全绿。subject纯比较求值提前的实际CPU成本归2.10；剩余family/descriptor/普通声明、bridge/raw与资源门及八旧失败继续承接，2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

下一步按[后续职责清单](results/s1-generic-constructor-policy-v1/next-boundaries.md)迁移已选family的源码策略，保留root物理关系/候选/停止/事务；已迁移构造器不再计为root策略残留。


### S1 finite exit checklist（2026-10-26 修订）

以下任务取代旧 2.10 的开放式总收尾；上方与后附带日期的收据保留原门和失败，不自动满足新任务。范围、原 S0 资源行归属和 E7 分组见 design D12。

- [x] 2.11 依赖 2.9–2.10，核对当前有效成功集及八项历史 native 失败；验证 **A1.1、A1.4–A1.6**：136×3 核心、完整类/来源和既有正负例不无解释退化；每个旧失败绑定原名/输入/实际基线，产品错若破坏有限边界必须修复，过时断言修订有独立证据，后续域缺口保持失败并指明归属。

2.11当前收据：[2469完整报告逐项审查](results/architecture-completion-v1/report-retention-reviewed.json)。2361份在验证版本/实现身份与usage.elapsed后完整相等；108份不是全JSON相等：10份新增实际quota、72份序列化Error.requested增加29、26份R06必要字段证明计费。隔离三行反转把26份计费全部还原CG；真实task交付check前trace确认72份新增字段各29字节及Error.requested等于实际被拒绝文档长度。row171独立debug probe比正式release另多9字节，保留未归因，不冒称历史隐藏文档全等。正式可观察错误仅requested变化，无正文/状态/身份/coverage或退出码未审查差异。408核心完整JSON保持，6编译12受控JVM和真实两类4重编通过；八旧失败逐名/输入/实际基线分类，五项修订有独立合同证据，WA main三项仍失败并按D13/S2a/S3a承接，不改成通过。
- [x] 2.12 先于本项产品运行冻结原 S0 全配置到 S1 或 E2/S6 的逐行映射，再完成 S1 适用资源门；验证 **A0.3、A1.7**：按原公式/配置各 10 次交错核对实际 read/Body/IR、live/peak、时间/RSS 及取消，方法/类范围旧时间超门须实际修复；bulk 原失败与分母完整保留，不按结果选择范围或调阈值，被 S1 修改/依赖的 bulk 行仍阻塞。

2.12当前收据：[原门槛30/30时间与30/30RSS](results/architecture-completion-v1/resources-clock-cost/gate-review.json)、[300对实际工作/计费/留存与100 query窗](results/architecture-completion-v1/resources-clock-cost/logical-review.json)。原47行已在首次运行前固定为30 S1/17 E2-S6，旧S0 producer逐字节重建且SHA一致。初探及两个失败批次完整保留；实际采样定位Budget成功操作末尾冗余取时，删除三处未消费elapsed缓存写入，计费/检查点不变，Reader183及ledger/公开停止控制通过。新产物正式600进程、每配置10轮；按S0未清OS缓存条件，在批次外记录共同60进程准备，正式样本仍fresh process/FactsCache、原轮转顺序/预算/evidence/阈值/guards；不承诺冷机/首次执行时延。纯Clock优化前后300对所有非elapsed工作/计费/留存逐项相同，2469完整报告及408核心完整报告保持；原17个bulk行、10个WAR RSS失败及bulk token/sink/panic门归属不改。共同准备与失败原流边界见[独立复核](results/architecture-completion-v1/finite-exit-independent-review.json)，不以准备样本替换正式样本，不宣称原47行全通过。
- [x] 2.13 依赖 2.9–2.11，同步本次边界涉及的库/CLI/示例/API 说明并移除会分叉语义的临时路径；验证 **A1.2、A1.5**：源码目标/环境/输入来源/停止/版本一致；仅供转发的适配不当成另一套语义，不为了 root 清零扩大范围。

2.13当前收据：[API/文档核对](results/architecture-completion-v1/api-doc-review.json)、[完整workspace类型检查](results/architecture-completion-v1/workspace-current-check-r3.log)、[源码目标库/CLI实际对照](results/architecture-completion-v1/api-final-commands.json)。预算维度/新增quota的全部测试模式适配完成，相关两批25/25与121/121通过；源码目标库/薄CLI控制通过，examples随all-targets检查。普通root294与观测root310分别验证。删除会丢原Report的Result-only owner扫描包装，实际消费者共享原DependencyProofResult；仅转发/纯归属清理保留E7。README当前binding说明由陈旧4改为12，停止及恢复50/解析5/inputs9说明一致；历史带日期收据不重写。2.12时间门仍未通过，2.14/S1未验收、S2未进入。
- [x] 2.14 依赖 2.9–2.13，按 design D12 / **A1.7** 一次汇总有限退出；验证：A1.1–A1.6 证据和适用资源全部实际通过，E1–E7 失败/未覆盖及 S2a 输入可追溯，既有勾选和 S0 原记录保持；达到本项即停止 S1 扩张并转 S2a，不因规划更新提前勾选。

S1有限退出汇总：[acceptance.json](results/architecture-completion-v1/acceptance.json)、[当前审查](results/architecture-completion-v1/finite-exit-review.md)。2.9–2.13全部有实际证据，A1.1–A1.6原验收与本轮有限闭包对应，A1.7适用30行通过；不是整项目或E1–E7全部完成。三项WA整类/main缺口仍失败；SYN01待最终架构后复核，S2a/S3a机制仍须实现。独立审查初稿误用历史17/65和旧false状态的错误已封存、更正；未增加新的raw/stop阻塞。进度23/65，下一项3.1；停止S1职责迁移扩张，转S2a既有声明/SSA规划。

## 3. S2 — 共享源码类型与变量合法化

前置：S1 的有限退出 2.14 / A1.7 实际通过。**先做 S2a：3.1 → 3.2/3.3 → 3.6 → 3.7 → 3.9**，两族为分支合流/生命周期及循环边复制/handler 可见值。S2b 的 3.4–3.5/3.8 保留泛型扩展验收，不阻塞已限定类型域内的 S3a；整个 S2 未完成前不进入 S4。

- [x] 3.1 在现有声明规划中分开槽、SSA 值与源码变量分组；验证 **A2.1**：互斥生命周期独立声明，宽/窄槽复用有使用边界，跨界使用与缺入边保持拒绝。

3.1当前验收见[槽/SSA/源码变量分组收据](results/s2a-variable-legalization-v1/task-3.1-receipt.json)：多类别与互斥分支按SSA归属分组，debug只附命名线索；存活跨界别名、未证明入边与异常边保持拒绝。四开发配置完整类原样编译/隔离行为通过，Java598及直接正负控制通过。S2a其余类型/边值与资源、holdout门继续开放，不借本项宣布阶段完成。

- [x] 3.2 依赖 3.1，汇总引用赋值、全部使用、null/引用 phi 和数组约束，接入有界队列；验证 **A2.1**：分支遍历顺序不改类型，全部写入/使用被核对，boolean 0/1 与 char/窄化边界保留，类型规划停止后不构建语句。
- [x] 3.3 依赖 3.1，统一合法化所需的完整环境绑定 assignability、平台契约及最终声明的既有调用/重载约束；验证 **A2.2**：真实输入 Header 优先，缺失/不兼容/歧义分开，父 roots 改变不复用旧结论，同名不同 loader 不合并。
3.3 静态复核输入：[2.6 对抗复核](results/s1-environment-v1/adversarial-review.md)发现平台契约仅检查 caller uncertainty；非 caller domain 的 uncertainty 已在3.3按实际供给复现错误正例并修复，证据见[非caller复核](results/s2a-variable-legalization-v1/C/noncaller-platform-uncertainty-r39/fixed-review.json)。2.6历史静态风险记录保持，不倒写为当时已复现。

- [x] 3.6 依赖 3.1–3.2，统一声明支配、definite assignment、catch/循环/捕获作用域；验证 **A2.4**：各到达路径可合法使用，debug/no-debug 只影响命名线索，不以默认初始化或拆名掩盖未赋值/跨界使用。
- [x] 3.7 依赖 3.3、3.6，核对 phi 所属边赋值、临界边与复制环临时值，保持同时复制及 handler 可见值；验证 **A2.4**：循环交换值、多前驱/兄弟边无串写，throw 前局部与 catch 入口分开；完整类编译/行为一致，缺前驱或不兼容使用继续拒绝。
- [x] 3.9 依赖 3.1–3.3、3.6–3.7，验收 **S2a / A2.5**；验证：D12 两问题族、负例/停止、独立保留集和至少一个固定真实包有净恢复收益，完整类原样编译/隔离行为及适用资源门通过；当前调用目标不因声明改变，未交付泛型能力仍拒绝，不等待 3.4–3.5。

### S2b — declared generic expansion

前置：S2a 实际通过；完整 S2 仍须完成本组。

- [ ] 3.4 依赖 3.2–3.3、3.9，沿已校验继承路径传播已声明泛型实参；验证 **A2.1**：平台与自定义容器共同覆盖字段/局部/参数/返回，不支持 capture/推断保留原因，不靠类名放行。
- [ ] 3.5 依赖 3.4，让调用参数、必要 cast 与重载检查消费最终声明；验证 **A2.2**：实际调用目标与原 class 相符，缺证不靠任意 cast/擦除过编译，完整类原样编译并受控对照。
- [ ] 3.8 依赖 3.1–3.7、3.9，执行两族矩阵、holdout、固定包和资源对照，移除被替代的重复判据；验证 **A2.3–A2.4**：至少一个固定包净恢复收益、变量合法化与保留集收益成立，已知错误修正和有效成功退化分开，适用 E1/E6 门满足。

## 4. S3 — 表达式效果、顺序基线与原子转换

观测承接：raw-member-paths 检查点的 varargs R0 在成员构造参数 `mark(value + rest.length)` 上，旧 R3 两例被 `jre_new_member_order` 拒绝；当前 R4 首例相同拒绝后停止，未完成第二例或生成编译。它是既有成员参数组合覆盖缺口，原流见 results/s1-raw-member-paths-v1/varargs-baseline 与 varargs-product-r4；不放宽当前 S1 守卫，后继由本阶段的参数效果/顺序组合任务分析。该变体已用于定位，不能充当独立 holdout。

前置：S2a / 3.9 / A2.5 实际通过，消费其声明、类型、边赋值与既有调用决定。**先做 S3a：4.1 → 4.2/4.5 → 4.6 → 4.8**；不等待 S2b 或 S3b。两问题族为局部更新/赋值多消费，以及字段/数组左值/RHS/普通调用参数顺序；未证明泛型/生命周期域保持边界。

- [x] 4.1 依赖 3.9，在既有 AST builder 为两族建立有类型顺序语句/临时值基线；验证 **A3.1**：局部旧值/更新及字段/数组接收者、索引、RHS、调用参数各按原次数/左到右顺序执行，短路/三元操作留在原分支，必要窄化和 handler 归属不变，完整类原样编译/执行；不先追求糖化。
- [x] 4.2 依赖 4.1，候选共同检查全部消费点、吸收效果、控制/异常归属与 origins 后提交；验证 **A3.1**：拒绝后基线不变，无重复/遗失效果，原整方法守卫保持直到等效验证。
- [x] 4.5 依赖 4.1–4.2，先核顺序基线涉及的异常与数值边界；验证 **A3.4**：null/越界/cast/调用各抛错位置、窄化/溢出及适用的装箱/浮点轨迹一致，类初始化和特殊状态保留边界，oracle 核对必要 cause/suppressed。
- [x] 4.6 依赖 4.2、4.5，增加到达 Java 阶段的有界 recovery fuzz；验证 **A3.3**：实际进入阶段数单列，无 panic、丢效果/来源、越预算或假 Complete，不执行变异目标。
- [ ] 4.8 依赖 4.1–4.2、4.5–4.6，验收 **S3a / A3.5**；验证：关闭或拒绝糖化仍有可用完整顺序表示，两问题族/负例/独立保留集及至少一个固定真实包有净恢复收益；首个异常、效果轨迹、来源、预算/取消和适用资源门通过，不等待 4.3–4.4，不宣称整个 S3 完成。

4.1/4.2/4.5/4.6组件证据见[当前收据](results/s3a-ordered-baseline-v1/component-receipt-r4.json)。原型完整类开发24编译/48隔离运行通过，另有NestedEval完整类公开回归；独立保留集、完整报告保留与资源/来源审查继续，不代替4.8验收。旧拒绝升级和wrapper误用的原流/边界均保留。


### S3b — expression sugar

前置：S3a 实际通过；涉及新增泛型类型决定时同时依赖 S2b 对应任务实际通过。

- [ ] 4.3 依赖 4.8，迁移前后缀/赋值值糖化与消费者失败回退；验证 **A3.2**：多个使用点保留旧/新值，消费失败不丢更新，未提交内容不外泄，全局停止不再发射基线。
- [ ] 4.4 依赖 4.8，在已验收顺序基线上糖化字段/数组左值与普通调用参数组合；验证 **A3.2**：地址不重复求值，短路操作不提前执行，不移入另一 handler，普通规则不放宽 volatile/monitor/未初始化对象。
- [ ] 4.7 依赖 4.1–4.6、4.8，执行双问题族、holdout、真实包与停止对照，清理重复效果判据；验证 **A3.1–A3.4**：合法顺序基线可用覆盖增加，拒绝糖化不丢有效表示，真实包净收益与 S0 资源门通过。

## 5. S4 — 区域归属与出口组合

前置：S2、S3 全部验收通过（含 S2b / 3.8 与 S3b / 4.7），继续消费完整变量与效果合同；不以 3.9/4.8 的局部里程碑代替。问题族：循环体内分支/catch；catch 保护体/处理体内循环及非局部退出。现有 finally/TWR/monitor 作为回归边界。

- [ ] 5.1 复用 owner/continuation 整理入口、拥有块、出口、throw site/handler 和跨边界值；验证 **A4.1**：与 CFG/SSA 对账，无遗漏/重叠拥有，出口与跨边界声明/边复制可证明。
- [ ] 5.2 依赖 5.1，将可约循环/分支组合迁入共同合同；验证 **A4.1**：嵌套 break/continue/return/throw 保持目标及先行效果，前后语句/变量变形不依赖固定布局。
- [ ] 5.3 依赖 5.1–5.2，将闭合普通 named catch 与循环组合接入共同检查；验证 **A4.2**：两问题族恢复，handler 顺序和 throw site 保护归属不变，外来入口、交叠保护、未决出口与重复归属拒绝。
- [ ] 5.4 依赖 5.2–5.3，收拢试探回退并按接受域替换固定形状判据、复用边索引；验证 **A4.2**：列出实际替代规则与命中证据，失败不污染 owner/visited，合法 BCI 来源保留，不以删除数组数量充当泛化收益。
- [ ] 5.5 依赖 5.4，核对既有 finally/TWR/monitor、legacy 与来源回归；验证 **A4.3**：return/throw 覆盖、清理与适用 suppressed/monitor 行为不退化，未迁移域仍有边界，旧 goto 缺口独立保留。
- [ ] 5.6 依赖 5.1–5.5，执行组合矩阵、holdout、真实包、大图/深嵌套与取消测量；验证 **A4.1–A4.3**：净收益成立，默认栈可报告停止，实际时间/RSS/取消满足 S0，逻辑 AnalysisSteps 与实际成本分列，适用 E1/E6 门满足。

## 6. S5 — 类级声明、使用与来源共同提交

前置：S2、S3、S4 全部验收通过；同时依赖 S1 身份与绑定。问题族：泛型声明及其调用；synthetic/匿名类族与初始化。验收范围为声明的使用闭包，不声称未知 ABI/反射/序列化全部等价。

- [ ] 6.1 按候选建立有限修改计划并检测交叠修改/共享隐藏成员冲突；验证 **A5.1**：关联声明、全部相关使用、效果与 origins，不启动无理由全包 Body，原始物理事实保持可用。
- [ ] 6.2 依赖 6.1，让泛型声明与受影响调用/赋值共同检查和提交；验证 **A5.1**：完整类类型/实际调用目标相符，任一相关使用失败无半项改写。
- [ ] 6.3 依赖 6.1，将 accessor/bridge 隐藏接入使用闭包与可观察域判定；验证 **A5.2**：外部可见/未知使用者、依赖缺失或扫描停止不当成零使用，保留成员或明确受限输出，原始查询关系不消失。
- [ ] 6.4 依赖 6.1，多创建点匿名/成员类投影共同核对捕获与构造顺序；验证 **A5.2**：第二创建点失败撤销整项投影，各点保留有效原表示或缺口，不只隐藏声明。
- [ ] 6.5 依赖 6.2–6.4，字段初始化提取消费效果与声明合同；验证 **A5.2**：静态初始化、构造链、可见写入与抛错顺序一致，不能跨越可观察效果移动。
- [ ] 6.6 依赖 6.2–6.5，验证单候选事务和独立已提交前缀；验证 **A5.1–A5.3**：同项任一失败无半提交，独立成员失败允许保留先前成果并标 partial，全局 stop 后不读新 Body 或重建整类。
- [ ] 6.7 依赖 6.1–6.6，复用方法 map/derived spans 补齐整类来源交付；验证 **A5.4**：文本摘要与 UTF-8 byte 半开区间匹配，metadata/方法 BCI/多物理来源可回溯，重复片段/缩进/非 ASCII/非 BMP/多轮替换不串坐标，缺口与映射停止显式，可选 map 不改正文。
- [ ] 6.8 依赖 6.6–6.7，增加到达类级投影的有界 fuzz 与停止探针；验证 **A5.3–A5.4**：报告实际进入阶段数，无冲突半提交、越界来源、目标执行、越预算或假 Complete。
- [ ] 6.9 依赖 6.1–6.8，执行两族矩阵、holdout、真实包完整类/资源对照并同步类级架构；验证 **A5.1–A5.4**：完整类净收益、初始化/异常行为和来源成立，等价上下文入口决定一致，独立 E6 来源债务不冒称已修。

6.9 既有失败输入补充：[S1 anonymous同源super矩阵](results/s1-raw-anonymous-v1/README.md)保存8组javac8在anonymous_super_child_shape_unproved处拒绝，与已通过的javac23控制分列。先核对完整shape条件及可观察域，再在S5共同投影机制中决定覆盖；本次raw迁移不放宽该守卫，不把此失败归给名称解码。

## 7. S6 — 显式查询、解析与源码交付组合

前置：S1、S5 全部验收通过（含 A1.4–A1.6、A5.4）。E2 阻塞批量故障交付验收；E1/E3/E6 按实际消费路径触发。产物保持独立 query 的边界。

- [ ] 7.1 在现有统一入口声明 typed 阶段、谓词、环境、源码交付和预算；验证 **A6.1**：crate 方向不变，独立 query 的 UnsupportedAnalysis 不暗改，未请求源码不引入 SourceTarget 工作。
- [ ] 7.2 依赖 7.1，连接安全结构筛选与声明/有界派发，保留 Unknown；验证 **A6.1**：同分立操作的候选和身份一致，缺依赖/开放世界不生成假 NoMatch 或唯一运行目标。
- [ ] 7.3 依赖 7.2，接入明确选择的源码交付及同次事实复用；验证 **A6.1**：结构查询零 resolver/IR/AST、声明查询零 Region/AST，仅需要的成员及有 reason 依赖分析，隐藏成员仍保留原始物理引用。
- [ ] 7.4 依赖 7.3，实现完整计划/环境/语义版本绑定的分页和全局停止；验证 **A6.2**：只改父 roots/prefix/delegation、依赖 snapshot、词法/投影/source target 或相关语义版本即拒绝不匹配旧游标，改页大小仍合法；分页与无分页结果/来源一致，全局 stop 后零新 Body。
- [ ] 7.5 依赖 7.4，薄 CLI 适配库请求并同步相关配置/版本说明；验证 **A6.2**：同配置库/CLI 一致，逐阶段 coverage、整体 execution、未处理范围与前缀如实保留，适用 E3 格式门通过。
- [ ] 7.6 依赖 7.3–7.5，关联 E2 独立修复的证据并执行组合交付故障门；验证 **A6.2–A6.3**：非 Refused 正常结束有 Prepared，Refused 按自身协议，sink 拒绝不强发；token/budget/sink/panic 分别报告，前置未通过则阶段不得验收，不能在此顺手实现 E2。
- [ ] 7.7 依赖 7.1–7.6，执行重复查询、单方法、类、bulk、慢 sink 及 cache 禁用/冷/热/容量不足对照；验证 **A6.3**：充分预算下语义/身份/来源同分立参照，紧预算下完成范围及成本差异可解释，背压、峰值保留、时间/RSS/token 取消通过 S0 门，架构同步。

## 8. 跨阶段收尾

前置：S0–S6 全部验收通过，适用独立债务门关闭。本轮不执行。

- [ ] 8.1 汇总 design 所有 A0.*–A6.* 验收 ID、审计项、任务和实际证据；验证：输入/命令/产品/完整源码/原流可追溯，已验证收益、拒绝收益、未覆盖和独立债务分别列出，不将规划或 JADX 清点当实现完成。
- [ ] 8.2 依赖 8.1，重读当时主规格后仅同步已验收 delta；验证：MODIFIED 保留期间新增场景、独立平面与原停止合同，strict 通过，旧 change 状态独立，不提前归档。
- [ ] 8.3 依赖 8.2，完成适用工程门和最终交付；验证：受影响回归/集成门通过，无未测新依赖，Rust 构建残留按项目约束清理，明确区分规划、实现、验收与归档。


## S1 泛型类族方法策略检查点（2026-10-24，AC）

[结果](results/s1-member-family-policy-v1/README.md)、[收据](results/s1-member-family-policy-v1/acceptance-r1.json)、[后续边界](results/s1-member-family-policy-v1/next-boundaries.md)。

Java 接管已选泛型类族 method 的三阶段源码策略：属性读取前的准入、parse 后 erasure 前的作用域拒绝、erasure 后的完整 shape/声明与 proof 分类；root 保留同次读取、raw 身份、erasure、物理关系/候选/闭包、停止和原子提交。消除 child 为清 marker 复制整份 record 的临时 clone。两生产文件纯迁移，无新 DTO/read/cache/test 函数/产品资产，恢复35/schema12/解析4/inputs v9保持。

六项局部门禁通过：24 native 1034 passed/0 failed/2既有ignored；19类族输入×三evidence及8非零预算边界共65份报告保持；继承构造器240+8、NoBody24/body12/130cutoff/name3及65 compact error/32原子partial点；168全文/78声明保持；当前136×3与408完整JSON保持，6编译12受控JVM/真实两类4重编通过，真实目标未执行。四组原件/旧生成8 oracle通过，当前4份生成全文相同后复用，无重复JDK。完整JSON仅排除已核验actual engine tuple与usage.elapsed，物理snapshot、正文/marker、limits/费用、来源/coverage及partial范围不变。

类族normal57为45complete/12partial；24份prepared_fold保留原普通回退，effectful_id/method_local_root12份保持partial，extra_child6份家族refused但执行complete。2735来源/归档/构建前与25产物、五成功命令同次绑定，target清理；769748398bytes只作构建磁盘护栏。AB5934payload未改。collector遗漏javac版本stdout的原空字段保留，以采集后元数据补记说明；不补造原流。

剩余has_supported_generic_member_family_header的源码前置选择、member descriptor/普通声明、bridge/raw来源闭包与2.10实际资源/停止/八旧失败继续按清单推进。本局部不证明恢复域收益或实际CPU/RSS改善。2.9/2.10/S1仍未验收，OpenSpec17/59，S2未进入。


## S1 普通方法声明与类族header策略检查点（2026-10-24，AD）

[结果](results/s1-ordinary-method-policy-v1/README.md)、[收据](results/s1-ordinary-method-policy-v1/acceptance-r1.json)、[剩余职责](results/s1-ordinary-method-policy-v1/remaining-source-responsibilities.md)、[后续边界](results/s1-ordinary-method-policy-v1/next-boundaries.md)。

Java承接完整普通method的descriptor源码准入、flags/default方法、ctor/clinit、debug与同次恢复参数名、wide slots/MethodParameters final、varargs/throws/default/type annotation装饰及marker，既有参数事实类型与共享命名/可见性/类型装饰helper迁移后内部重导出。has_supported_generic_member_family_header的源码子域也归Java；root保留物理读取、身份、属性事实/报告、scope可用性、原阶段/预算/停止及完整提交。五生产文件纯迁移，无新DTO/Session/read/cache/产品test函数/资产，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：26 native 1158 passed/0 failed/2既有ignored（新增纳入class_source110与annotation_default14旧测试）；16上下文48complete与8非零预算边界共56份普通声明完整报告保持；家族65/构造器240+8/NoBody24及body12/130cutoff/name3、65compact error/32原子partial点保持；168全文/78声明及当前136×3和408完整JSON保持，6编译12受控JVM/真实两类4重编通过，真实目标未执行。16份当前生成全文相同后复用4组旧生成编译/4定向隔离Driver；数组return维度注解旧拒绝造成四组reflection差异仍明确保留，不称完整反射等价。完整JSON仅排除actual engine与usage.elapsed，physical snapshot/raw/正文/marker/limits/费用/来源/coverage不变。

2735来源/归档/构建前/当前、27产物与五成功命令同次绑定，target清理，769844016bytes仅作构建磁盘护栏；AC5527payload未改。原机制driver在constructor调用前误把历史label当pin，R2又在调用前误比较相对/绝对索引路径表示；原脚本和失败流保留，R3核验80个冻结JAR的实际path/bytes/SHA后只续验后缀，复用decl56/family65，无重复成功native/CLI/JDK，不称首次全绿。

前轮清单误称generic_enclosing_scope_is_eligible，实际源码一直是has_supported_generic_member_family_header，当前规划已更正，AC封存不改。重新审计明确class/field普通声明、class/field Signature准入、annotation位置/重复与package-info仍有源码策略在root，按共享机制继续迁移，不把所有String assembly一概当债务。随后继续2.9 raw/span闭包与2.10 actual read/IR/live/peak/RSS/停止/背压及八旧失败；不据局部门禁关闭S1，进度17/59，S2未进入。

## S1 类与字段源码策略检查点（2026-10-24，AE）

[结果](results/s1-class-field-policy-v1/README.md)、[收据](results/s1-class-field-policy-v1/acceptance-r1.json)、[后续职责](results/s1-class-field-policy-v1/next-boundaries.md)。

Java 接管 class/interface/enum/@interface 的 flags/name/父类型拼写、普通与泛型字段共享的 descriptor/type path、flags/alias/ConstantValue/marker，以及 field Signature 的源码准入。root 保留同次读取、身份、Reader erasure、same-class/SSA writer、CP 闭包、预算/停止、deferred 与原子提交，gate 顺序不变。四生产文件纯迁移，无新增 DTO/Session/cache/read/永久测试函数/产品资产，恢复35/schema12/解析4/inputs v9保持。

六项局部门禁通过：29 native 1173 passed/0 failed/2既有ignored；16上下文48complete与8非零预算报告保持；继承普通声明56、类族65、构造器248、NoBody24、body12/130cutoff/name3及65 compact error/32原子partial点；168全文/78声明保持；当前136×3正文与408完整JSON保持，6编译12受控JVM/真实两类4重编通过，真实目标未执行。完整JSON只排除已核验actual engine与usage.elapsed，snapshot/raw/正文/marker/limits/费用/来源/coverage不变。

四组非枚举ClassField/Iface/Anno原样生成源的隔离编译、反射及ConstantValue通过；当前16份完整源码相同后复用oracle。所选class-source域的FieldEnum四源整体编译仍失败，旧enum常量字段拼写及AD数组return维度注解拒绝均保留，不称完整枚举或反射等价。首partial采集与非枚举汇总器列解析失败保留，首组原流离线复用、后三组续采，exit重建依据和派生元数据纠错明确，不称首次全绿。

2735来源/归档/构建前/当前与30产物、五成功命令同次绑定，target清理；906665720bytes仅作构建磁盘护栏。AD5759payload核验未改。下一组先迁class Signature完整shape/source策略，继续保留物理父定义/nested证明与budgeted callback；随后迁annotation位置/重复与package-info，再完成2.9 raw/span/bridge闭包和2.10 actual read/IR/live/peak/RSS/停止/背压及八旧失败。各组复用直接前继固定输入与完整报告门，不以formatter迁移关闭完整策略。2.9/2.10/S1仍未验收，进度17/59，S2未进入。

## S1 完整 class Signature 源码策略检查点（2026-10-24，AF）

[结果](results/s1-class-signature-policy-v1/README.md)、[收据](results/s1-class-signature-policy-v1/acceptance-r1.json)、[剩余职责](results/s1-class-signature-policy-v1/remaining-source-responsibilities.md)、[后续边界](results/s1-class-signature-policy-v1/next-boundaries.md)。

Java承接class Signature的一次性shape分类、kind/name/type-use准入、interface-only own-header fallback、direct Parent<String>/interface源码决定、type variable/bound/Object省略及完整header拼写；已有父位置/证明枚举迁移并由root内部重导出。root保留属性读取/parse、raw Signature记录时机、nested/physical header与Reader erasure、所选环境父定义查询及原budgeted callback、最后OutputBytes charge和完整提交。两生产文件纯迁移，无新事实DTO/Session/cache/read/永久测试函数/产品资产，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；14上下文42complete+10非零预算报告保持，实际5个header projected/5refused/4无投影；继承class-field56/普通声明56/类族65/构造器248/NoBody24/body12/130cutoff/name3与65compact error/32原子partial点保持；168全文/78声明、当前136×3与408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标未执行。仅规范actual engine与usage.elapsed，snapshot/raw/正文/marker/limits/费用/来源/coverage不变。

原44基线缺direct-parent proved控制，Cargo终态后用既有Child/Parent固定fixture补8报告，原44和JAR不重跑/重建；52行视图分别pin两源。当前16类/字段完整源码相同后复用AE四组非enum编译/反射/ConstantValue，保留首组exit控制流重建的原证据层级。FieldEnum所选域四源编译失败、数组return维度注解旧拒绝均保留，不称恢复域增益或全反射等价。self-test、R1机制/核心只读入口的导出/字段错误及静态mode不符保留；R2独立副本修正，原native九工具不改，无成功native/CLI/JDK重跑，不称首次全绿。

2735来源/归档/构建前/当前与30产物、五成功命令同次绑定，target清理；906653750bytes仅作构建磁盘护栏，AE5980payload未改。下一组先迁annotation现有类型所有权、重复/参数对齐/target/path和field/method实际源码位置，再迁package-info窄证书；root留Reader事实/停止/报告事务，Java接管源码准入，不能只搬formatter。其后完成2.9 raw/bridge/span闭包与2.10 actual read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败的原合同。2.9/2.10/S1仍未验收，17/59，S2未进入。

## S1 annotation 位置与重复策略检查点（2026-10-24，AG）

[结果](results/s1-annotation-position-policy-v1/README.md)、[收据](results/s1-annotation-position-policy-v1/acceptance-r1.json)、[覆盖范围](results/s1-annotation-position-policy-v1/coverage-scope.md)、[后续边界](results/s1-annotation-position-policy-v1/next-boundaries.md)。

Java接管八个既有annotation报告类型、raw重复域、descriptor参数对齐、owner/target/info/path/冲突与拼写、field实际类型放置和method的clinit/无声明拒绝；root保留Reader shells/facts、计费、错误/停止、报告装配和发布。四生产文件纯迁移，无新DTO/Session/cache/read/永久测试函数/产品资产，2735来源，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；13注解上下文×3evidence的39份完整报告保持（36complete/exit0、Holder3份旧partial/exit4）；继承classSignature52/classField56/ordinary56/family65/constructor248/NoBody24及body12/130cutoff/name3、65compact error/32双body原子partial点保持；168全文/78声明、当前136×3及408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标及字节变异未执行。仅规范核验过的actual engine tuple与usage.elapsed，正文/marker/raw与physical snapshot/limits/费用/coverage/来源保持。

39份CLI矩阵不是完整畸形输入分母：类级重复与非法element name等由既有native补充，任意畸形info未穷尽。Holder记录static声明与旧anonymous nesting partial，空注解列表不能冒充拒绝收益。AE FieldEnum四源整体编译失败、AD数组return维度注解旧拒绝，以及八项既有失败继续保留；本轮无恢复域或实际资源性能收益声明。

2735来源/构建前/归档/当前与30产物、五成功命令同次绑定，target已清理；907163740bytes为构建磁盘采样峰值，仅用于构建守卫。AF6146冻结payload及精确目录清单核验未改。工具在native前补强真实input/process绑定并改为只读复用408保存报告；native后、产品调用前又发现跨evidence误用全JSON比较，以resume-r2复用AF原outline检查核心，并继续由独立408同evidence完整JSON门严格验收。原10工具保持native AST pin，修订前preflight保留，没有因修正重复CLI或冒充产品收益。

下一组独立迁package-info窄源码证书，保持Java8/flags/空members/单空Deprecated及原拒绝次序；随后继续2.9 raw/bridge/span真实来源闭包和2.10 actual read/Body/IR/live/peak/RSS/取消/停止/背压及八旧失败的原合同。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 package-info 源码证书检查点（2026-10-24，AH）

[结果](results/s1-package-info-policy-v1/README.md)、[收据](results/s1-package-info-policy-v1/acceptance-r1.json)、[覆盖分布](results/s1-package-info-policy-v1/coverage-scope.md)、[后续边界](results/s1-package-info-policy-v1/next-boundaries.md)。

Java接管package-info窄源码证书与raw JVM包名合法性，按原顺序检查版本/flags/header/空成员、完整执行、属性/注解和单空Deprecated，再拼写package文本。root仅供给既有事实/切片，保持触发、Reader、预算/停止/diagnostics、物理fallback和完整提交。三生产文件纯迁移，无新DTO/read/cache/永久测试函数/产品资产；2735来源，恢复35/schema12/解析4/inputs v9保持。

六项有效门通过：29 native1173 passed/0 failed/2既有ignored；package-info36条同模式完整报告保持（31exit0、4exit4、1exit2）；继承annotation39、classSignature52/classField56/ordinary56/family65/constructor248/NoBody24、body12/130cutoff/name3及65compact/32双body原子partial保持；168全文/78声明、136×3与408完整JSON保持，核心6编译12受控JVM/真实两类4重编通过，真实目标和字节变异未执行。只规范核验过的actual engine tuple与usage.elapsed，physical/raw/marker/limits/费用/coverage/来源保持。

36行是11上下文×三evidence加3预算观察：4行成功投影、27行证书拒绝、4行前置partial和1行compact输出错误；class_headers=1是成功控制，不称停止。截断annotation只证明Reader/完整执行门。另3条误改魔数的早期探针及原流独立保留，不计源码证书分母；不冒充合法包名反例。 合法fixture原始/生成源在JDK8和23下各编译并运行，共4编译4JVM，输出均true；没有compiler/runtime笛卡尔积。当前三个正常evidence及成功预算控制全文相同后复用该oracle，未重复JDK；生成侧只有2编译2JVM，其余2编译2JVM是原始侧。

2735源码/构建前/归档/当前与30产物、五成功命令同次绑定，精确target已清理；907193463bytes仅为构建磁盘采样峰值。AG6332冻结payload和精确目录清单核验未改。12个工具AST与离线preflight绑定；跨evidence核心/正文检查与同evidence408严格完整报告检查分开，两个保存流复核均不再调用CLI。

证书仍只接受Java8固定shape和单空Deprecated；FieldEnum整体编译、数组return维度注解、Holder匿名嵌套partial与八项旧失败继续保留。本轮不声称恢复域扩大、完整来源map或实际资源性能收益。

下一组按[raw消费者清点](results/s1-package-info-policy-v1/remaining-raw-consumers-audit.md)先复现完整入口，再统一实际raw到source消费；generic static/nested family、父定义/override和widening的静态可疑点不冒充已复现收益。span同run/BCI/唯一性属S1边界，整类map与编辑后重定位另归S5。继续2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压和八旧失败的原合同。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 类族 raw→source 名称边界检查点（2026-10-24，AI）

[结果](results/s1-family-source-name-boundary-v1/README.md)、[有效收据](results/s1-family-source-name-boundary-v1/acceptance-r2.json)、[覆盖边界](results/s1-family-source-name-boundary-v1/coverage-scope.md)、[后续职责](results/s1-family-source-name-boundary-v1/next-boundaries.md)。

generic static、nested enum、nested annotation 和 common fold 的实际名称消费者收拢至既有 Java 严格命名出口；descriptor/CP/Code 身份保持 raw，嵌套路径仅转换已证明的 InnerClasses 边，保留标识符内字面$。两个生产文件、两个已有测试增强，无新DTO/read/cache/test函数/产品资产；2735来源保持，恢复35→36，schema12/解析4/inputs v9不变。既有A<T>/compareTo切片、bridge/闭包、完整执行、BCI/span、预算/停止与原子提交不放宽。

六项有效门通过：30 native1181 passed/0 failed/2既有ignored；十二开发上下文三evidence共36当前报告，四个名称拒绝上下文恢复、四旧有效控制保持、四JDK8形状拒绝保持。当前24 projected、6 generic JDK8 prepared/refused、6 enum JDK8 family refused，均exit0；不是退出码通过率或真实包整体恢复率。八组完整生成源码原样编译/隔离执行成功，原始六组Driver复用、两组fold原始Driver补验，生成classpath无原目标后备。继承六cohort501完整JSON、annotation39/package36、body12/budget130/name3、65compact/32双body原子partial；168全文/78声明、136×3与408同evidence完整JSON保持，核心6编译12受控VM/真实两类4重编通过，真实目标和字节变异不执行。

R1脚本误要求essential公开SourceMap，且两fold原始Driver因文件名编译失败；原流/原21工具与弱版续验保留。最终离线复核使用同context已保存source_map peer，MethodPoint仅来自physical recovery source-map段，物理ID取冻结旧事实；旧JSON缺少的两个Minor ID另用2次旧AH CLI只读查询补齐（contract35，单列不并36恢复分母）。8生成oracle和36当前恢复全部复用，只补失败原始Driver，没有重跑成功native/CLI/JDK。新增验证器对完整报告的伪造digest/BCI/span拒绝，不允许derived自证；不得称首次全绿。

五构建命令同次成功，2735来源/归档/构建前与31产物绑定，精确target清理；907183333bytes仅是构建磁盘采样峰值。AH6575冻结payload及目录清单未变。下一组迁已选泛型父定义读取后的源码资格至既有Java module，再按原阶段迁直接Override方法资格，root保持解析/读取/费用/停止/提交。旧UTF8 gate的语义修复独立复现；enum anonymous-body、不同raw widening、integer constant候选和2.10实际read/IR/live/peak/RSS/取消/停止/背压及八旧失败继续承接。整类map/重定位归S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 已选泛型父定义源码资格检查点（2026-10-24，AJ）

[结果](results/s1-selected-parent-policy-v1/README.md)、[有效收据](results/s1-selected-parent-policy-v1/acceptance-r1.json)、[覆盖边界](results/s1-selected-parent-policy-v1/coverage-scope.md)、[后续职责](results/s1-selected-parent-policy-v1/next-boundaries.md)。

Java接管已选直接泛型父类和接口读取后的源码资格、Signature形状/arity及原预算化解析；门面保留环境/loader选择、实际读取、SingleClass读前短路、缺定义平台fallback、execution/停止合并与提交。父类bool和接口三态不合并，接口仍只证明现有kind/arity等窄条件，不声称bounds/实参与bound或interface erasure证明。两生产文件纯迁移，无新DTO/Session/read/cache/永久测试函数/产品资产；2735来源，恢复36/schema12/解析4/inputs v9保持。

六项有效局部门通过：30 native1181 passed/0 failed/2既有ignored；14上下文三evidence42份完整JSON保持（39exit0、3预算停止exit4）；继承六cohort501/annotation39/package36/AI family36共612份完整JSON，body12/budget130/name3、65compact/32原子partial保持；168全文/78声明，136×3与408同evidence完整JSON和跨evidence outline保持，核心6编译12受控VM/真实两类4重编通过。AI八份生成源码相同后复用隔离oracle，未重复家族JDK，真实目标及变异未执行。仅规范已绑定实际构建的engine tuple和usage.elapsed，raw/physical/正文/marker/limits/费用/coverage/来源/partial范围保持。

R1执行分支误用contract35 resolver，在第一份实际CLI后拒绝真实AI36；原16工具/失败流不改，R2离线复核并复用000后续验，无成功Cargo/native/该次CLI重跑。门后独立核验42selected与36family报告的非空engine、原流/进程/输入和完整JSON，闭合selected原工具只查tuple互相一致的缺口。基线三份最初SingleClass参数错误原流因路径覆盖丢失，excluded为原argv实际补采，明确不入42分母。

五构建命令成功，2735来源/归档/构建前/当前与31产物绑定，精确target清理；907426558bytes只作构建磁盘采样守卫。AI6976冻结payload与精确目录清单未改。父类变体在erasure前shape拒绝、接口结构变体在更早原始检查拒绝，不充后续helper覆盖；CLI无token取消入口，native只证明既有回调取消/投影原子性。

下一组按原候选阶段迁直接Override源码资格，root保留same-package前筛选、raw唯一配对、物理来源、读取/费用/停止及偏移事务。旧父定义UTF8 gate修复另经完整入口复现；widening、enum anonymous-body、integer constant、raw/BCI/span可信闭包及2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压与八旧失败继续承接。整类map/重定位属S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 直接 Override 源码资格检查点（2026-10-24，AK）

[结果](results/s1-direct-override-policy-v1/README.md)、[有效收据](results/s1-direct-override-policy-v1/acceptance-r1.json)、[覆盖边界](results/s1-direct-override-policy-v1/coverage-scope.md)、[后续职责](results/s1-direct-override-policy-v1/next-boundaries.md)。

Java declaration接管直接Override的child class flags/scope、parent class flags/scope、child method普通flags/name/Signature/已有声明与注解/正文/descriptor以及parent method资格/visibility。root保留环境/loader、读取/完整性、raw index/name/descriptor/flags绑定、同origin/同包、唯一配对、原计费/停止、唯一offset与output charge后的原子提交。child资格在父表扫描费前，parent资格在唯一匹配及计费后；descriptor是已证明与报告相等的同一raw member。两生产文件纯迁移，无新DTO/Session/read/cache/永久测试函数/产品资产；2735来源，恢复36/schema12/解析4/inputs v9保持。

六项有效局部门通过：30 native1181 passed/0 failed/2既有ignored；16上下文三evidence48份完整JSON保持（42exit0、3compact exit2、3partial exit4），三成功上下文的9份Override proof保持，11无提示上下文保持。继承六cohort501/annotation39/package36/family36/selected-parent42共654完整JSON、body12/budget130/name3、65compact/32原子partial保持；168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持，核心6编译12受控VM/真实两类4重编通过。family同字节后复用8生成隔离oracle，未重复家族JDK；真实目标与变异未执行。仅规范已核验actual engine与usage.elapsed，raw/physical/正文/marker/limits/费用/coverage/来源/partial范围保持。

R1 AA wrapper旧变量名在产品调用前失败；R2复用六组501报告并通过三个AA门，随后首annotation查询后因收据pin类型错误停止。原19工具与两轮失败原流保留，R3复用九成功门和首annotation，只跑剩余五门，14项最终有效通过；门后201份父/Override报告逐条核对engine/输入/实际命令/原流/收据及完整JSON。前三份visibility setup误打public class的真实报告另存excluded，修正protected输入另存；不称首次全绿、参数错误或恢复收益。重复工具协议债务[单独记录](results/s1-direct-override-policy-v1/separate-harness-debt.md)，不混入产品迁移。

五成功构建、2735来源/归档/构建前/当前和31产物绑定，精确target清理；907830747bytes只作磁盘采样护栏。AJ6838冻结payload与精确清单未改。预算compact三行无artifact engine，analysis三行在方法恢复期停止，不冒称helper内部覆盖；native精确output charge原子性保留，内部定点取消仍不宣称穷尽。

下一组在实际入口统一父定义与Override的严格名称出口：U+10400方法名的旧无提示已复现，本组不修接受域；合法supplementary/Java7-8 `_`、源码不可表达、损坏MUTF/孤立surrogate分组，保留raw匹配/环境/loader/erasure/Signature/visibility及原预算事务。widening、enum anonymous-body、integer constant、raw/BCI/span可信闭包和2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压、八旧失败继续承接；整类map/重定位归S5。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 父定义与 Override 严格名称边界（2026-10-24，AL）

[结果](results/s1-parent-override-name-boundary-v1/README.md)、[验收收据](results/s1-parent-override-name-boundary-v1/acceptance-r1.json)、[独立名称审查](results/s1-parent-override-name-boundary-v1/name-results-independent-review.md)、[覆盖范围](results/s1-parent-override-name-boundary-v1/coverage-scope.md)、[后续边界](results/s1-parent-override-name-boundary-v1/next-boundaries.md)。

Java层既有方法/类名严格出口接入直接Override与已选父类/接口资格；父定义的单一Object-bound类型参数严格解码，并允许Java7/8的`_`，该变量不在Child发射。全局source_identifier/局部别名策略未改。root环境/loader、实际读取、raw唯一配对、物理来源、flags/arity/erasure、收费/停止和原子提交保持。三生产文件、两测试文件，无新产品helper/DTO/读取/缓存/资产，2735来源路径不变；恢复36→37，schema12/解析4/inputs v9不变。这是名称语义修复，与前继纯职责迁移分别验收。

六项局部验收通过：30 native1182 passed/0 failed/2既有ignored；21上下文三evidence共63份，12个合法名称上下文的36份修复、27份完整JSON保持。三组方法名新增9份精确raw Override proof，九组父类/接口解除header拒绝。新增12次完整生成Child重编、24次driver编译、Java8/23共48条VM命令及24对输出对照通过；生成classpath无原Child，父/接口为声明的原始依赖，观察限于方法反射调用和泛型继承元数据，不声称整包、父定义自身源码或JVM7覆盖。真实目标和字节变异未执行。

654份继承机制及45份旧Override完整JSON保持；body12/budget130/name3专项保持，另核65compact和32个0/2-header原子partial点，不冒称这些专项均为全JSON。168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持；核心6编译12受控VM及真实两类4重编通过。family全文不变后复用8组生成隔离oracle，无新增家族JDK。全JSON保持门仅规范actual engine与usage.elapsed，名称修复的36份另核完整差异及raw/方法正文，不靠删除字段放宽。

旧基线15份方法名负例因generic基底提前拒绝，原流保留为setup，普通无Signature父子类另采；有效63份含9复用和54新查，历史新旧CLI调用69次。名称工具R1误把成功Signature说明注释当拒绝，已结束的31次查询全部复用，R2只新增余下32次；原20工具与失败流保留，不称首次全绿。非法父类名exit2是原始父类声明拼写失败，不是CLI选择参数错误或selected-parent helper覆盖；四字节名称负例是stdout failed ClassSourceReport。

五成功构建、2735来源/归档/构建前/当前与31产物同次绑定，精确target清理；907386668bytes仅为构建磁盘采样护栏，不代表实际read/IR/RSS收益。新名称进入更深的原证明路径，费用变化单列；内部定点取消未穷尽。当前类自身`_`泛型header/变量等静态剩余消费者先做完整入口复现，再闭合名称与相关使用；widening、enum anonymous-body、integer constant及同次raw/方法/BCI/span继续按2.9推进，整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS/取消/停止/背压与八旧失败按原协议承接。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 当前声明与类型变量名称闭包（2026-10-24，AM）

[结果](results/s1-own-generic-name-boundary-v1/README.md)、[收据](results/s1-own-generic-name-boundary-v1/acceptance-r1.json)、[覆盖边界](results/s1-own-generic-name-boundary-v1/coverage-scope.md)、[后续职责](results/s1-own-generic-name-boundary-v1/next-boundaries.md)。

Java层收拢当前声明与类型变量严格名称出口：类头、字段、方法、构造器、throws/数组及class/method同名遮蔽共同支持Java7/8合法`_`；raw scope、erasure、完整使用、预算/停止和提交守卫保持。新增两个names helper，三生产文件/一测试文件变化，未新增DTO/Session/cache/read/永久资产或测试函数；2735来源不变，恢复37→38，schema12/解析4/inputs v9保持。该名称语义修复与前继纯职责迁移分别验收。

六项局部门通过：30 native1182 passed/0 failed/2既有ignored；当前typed126中60预期修复/66完整JSON保持，owner30中12修复/18保持，继承762完整JSON保持。72是报告数，包含目标/evidence同源变体，不作为独立问题族分母。AA body12/budget130/name3子集合同保持，另核65compact/32双body原子partial（header0:31、2:1）；168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持。核心6编译12受控VM/真实两类4重编通过，真实目标和字节变异未执行。

新增18个合法受控程序、34次完整生成源码编译；生成全部输入目标类、空编译classpath、运行仅生成classes与Driver，无原目标或原依赖后备。Java8/23共104条VM命令、68对输出一致，观察GenericDeclaration归属、bounds/数组/throws及字段/构造/普通调用。仅为定向小型程序及两组完整Parent+Child，不外推任意JAR/WAR、任意行为或JVM7执行。family同字节后复用旧oracle，无重复家族JDK。

owner旧采集51条含18条重复argv，初版三份ASCII控制非泛型且原脚本未另存，旧工具无timeout；原流和偏差如实保留。新增typed工具先固定再执行并设置90s超时。offline preflight的evidence字典/选择字符串误判在产品执行前修正，13原工具在native前固定。native summary.scope误留AL标签，实际schema/来源/30二进制为AM，不以标签冒称额外覆盖。

五构建命令与31产物同次绑定，target清理；907408680bytes仅为磁盘采样护栏。AL7380冻结payload与目录清单保持。下一组仍按2.9审查实际源码策略/事实编排边界，widening、enum anonymous-body、integer constant与raw/方法/BCI/span分组推进；整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS、取消/停止/背压与八旧失败继续按原协议承接。2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

## S1 类族调用名称职责迁移（2026-10-24，AN）

[结果](results/s1-member-call-name-policy-v1/README.md)、[收据](results/s1-member-call-name-policy-v1/acceptance-r1.json)、[覆盖范围](results/s1-member-call-name-policy-v1/coverage-scope.md)、[后续职责](results/s1-member-call-name-policy-v1/next-boundaries.md)。

Java source_proof接管类族调用证明的root严格MUTF解码、child源码资格、root既有保守名称资格及dotted拼写；facade保留物理关系/capture/header事实、完整方法/IR、raw new/constructor/BCI、计费/停止和提交。三条拒绝文本与语义优先级不变，没有借纯迁移放宽`_`等名称。两个生产文件、一处既有cfg(test)断言扩展，新增一个纯helper，无新DTO/Session/cache/read/测试函数或永久资产；2735来源不变，恢复38/schema12/解析4/inputs v9保持，Java与facade实际实现指纹更新。dotted String和参数clone分配时机改变，不宣称内存/时间不变。

六项局部门通过：30 native1182/0/2；AM既有918份加新18份完整JAR入口，共936份完整JSON保持（仅规范actual engine与usage.elapsed，raw/正文/费用/coverage/partial/BCI/来源保留）。AA body12/budget130/name3子集门及65compact/32双body原子partial保持；168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持。核心实际6受控编译12VM、真实两类4重编通过，真实目标不执行。AM的34个完整源码编译变体/68对运行输出在text相同和pins有效后复用，本轮不重跑这组JDK，也不将新18份计为行为oracle。

有效R3为Corretto8编译的root+Inner完整JAR，ASCII/supplementary各6份calls proved；单独`_`6份最终prepared_fold而无calls字段，不充narrow调用证明成功或外部拒绝。R1模板guard误判、R2单CLASS缺成员/Outer_错误名称及javac23、工具parent路径修正和初次构建漏参均分别保留，不作为恢复收益。五构建命令/31产物同次绑定，target清理；907388435bytes仅为构建磁盘采样护栏，AM8545冻结payload与精确清单未变。

下一组先冻结static family target末尾名称资格的完整入口及停止前缀，再迁Java；该组child仅要求可严格解码，不能整套复用更强的member-call helper。调用仍位于物理/descriptor/IR/Java recovery证明之后。随后继续widening、enum anonymous-body、integer constant及同次raw/方法/BCI/span；整类map/重定位仍属S5。2.10实际read/Body/IR/live/peak/RSS、各类取消/停止/背压/panic与八旧失败保持原退出门。2.9/2.10/S1未验收，17/59，S2未进入。

## S1 static target 三段源码判据迁移（2026-10-25，AO）

[结果](results/s1-static-target-policy-v1/README.md)、[收据](results/s1-static-target-policy-v1/acceptance-r1.json)、[范围](results/s1-static-target-policy-v1/coverage-scope.md)、[下一组](results/s1-static-target-policy-v1/next-boundaries.md)。

static target的空Object构造器三指令/CP形状、原ClassSourceRecovery资格、root/child名称解码与root路径三个源码判据迁入Java source_proof。root保留物理选择/完整性、实际Header/IR/recover、依赖登记/计费/stop、target装配；各判据仍在原阶段，名称未提前到读取前，static child仍仅需严格解码。早期static/root-child Signature/fields与constructor selector的元数据资格仍未迁完，单列下一组，不宣称整个static函数策略完成。

相对AN，两生产文件及一处既有names cfg(test)断言扩展，新增3纯helper，无DTO/Session/cache/read、新测试函数或永久source资产。2735来源路径不变，恢复38/schema12/解析4/inputs v9保持；实际Java/facade指纹随构建变化。分配生命周期可变，不以纯迁移零文本差异推断性能不变。

六项局部门通过：30 native1182/0/2；前继936加新static24共960完整JSON保持，仅规范actual engine与usage.elapsed。AA body12/budget130/name3子集及65compact/32双generic body原子partial保持；168全文/78声明、136×3与408同evidence完整JSON/跨evidence outline保持；核心实际6受控编译12VM、真实两类4重编通过，真实目标不执行。AM的34源码编译变体/68运行对经AM→AN→AO两跳text/pins核验后复用，未当作本轮新增JDK或新24份的行为oracle。

新24为固定Corretto8的四完整root+Box JAR、release8与SourceTarget7/8×三evidence：12份明确static_proved；_和effectful的12份仅prepared_static、未发capture/calls，不充内部helper切点负例。五成功构建/31产物绑定，target清理；907410424bytes仅为磁盘采样护栏。AN7413封存payload与精确清单不变；继承工具标签依actual schema/source/build核对，不把组名当producer。

下一组按共享元数据源码准入机制批量迁static header资格与integer constant候选，保留读前/读后时序、reader完整性/物理绑定及预算提交；integer supplementary限制仍只是静态风险，先独立复现再修语义。随后继续widening、enum anonymous-body及raw/方法/BCI/span；整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍按原退出门，2.9/2.10/S1未验收，17/59，S2未进入。

## S1 共享元数据源码准入职责迁移（2026-10-25，AP）

[结果](results/s1-metadata-source-policy-v1/README.md)、[收据](results/s1-metadata-source-policy-v1/acceptance-r1.json)、[范围](results/s1-metadata-source-policy-v1/coverage-scope.md)、[下一组验收](results/s1-metadata-source-policy-v1/next-boundaries.md)。

static target的读前static/root Signature、读后child字段/Signature资格和完整表唯一raw构造器selector，以及integer constant的准入和名称先于值去重，成组迁入Java source_proof。facade保留物理读取/关系/identity、字段/方法完整性、CP/IR/recover、逐项计费/poll、停止清空及提交；selector返回原Header引用，mask保留原FieldItem和顺序，不以源码拼写授予物理身份。

相对AO仅两生产文件，Java文件同时新增一个cfg(test)测试函数；新增5纯helper，无DTO/Session/cache/read或永久source inventory资产。2735来源路径、恢复38/schema12/解析4/inputs v9保持，实际Java/facade指纹更新。局部分配与生命周期可变，不从纯迁移推断时间/RSS不变。

六项局部门通过：30 native1183/0/2；继承AO960加新合法90/变异30，共1080份完整JSON保持，仅规范actual engine与usage.elapsed_millis。AA body12/budget130/name3为字段子集，65compact与32个constructor+两个generic body均complete的partial点保持，header0:31/2:1；145份中170条engine identity全匹配。168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；实际6受控编译12VM、真实两类4重编通过，真实目标未执行。AM34完整源码编译变体/68运行对经AM→AN→AO→AP三跳text与pins核验后复用，不计为本轮新增JDK或新120份的行为oracle。

新15合法JAR覆盖static六变体与integer九变体，固定Corretto8、release8、SourceTarget7/8×三evidence。合法90份均exit0；五query-only变异30份中duplicate ctor/truncated table合计12份exit4，是classfile前置拒绝，不充内部source helper定点负例；变异不编译或执行。KEY𐐀有完整字段声明和ConstantValue但0常量名投影，已从静态风险升级为完整入口复现的可读性缺口，不等于整类失败。本轮保持旧std UTF-8准入与display bytes比较域；后续必须同时改严格MUTF名称出口及candidate.field.name.raw()去重key，不能只替换decoder。

R1 ZIP排序误判和首次变异父verifier失败原流保留；R2复用首输入成功编译pins，另14输入各首次编译，15次成功fixture编译均只执行一次。父adapter错误lineage/current checkout比较缺失在freeze前修复；初版与diff保留，11原tools固定为native前AST版本，协议统一独立承接。五构建命令/31产物绑定，target清理，907843864bytes仅为构建磁盘护栏；AO7485封存payload与精确清单保持。

下一组按整型常量raw identity→忠实源码名称→switch/return/int[]消费者闭环推进：先保存旧完整入口，再同时修准入/raw key；保留声明/flags/I/ConstantValue/完整表/停止清空与anchors，接受域变更恢复38→39，建立新producer绑定，并以完整生成类的隔离编译、反射字段名/flags/value及行为对照验收。malformed/源码不可写/重名/同值/遮蔽/停止分组，变异仍只查询。随后按原阶段继续enum anonymous-body、widening及raw/方法/BCI/span；整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类取消/停止/背压/panic与八旧失败仍按原退出门。2.9/2.10/S1未完成，17/59，S2未进入。

## S1 整型常量 raw 名称恢复闭包（2026-10-25，AQ）

[结果](results/s1-integer-source-name-v1/README.md)、[收据](results/s1-integer-source-name-v1/acceptance-r1.json)、[范围](results/s1-integer-source-name-v1/coverage-scope.md)、[后续](results/s1-integer-source-name-v1/next-boundaries.md)。

Java准入复用既有严格MUTF-8 source_identifier，mask从既有FieldItem传raw key，全部物理字段先去重名、再计候选重复值；源码String不再回编码充物理身份。root保留Integer/声明/static final/I/单ConstantValue、完整field表、读取/预算/poll、停止清空与提交。三个生产文件含恢复版本，扩展一个既有Java单测，无新增helper/DTO/Session/cache/read或永久source路径。恢复38→39，schema12/解析4/inputs v9保持，实际实现指纹更新。

六项局部门通过：30 native1183/0/2；AP1080加新合法42/变异48共1170，1152完整JSON严格保持，18份精确名称/UTF-8 byte spans及原Field/Method/BCI增量通过（AP6由0→2，新12由0→4）。仅规范已验证engine与usage对象直属elapsed_millis；新增AST保留及后续累计预算快照的费用变化逐位置保存，方法级只允许steps/ir_items非负，不能对累计快照求和充实际资源。原比较18失败、R1路径标签TypeError和R2修正都保留，11原tools未改、CLI未重跑。6个审查负例均拒绝。

168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；核心实际6编译12VM、真实两类4重编通过，真实目标未执行。AA body12/budget130/name3继续为原字段子集。新7合法contexts（名称×debug和同值控制）×SourceTarget7/8生成14完整源码编译变体，三evidence先核全文相同；JDK8/23下14原件VM/28生成VM形成28对反射/行为一致，额外Driver编译1次，隔离加载并校验实际CodeSource。新7原fixture各成功编译一次，变异只查询；12份编码前端拒绝不计Java helper覆盖，raw重名/同值先后次序及既有g()独立return边界保持。同源变体不充独立holdout，语言目标7不充JVM7覆盖，也不将名称可读性改善计作新增可编译类。

五串行构建/31产物/2735来源/11原tools绑定，target清理，907839575bytes仅为构建磁盘护栏；AP8468封存payload保持。下一组先冻结enum constant anonymous-body完整JAR与有效/拒绝/停止域，再迁源码/flags/可展示资格，root保留物理配对/coverage/read/recover/费用/提交，避免crate反向依赖和新DTO。该处旧UTF8仅为静态风险，未冒称已复现；若改接受域另声明相对39的恢复版本。其后widening和raw/方法/BCI/span按共享机制推进，整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍未验收；2.9/2.10/S1未完成，17/59，S2关闭。

## S1 enum child 源码资格职责迁移（2026-10-25，AR）

[结果](results/s1-enum-child-source-policy-v1/README.md)、[收据](results/s1-enum-child-source-policy-v1/acceptance-r1.json)、[范围](results/s1-enum-child-source-policy-v1/coverage-scope.md)、[后续](results/s1-enum-child-source-policy-v1/next-boundaries.md)。

Java source_proof接管四组分阶段判据：child声明/无字段；旧UTF8方法名、identifier与flags；raw唯一base配对后的override可见性；同次RecoveryReport/analysis execution/声明与marker事实的可展示资格。facade保留实际class_source_with_evidence、物理identity、完整Code coverage、constructor/clinit、raw name/descriptor唯一配对、method poll、停止/原错误与group提交。root wrapper只解包Outcome并传已有类型/两bool，无ClassSourceMethod穿层或新DTO。两生产路径，Java既有测试和root cfg(test) helper assert_positive扩展，0新测试函数；4新纯helper，无Session/cache/read/永久source资产。恢复39/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：30 native1183/0/2；AQ最终semantic-r2的1170 current报告加AR138，共1308完整JSON保持，仅规范校验后的engine与usage直属elapsed_millis，raw/费用/partial/BCI/来源和错误优先级不变。168全文/78声明、136×3及408同evidence完整JSON/跨evidence outline保持；核心实际6编译12VM、真实两类4重编通过，真实目标未执行。AA body12/budget130/name3为原字段子集；AQ14完整源码编译变体/42源报告/28运行对经AQ→AR一跳text与pins核验复用，不计本轮新增JDK或新138份的行为oracle。

9同源contexts在Corretto8 source8/target8与JDK23 --release8各编译一次，共18成功编译、108报告（runtime8、SourceTarget7/8×三evidence）。现代54中ASCII两族/debug与BMP共30发布enum_projection，supplementary/_/fields/extra-method24无投影；旧compiler54缺$values形状，不能当源码helper覆盖。5 query-only变异30中18exit0、12exit4（instruction decode/method ambiguous），不编译/执行、不由终态断言内部拒绝切点。同源变体不充独立holdout，语言目标7不充JVM7覆盖。mutator初版parser失败、后续修正private/真duplicate及AA漏适配脚本的零产品调用失败均保留，未修改原11tools或重跑成功产品门。

五串行构建/31产物/2735来源/11原tools绑定，target清理，908144335bytes仅是磁盘护栏；AQ8887封存payload保持。下一组以严格MUTF与source_method_name统一child方法资格和既有忠实声明（不是保守source_identifier），保留raw配对/flags/Code/core证明与事务；接受域变更单列39之后版本及完整类/隔离行为/元数据/来源验收。现代supp/_12无投影仅为入口事实，不推断唯一根因；旧$values lowering差异独立归S5类级机制，不能混入名称修复。随后widening及raw/方法/BCI/span按共享机制推进，整类map/重定位属S5。2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic与八旧失败仍未验收；2.9/2.10/S1未完成，17/59，S2关闭。

## S1 enum 方法忠实名称恢复闭包（2026-10-25，AS）

[结果](results/s1-enum-method-name-v1/README.md) · [收据](results/s1-enum-method-name-v1/acceptance-r1.json) · [范围](results/s1-enum-method-name-v1/coverage-scope.md) · [后续](results/s1-enum-method-name-v1/next-boundaries.md)

Java child 方法资格与既有方法声明共同消费 names::source_method_name：严格 MUTF-8，保留 Java 7/8 单独 _。raw name/descriptor、base 唯一配对、flags/visibility、Code/core 完整性、预算/停止及组提交保持。仅 source_proof.rs 与 artifact.rs 改变，facade 与 AR 逐字节一致；0 新 helper/测试函数/DTO/Session/cache/read/永久 source 路径。接受域改变使用恢复39→40，schema12/解析4/inputs v9保持。

六项局部门通过：30 native1183/0/2；1452报告中1440完整JSON保持，12份 supplementary/单独_由无enum_projection变为闭合投影。12份完整文本及完整projection_inputs由已有ASCII有效机制与独立child物理事实参数化核对，其余physical/method/BCI/artifact/status保持。唯一逻辑费用变化是既有constants_text计费：supplementary6份3190→4907、_6份3174→4859，只出现在usage及execution.usage的output_bytes；两者为累计快照，不求和充实际I/O或RSS。168全文、78声明、136×3核心、408同evidence完整JSON/跨evidence outline与AA原字段子集12/130/3通过。

七个有效现代contexts×SourceTarget7/8生成14完整源码程序（字符串族包含当前Jarde IOps），三evidence先核全文一致，空classpath原样编译；Driver另编译1次。复用冻结原class，在JVM8/23执行原件14次/生成28次，28对方法名/参数返回类型、enum name/ordinal、常量体调用、tag/getOp输出一致，校验root/children/IOps的Loader与CodeSource。核心另实际6编译/12VM、真实两类4重编；真实目标和变异不执行。新child108/名称变异24/接口12报告复用完整AR JAR，原fixture成功编译不重跑；4名称变异18exit0/6exit4，前端失败不充Java helper覆盖。同源变体不充独立holdout，语言目标7不充JVM7。

普通widening主体已在Java；后续先核raw/String handoff、bridge共同源码证明与同次方法/BCI/span，不重复迁移薄适配包装。Corretto8缺$values的旧lowering单列S5，整类map/重定位同属S5。2.10实际read/Body/IR/live/peak/RSS及取消/预算/elapsed/sink/backpressure/panic、八旧失败和API/双轨收尾仍待原门验收。2.9/2.10/S1未整体通过，OpenSpec17/59，S2未进入。

## S1 bridge 当前类源码准入职责迁移（2026-10-25，AT）

[结果](results/s1-bridge-source-policy-v1/README.md) · [收据](results/s1-bridge-source-policy-v1/acceptance-r1.json) · [范围](results/s1-bridge-source-policy-v1/coverage-scope.md) · [后续](results/s1-bridge-source-policy-v1/next-boundaries.md)

Java source_proof接管bridge当前类27个准入守卫、同次Report/analysis完整性、raw name/descriptor唯一配对、flags/MethodParameters、参数cast form及covariant return判据；两个既有Signature是否参数化函数一并迁入。门面供给借用事实iterator和selected-header callback，保留resolver/平台absence/访问检查、reloaded-current、Stop清空bridge proofs、原成员和staging/commit。当前类证明是完整阶段迁移；最终继承header再生gate仍由门面编排，后续单列收口。

六项局部门通过：30 native1183/0/2；AS权威1452加新bridge84，共1536完整JSON严格保持，0语义例外，仅规范校验后的actual engine及usage直属elapsed_millis。168全文、78声明、136×3核心、408同evidence完整JSON/跨evidence outline及AA12/130/3原字段子集保持。核心实际6受控编译/12VM、真实两类4重编；既有bridge完整源码/erased dispatch/跨层级协变与停止测试本轮实际通过。AS14完整程序/54源报告/28行为对经一跳text、输入和pins核验复用，该项新增JDK0，不把新84份计入旧Oracle。

新84来自14既有context×SourceTarget7/8×三evidence，共7完整JAR，0新fixture编译/目标执行/变异；54报告有投影、30拒绝。跨配置102条proof记录为72 admitted/projected、30 refused，不能当独立holdout或27内部守卫探针。两个生产文件、facade减少359行/Java增加390行，新增一个阶段证明入口、两既有函数迁移，无新DTO/Session/cache/read/Body/测试函数；恢复40/schema12/解析4/inputs v9保持，实际Java/facade指纹更新。五构建/31产物/2735来源/11原工具绑定，target清理，908139246bytes仅为构建磁盘峰值。

下一项先收口继承header再生gate，并复现平台Comparable契约未明确匹配raw方法名的静态盲区；只有完整入口反例支持后才修接受域和版本。随后核raw enum owner/descriptor、widening同次method/BCI/loader。2.10真实资源/各类停止/背压/panic、八历史失败当前复核、API/双轨收尾仍未验收。2.9/2.10/S1未整体通过，OpenSpec17/59，S2未进入。


### 2026-10-25 / AU：继承 bridge 契约与源码 header 再生

[结果](results/s1-bridge-contract-source-v1/README.md)、[验收](results/s1-bridge-contract-source-v1/acceptance-r1.json)、[完整报告](results/s1-bridge-contract-source-v1/semantic-final-r1/summary.json)、[后续边界](results/s1-bridge-contract-source-v1/next-boundaries.md)。

Java source_proof 接管继承 owner 选定后的 interface/superclass header 再生资格，两个 Signature helper 收为内部实现；门面保留现有 resolver、平台 clean-Missing 探测、访问检查、事实供给、停止清空与原子提交。平台 raw owner/name/descriptor/use-kind matcher 放在 clean-Missing 后、认领契约前；不匹配继续原 member resolver 和其他父接口，不新增 provenance bool、DTO、Session 或缓存。

旧引擎完整 JAR 入口已复现 notCompare 被错认 Comparable.compareTo。恢复契约 40→41，schema12/解析4/inputs v9保持。1584 完整 JSON 严格保持；另18条错误平台准入转为拒绝并保留原物理方法，6条合法 Dual 双接口保持两个 bridge 投影。24条变化全部按事先冻结的 proof/text/费用向量核验，其余字段保持。absence每条少 AnalysisSteps6、AttributeBytes8及原 marker 257/275 UTF-8 bytes；Dual每条新增一次Other header及必要 resolver 逻辑费用，费用不充实际 I/O、RSS 或 resolver read 身份。

六项局部门通过：30 native 1183/0/2；1608完整报告；168全文、78声明、136×3核心及408完整JSON/跨evidence outline；AA12/130/3既有字段子集。新合法Dual+Other完整源码按SourceTarget7/8原样重编，3 javac（含独立Driver）/6 VM形成4个同runtime行为对；桥flags/擦除调用/实际CodeSource和loader均通过。AS14源变体/54报告/28行为对经AS→AT→AU两跳完整text/pins核验复用，该项新增JDK0。变异只查询，真实目标未执行。

本轮只改三条生产路径及一个既有测试函数，2735来源同路径、五构建/31产物/11原工具绑定；target清理，908105990bytes只是构建磁盘峰值。collector缺row.engine和core审阅器错指父目录均在成功采集后由独立只读finalizer修正；AS复用脚本的dict/list错误也只重跑审阅。原错误收据保留，成功CLI/JDK命令未重跑，原collector不改写成通过；权威报告为semantic-final-r1。

下一项仅迁 enum anonymous-child 的纯形状政策，复用现有facts引用、pending与Result/refusal，root保留selected read/原始身份/census/预算/stop/递归请求/commit。widening主体已在Java，已有公开class-source入口回归；待补同一数值BCI的跨method与跨loader同名异物理定义组合，不重复搬薄wrapper。Other类级可见性仅记静态待核实债务；当前Dual不是非法访问反例，不混修。验收工具的父producer/报告映射预检债务独立记录，后续必须先确认角色一致再执行产品。

2.9的其余职责/raw handoff与2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic、八历史失败当前复核、API/双轨收尾仍待原门验收。2.9/2.10/S1不勾完成，OpenSpec17/59，S2未进入。


### 2026-10-25 / AV：enum child metadata relation 完整阶段迁移

[结果](results/s1-enum-child-relation-source-v1/README.md)、[验收](results/s1-enum-child-relation-source-v1/acceptance-r1.json)、[完整保持](results/s1-enum-child-relation-source-v1/report-retention-r1/summary.json)、[下一项](results/s1-enum-child-relation-source-v1/next-boundaries.md)。

Java source_proof接管selected child的metadata relation阶段：child/enum两侧唯一anonymous InnerClasses行、类型级EnclosingMethod及唯一匹配构造器。输入借用现有reader facts/pools与raw owner/descriptor；返回原anonymous owner列表和借用MemberHeader，复用标准Result/Option/tuple，无新DTO/Session/cache。两条原扫描收费和逐行poll原位保留。root继续selected read、this/super与物理身份、pool/attribute读取、constructor/override Code、use census、递归请求和全组commit；不把原忽略的无效CP行提升成全局错误。

本轮纯职责迁移，recovery41/schema12/resolution4/inputs v9不变，实际Java/facade指纹更新。只有facade/source_proof两生产路径，root减少56行、Java增加85行；0测试函数/永久资产/来源路径新增。六项局部门通过：30 native1183/0/2；1740完整JSON全部严格保持，0语义例外，仅规范已核验的实际engine tuple及usage直属elapsed_millis。168全文、78声明、136×3核心、408完整JSON/outline、AA12/130/3原字段子集均保持。

分母1740=AU1608+旧lowering边界66+现代metadata负例66，两组各6复用控制已包含在AU，不重复计数。首轮误选Corretto8无$values输入，原流保留并明确不计本阶段覆盖；现代控制在新CLI前已按AS实际编译源码/report/input唯一匹配，两个常量体的projection_inputs存在。现代11类单点负例60 exit0/6重复constructor exit4，均保留可解析报告且不再投影；不能从末端报告推断所有私有helper guard均命中，变异只查询、不编译执行。

核心实际6编译/12VM与真实成功两类4重编通过。AS14源变体/54报告/28行为对按三跳，Dual两source targets/12报告/4对按一跳，均以完整源码与过程/pins核验复用；该复用项新增JDK0，不把新增132变异计入oracle。真实目标未执行。

五构建/31二进制/2735同路径来源、11原工具与3adapter绑定，target清理，908243530bytes仅为构建磁盘峰值。实际core-parent工具预检和旧新engine内存比较自检在构建前通过；构建后current producer发现build-plan缺旧模板baseline合同键，补入与原parent和冻结源码一致的四值，构建命令/目标/源/预算不改，原计划保留，原工具不改且成功命令未重跑。后续工具必须覆盖current producer冻结后预检，不能只用parent预检证明另一执行路径。

下一步验证widening完整class-source请求中的跨method同数值BCI与跨loader同名异PhysicalDefinitionId组合；已有普通/平台widening公开入口不重复搬迁。剩余enum group政策受root-private Pending/code candidate类型约束，先设计基于现有facts的边界，不凭空增实体或搬读/停止/commit。raw enum_owner旧UTF8仅为待完整入口复现风险，本轮不混修。2.9剩余职责/raw handoff与2.10实际资源/各类停止/八历史失败/API双轨仍待原门验收；2.9/2.10/S1不勾完成，OpenSpec17/59，S2未进入。


### 2026-10-25 / AW：完整类入口的 loader 图与 widening scope

[结果](results/s1-widening-entry-scope-v1/README.md)、[验收](results/s1-widening-entry-scope-v1/acceptance-r1.json)、[完整入口](results/s1-widening-entry-scope-v1/entry-report-review-r1/summary.json)、[后续](results/s1-widening-entry-scope-v1/next-boundaries.md)。

完整 class-source 请求原先只能构造单个无 parent 的 loader，不能表达 AV 计划的跨 loader 验收。现有 EnvironmentPolicy 增加 ExplicitDomains，借用现有 LoadDomain/HeaderProvider 声明完整图；EnvironmentRequest 的 loader 精确选择唯一 caller，缺失或重复直接 invalid input，不合成默认 domain。scope/profile/有序 domains/providers 原样进入 ResolutionEnvironment，后续继续原 validator、环境身份、defining loader 后继、缓存、读取、预算和停止；不新增 DTO/Session/cache/解析入口，也不改 Java widening 接受规则。该能力通过公开库入口验证，CLI 保留现有 flat 政策配置，不声称新图配置已由 CLI 交付。

同一 Scope 的 alpha(S) 与 beta(U) 经 reader 解码均只有 BCI1 的 invokestatic，分别需要 T/V。四组完整入口覆盖 child 链成功、child 链断裂、parent-defined S 不受 child Mid 遮蔽、parent T 与 child 同字节 T 不能合并；先核实际 loader/snapshot 选中身份，再核最终 cast/拒绝、独立 sibling、physical method 和 evidence 正文。RuntimeProfile11、SourceTarget7/8×三evidence，共24报告：alpha正12/拒12，beta24正。独立审查168个method-origin引用未串用。输入由冻结 Rust 测试内存构造，不新增binary fixture，未编译/执行本组输入；24份不计独立holdout或编译/行为oracle。

六项局部门通过：33 native1203/0/4（原30目标加2新测试及3既有widening目标；4忽略均为既有测试），1740完整JSON全strict、0语义例外；168全文/78声明/136×3核心、408完整JSON/outline和AA12/130/3原字段子集保持。完整报告仅规范已核验实际engine及usage直属elapsed_millis。核心实际6编译12VM、真实成功两类4重编保持；扩大native还有受控fixture的JDK调用，不由native汇总推算总调用数。AS14源变体/54报告/28对按四跳、Dual12报告/4对按两跳完整text/pins复用，该复用项新增JDK0；真实目标未执行。

一生产路径facade净增34行，一现有测试路径新增2测试/320行；2735来源同路径，recovery41/schema12/resolution4/inputs v9保持，Java指纹保持、facade指纹更新。五构建/34二进制绑定，target清理，909280925bytes只是构建磁盘峰值。工具审查在产品前补R2清单并让AA直达入口验证完整producer；之后runner残留30-target预检在零子命令时退出。原11tools及失败原流保持，外置continuation只将计数/报错绑定冻结plan33，其余检查与八子命令原样；成功native/1740及产品命令未重跑。原remaining-checks保留失败，权威产品结果为product-gates及continuation，不能改写为首次全绿。

本组闭合了既定widening组合的公开入口盲区，没有把薄wrapper再迁一层。下一项继续设计 enum group 现有 raw Code candidate/源码 policy 的所有权边界，root 保留 pending/身份/读/停止/commit；raw enum owner 仍先完整入口复现。2.9其余raw/方法/BCI/span及2.10实际read/Body/IR/live/peak/RSS、各类停止/背压/panic、八旧失败当前复核和API/双轨收尾未整体验收。2.9/2.10/S1不勾完成，OpenSpec17/59，S2未进入。


### 2026-10-25 / AX：enum Code candidate 与纯 API 证明的所有权

[结果](results/s1-enum-code-candidate-policy-v1/README.md)、[验收](results/s1-enum-code-candidate-policy-v1/acceptance-r1.json)、[边界](results/s1-enum-code-candidate-policy-v1/DESIGN-BOUNDARY.md)、[后续](results/s1-enum-code-candidate-policy-v1/next-boundaries.md)。

四个既有 EnumCodeReference/Instruction/Use/MethodCodeCandidate 定义、cheap eligibility、同次 MethodIr 候选捕获及完整 CP/bootstrap/operand 闭包归入 Java source_proof；values/valueOf 的纯 Code 证明及七个共享 descriptor/operand helper 一并迁移。root 通过重导出消费唯一实现，保留 ClassSource 物理索引/member/complete/handler 守卫、Pending、census、读/停止/递归请求和最终commit。公开 metadata/形状 bool 不授予可信 artifact。candidate名字、克隆内容、direct/bootstrap uses顺序、单层handle处理及原收费/poll逐项保持，不改恢复接受域。

两生产文件净减436/增445行，0测试函数/资产/来源路径新增，无新DTO/Session/cache/read；recovery41/schema12/resolution4/inputs v9保持，实际Java/facade指纹更新。33native1203/0/4、1740CLI完整JSON及独立24多domain完整入口JSON全strict，0语义例外；168全文/78声明/136×3与408完整JSON/outline、AA12/130/3原字段子集通过。五项既有捕获/预算取消/changed-values机制测试实际通过，不充所有私有guard覆盖。

核心实际6编译12VM、真实两类4重编保持，真实目标未执行。AS14源变体/54报告/28对按五跳、Dual12报告/4对按三跳全文/pins复用，该复用项新增JDK0。2735源/归档/五成功构建/34产物绑定，target已清理，908520366bytes仅为构建磁盘峰值。11工具/2adapter在产品前冻结，runner从plan取33目标，三执行组和八产品子门均直接通过；不重复成功命令。

下一项沿已有用户参数/实参类型、descriptor分类与纯Code证明的依赖闭包继续所有权迁移，不复制第二套模型；依赖ClassSourceField.declaration的构造器尾部和root-private Pending编排先保留。raw enum_owner仍先完整入口复现再修语义。2.9其余raw/方法/BCI/span、2.10实际资源/停止/八旧失败/API双轨未整体验收；2.9/2.10/S1不勾完成，17/59，S2未进入。


### 2026-10-25 / AY：enum 参数与实参纯政策归属

[结果](results/s1-enum-argument-policy-v1/README.md)、[验收](results/s1-enum-argument-policy-v1/acceptance-r1.json)、[边界](results/s1-enum-argument-policy-v1/DESIGN-BOUNDARY.md)、[后续](results/s1-enum-argument-policy-v1/next-boundaries.md)。

九个既有参数/实参/字面量/条件类型及impl、constructor descriptor分类、用户与固定int/String Code实参证明及源码拼写闭包归Java source_proof；共享raw方法唯一匹配、ASCII String解码、constant/连续BCI helper一并迁移。root消费唯一实现，保留initializer prefix、ClassSource字段声明/构造器耦合、物理配对、Pending/census、读取/停止与组commit。plain-int与user-int不同原判据、max3参数、String ASCII限制、字段符号和单次条件求值不改；不跨类求值、不新增DTO/Session/cache/read。

两生产文件净减808/增820行，0测试函数/资产/来源路径新增，2735同路径。recovery41/schema12/resolution4/inputs v9保持，实际Java/facade指纹更新。33native1203/0/4、1740CLI与独立24完整类JSON全strict，0语义例外；12既有参数/窄化/表达式/拒绝/预算机制测试实际通过。168全文/78声明/136×3与408完整JSON/outline、AA12/130/3原字段子集保持。24非enum入口是相邻回归，不充实参helper覆盖。

核心实际6编译12VM、真实成功两类4重编保持；AS14源变体/54报告/28对按六跳、Dual12报告/4对按四跳全文/pins复用，该复用项新增JDK0，真实目标未执行。修正后五成功构建/34二进制与源归档绑定，target清理，907404314bytes只作构建磁盘护栏。

构建前收敛未用重导出/多余public可见性；首check因漏ACC_STATIC导入退出101、0产物/产品，旧收据和对应源/归档保留。修正复用Java现有0x0008常量，重新冻结并用r2 target/bin完成构建；没有成功产品/验收命令重跑，不称首次全绿。13工具和4辅助脚本在产品前分别pin，24审阅重算完整producer并绑定native实际binary/receipt。

下一项按共享constructor Code/field机制迁固定与任意用户尾部的纯证明及已有输出类型，通过既有facts/窄事实供给拆除ClassSource耦合；root不永久持有纯源码政策，也不把request/identity/stop/commit搬入Java。raw enum_owner仍单列完整入口复现。2.9其余raw/方法/BCI/span、2.10实际资源/停止/八旧失败/API双轨未关门，2.9/2.10/S1不勾完成，17/59，S2未进入。


### 2026-10-25 / AZ：enum 构造器 Code/field 政策归属

[结果](results/s1-enum-constructor-policy-v1/README.md)、[验收](results/s1-enum-constructor-policy-v1/acceptance-r1.json)、[边界](results/s1-enum-constructor-policy-v1/DESIGN-BOUNDARY.md)、[后续](results/s1-enum-constructor-policy-v1/next-boundaries.md)。

四种既有固定 int、scalar String、String-varargs 与任意用户参数尾部的构造器 Code/field 证明、两个尾部类型及 Enum super descriptor 与带预算的连续 BCI helper 归 Java source_proof。root 重导出唯一实现；两个实际供应者在原唯一字段/重复字段判断之后，以借用闭包提供同次字段声明存在性，不预扫描、不增加读取/DTO，缺项仍拒绝。root 保留 ClassSource 记录、物理配对、请求/停止、constructor chain 与整组提交。

三生产文件净变化 root enum -403、facade +4、Java +411 行，0 新测试函数/资产/来源路径；原 root 测试体及 Java 既有政策区域逐字节相同。三个固定证明与预算连续 helper 仅改可见性，任意参数证明只替换事实供给；固定 int 既有弱检查没有顺手加固。三个带 Budget 的证明保留逐相邻指令 poll，原 IrItems/参数 poll、短路和错误优先级不变。recovery41/schema12/resolution4/inputs v9 保持，实际 Java/facade 指纹更新。

六项局部门直接通过：33 native 1203/0/4、1740 CLI 与独立 24 入口完整 JSON 全 strict，0 语义例外；168 全文/78 声明/136×3 核心、408 完整 JSON/outline 与 AA12 body/130 budget/3 name 原字段子集保持。12 项既有构造器/整组机制测试从当前 native 原流确认；missing declaration 与每个相邻 pair 的取消点未逐 guard 独立动态覆盖。24 非 enum Scope 入口只作相邻回归。

核心实际 6 编译/12 VM、真实两类 4 重编保持，真实目标未执行。AS14 源变体/54 报告/28 对按七跳、Dual12 报告/4 对按五跳全文/pins 复用，复用项新增 JDK0。五串行构建/34 产物与 2735 源/归档绑定，target 已清理；907228173 bytes 仅是构建磁盘采样峰值。11 工具/2 adapter 与 4 辅助脚本产品前冻结，三个执行组及八子门无成功命令重跑。

下一项先核 implicit constructor、delegation edge 与 terminal body 的完整事实/源码资格边界，沿现有 reports/candidates/reader facts 拆除 ClassSource 耦合，不搬 root 请求/物理事务，不新造 DTO。raw enum_owner UTF8 仍需完整入口复现再单列语义修复。2.9 其余 raw/方法/BCI/span、2.10 实际资源/各类停止/八旧失败/API双轨未关门；2.9/2.10/S1 不勾完成，OpenSpec17/59，S2 未进入。


### 2026-10-25 / BA：enum 构造链与 initializer-prefix 政策归属

[结果](results/s1-enum-constructor-chain-policy-v1/README.md)、[验收](results/s1-enum-constructor-chain-policy-v1/acceptance-r1.json)、[边界](results/s1-enum-constructor-chain-policy-v1/DESIGN-BOUNDARY.md)、[后续](results/s1-enum-constructor-chain-policy-v1/next-boundaries.md)。

source-tail 的既有枚举与完整 Signature 分类、implicit constructor、delegation edge、terminal 的源码资格和完整 Code/AST、initializer-prefix/factory 及共享 raw helper 归 Java source_proof；七个既有类型/输入/输出移动为唯一实现。root 保留两既有 input structs、初始 terminal 表计费/poll、source/header/index/raw 配对、Code.member 绑定与 method_code 原位借用回调。字段 callback 在唯一 Fieldref 匹配后才供物理配对/声明/markers/annotation refusals 事实，源码组合判据在 Java；原读取、停止与整组提交不穿层。

三生产文件净变化 enum -1047、class-source -53、Java +1213 行。两个 root 测试模块逐字节相同，Java 既有区域仅增 ACC_PRIVATE 导入；0 新测试函数/资产/来源路径/DTO/Session/cache/read。raw headers 的唯一性与三个 method_code 回调仍按原阶段和次序；terminal 的 Code.member 失配仍先于连续指令 poll；原 Signature、quality/content/Complete、flags/handler/效果与 BCI 拒绝保持。initializer-prefix 原本无 Budget，仍依赖原上层费用；各 charge/poll/错误映射不改。recovery41/schema12/resolution4/inputs v9 保持，实际实现指纹更新。

六项局部门直接通过：33 native1203/0/4、1740 CLI 与独立24入口完整 JSON 全 strict，0语义例外；168全文/78声明/136×3核心、408 JSON/outline与AA12/130/3原字段子集保持。18项既有构造链/prefix/Signature/停止机制测试从当前原生输出确认，不主张全部私有guard、逐pair取消注入或公开raw helper任意伪输入覆盖；24非enum入口只作相邻回归。

核心实际6编译/12VM、真实两类4重编保持，真实目标未执行。AS14变体/54源报告/28行为对按八跳、Dual12源报告/4对按六跳全文与历史pins复用，复用项新增JDK0。五串行构建/34产物与2735源及归档绑定，target清理；908568694bytes仅为构建磁盘采样峰值。11工具/2adapter和4辅助脚本产品前冻结，三个执行组与八子门通过，无成功命令重跑。

本组开始时发现后续审阅任务误写AZ两份已封存审稿；已保存改写稿，从首次写入工具payload只提取AST literal（未执行日志命令），按原SHA精确恢复两文件。AZ10497载荷/目录清单、原索引和验收复核通过；产品源码/测试/原始结果未改，未重跑旧产品。另一次迁移脚本函数定位错误发生在写源前，修正后实施；不混入构建/产品成功记录。审阅只允许当前检查点新路径，封存后禁止覆盖。

后续按完整ordinary group/census及suffix共享机制迁移剩余源码政策，保留物理/请求/停止/事务；raw enum_owner仍先完整入口复现再单列语义修复。2.9其余raw/方法/BCI/span与2.10实际资源/各类停止/八旧失败/API双轨未关门，2.9/2.10/S1不整体勾完，OpenSpec17/59，S2未进入。


### 2026-10-25 / BB：ordinary-enum group 与成员使用 census 政策归属

[结果](results/s1-enum-group-policy-v1/README.md)、[验收](results/s1-enum-group-policy-v1/acceptance-r1.json)、[边界](results/s1-enum-group-policy-v1/DESIGN-BOUNDARY.md)、[后续](results/s1-enum-group-policy-v1/next-boundaries.md)。

ordinary-enum group 的完整源码资格、常量/backing/构造器/API、全方法 Code 完整性及 direct/bootstrap 使用 census、prefix/clinit 使用/sidecar 准入和结果组装归 Java source_proof。ProvedEnumConstant/ProvedOrdinaryEnumConstantGroup 两个既有输出类型移动为唯一实现，root 重导出，未复制镜像 DTO。root 只保留 ClassSource/Body-family、原位物理表 owner/counts/completeness/初始收费/raw 配对，以及 Code、声明、Signature、PhysicalMethodId、构造链事实回调和原 proofenum/事务包装。

标准 Result<Result<Option<既有OrdinaryGroup>,String>> 保持外层 Stop、内层 Refused 与 None NotApplicable 的原含义；非枚举/不适用资格仍先于物理回调，首次表收费仍在完整性检查之后、逐项配对之前。五个预选 Code、随后 census、edge→terminal、String/任意参数字面量费用、clinit sidecar 与原结果字段顺序均保持；body-family fallback、int-field certification 和 nested 消费者继续连通。两生产文件 enum -707/Java +745 行，原测试体逐字节相同，0新测试/资产/来源路径/DTO/Session/cache/read。recovery41/schema12/resolution4/inputs v9 保持，实际实现指纹更新。

六项局部门直接通过：33native1203/0/4、1740CLI与独立24入口完整JSON全strict，0语义例外；168全文/78声明/136×3核心、408 JSON/outline及AA12/130/3原字段子集保持。20项既有group/完整性/census/预算取消机制测试从当前原生输出确认；不主张全部私有guard、公开raw helper任意假输入或资源总门，24非enum入口只作相邻回归。

核心实际6编译/12VM、真实两类4重编保持，真实目标未执行。AS14变体/54源报告/28行为对按九跳、Dual12源报告/4对按七跳全文及历史pins复用，复用项新增JDK0。五串行构建/34产物与2735源/归档绑定，target清理；908555195bytes仅为构建磁盘采样峰值。11工具/2adapter与4辅助脚本产品前冻结，三个执行组和八子门通过，无成功产品/验收重跑。

下一完整机制是 initializer-return/static-assignment/map suffix，之后再核 body-family 中仍混源码政策的阶段，root 事实/读/停止/提交边界不变。既有 method_code 在 census 中重复扫描候选仅为静态成本观察，留2.10按原测量协议验证，不在本组改费用或引入索引缓存。raw enum_owner 仍先完整入口复现再单列语义修复。2.9其余raw/方法/BCI/span及2.10实际资源/各类停止/八旧失败/API双轨未关门，2.9/2.10/S1未整体验收，OpenSpec17/59，S2未进入。


### 2026-10-25 / BC：initializer-return 与 suffix 完整政策归属

[结果](results/s1-enum-suffix-policy-v1/README.md)、[验收](results/s1-enum-suffix-policy-v1/acceptance-r1.json)、[边界](results/s1-enum-suffix-policy-v1/DESIGN-BOUNDARY.md)、[后续](results/s1-enum-suffix-policy-v1/next-boundaries.md)。

has_only_terminal_initializer_return、静态赋值和map后缀的完整Code/AST/元数据源码政策及两个既有结果类型归Java。root EnumStaticAssignmentInput不下移，仅以原位method/field事实借用适配；facade继续terminal-return优先、再static、只有static None才map，emitter/停止/安装/整组提交保持。static强字段匹配、map原弱字段/marker文本门槛分别保留，不合并守卫或新增recovered要求。

两生产文件 enum -772/Java +931行，原测试体及Java既有政策区域逐字节相同，0新测试/资产/来源路径/DTO/Session/cache/read。初始guard、static/map不同表费用、逐项poll、Code/sidecar原身份、BCI/primary与derived origins、clone/receiver清空/元素type重建原样保留。恢复41/schema12/解析4/inputs v9保持，实际指纹更新。旧固定名称/BCI/局部/步数与IndexLoop带否定any仅记录S5/待入口复现，不混修语义，不称泛化收益。

六项局部门通过：33native1203/0/4，1740CLI与独立24入口完整JSON全strict，0语义例外；168全文/78声明/136×3核心、408 JSON/outline、AA12/130/3原字段子集保持。14项既有suffix/行为/evidence/停止机制测试从当前r2原生输出确认，不主张全部私有guard、公开rawhelper任意假输入或资源总门；24非enum入口只作相邻回归。

核心实际6编译/12VM、真实两类4重编保持，真实目标未执行。AS14变体/54源报告/28行为对按十跳、Dual12源报告/4对按八跳全文/历史pins复用，该复用项新增JDK0。修正后五串行构建/34产物与2735源/归档绑定，r2 target清理；909989581bytes只作构建磁盘采样峰值。13工具/adapter与4辅助脚本产品前冻结，三执行组和八子门通过，无成功产品/验收重跑。

首次check因不存在的facts::ACC_NATIVE导入退出101，0二进制/产品；失败源码/归档/plan/预检/原流/receipt保留。只将原root私有0x0100位值随政策迁Java，移除错误导入；四只读预检重跑后以全新r2 target/bin成功构建。当前product-build-r1是固定检查点目录而非尝试次数，不把首次失败追认为通过。源审查r1对应原迁移、r2定点复核修正，旧绝对路径仅描述失败时点。

下一组核body-family构造器/initializer-prefix纯Code闭包及既有raw metadata记录；root保留query使用闭包、跨定义读取、pending与ClassSource/事务。2.9其余raw/方法/BCI/span与2.10实际资源/各类停止/八旧失败/API双轨仍按原门，2.9/2.10/S1未整体验收，17/59，S2未进入。

### 2026-10-25 / BD：body-family 构造器与 initializer-prefix Code 政策归属

[结果](results/s1-enum-body-code-policy-v1/README.md)、[验收](results/s1-enum-body-code-policy-v1/acceptance-r1.json)、[下一职责组](results/s1-enum-body-code-policy-v1/next-boundaries.md)。

body-family 的五个完整 Code 政策（constructor instructions/candidate、String constructor candidate、values factory、initializer prefix）及五个原事实记录已归 Java。root 从原调用点供给选中的 Code/member/fields，保留物理读取、MethodBodies/AnalysisSteps/IrItems 费用与 poll、arbitrary 分支、direct constants、跨定义解析、use census、ClassSource body 和提交。GroupShape 仍由 root 组装，待下一完整 shape/展示资格组处理；不复制 DTO，不新增 Session、cache 或读取。

facade 减少 490 行，Java 增加 496 行；五个函数体、五类记录字段只有路径/可见性/导入调整。物理读取函数、resolve relations 起的全部后续代码及测试体、Java 原有区域逐字节保持。构造参数逐槽转发、String putfield 物理索引、prefix 名称/ordinal/顺序、factory 与 suffix 的原拒绝域均保留；不新增固定样例或加固旧规则。恢复合同 41、schema 12、解析合同 4、inputs v9 保持，实现指纹更新。公开 raw 记录构造不能授予可信 Engine binding。

六项局部门通过：33 native targets 共 1203 passed / 0 failed / 4 既有 ignored；1740 CLI 与 24 独立入口完整 JSON 严格一致，零语义例外，只归一化经核验的实际 engine 元组和 usage.elapsed_millis。168 全文、78 声明、136×3 核心和 408 JSON/outline 保持，AA body12/budget130/names3 继续仅比较原字段。当前 native 原流确认 16 项既有 body-family 机制测试，覆盖参数/字段身份/prefix 正负、原子性和停止；不穷尽所有私有 guard。24 入口是非 enum 相邻回归，不是本组编译 oracle 或独立 holdout。

核心实际 6 编译/12 受控 VM、真实两类 4 重编通过，真实目标未执行。AS 54 源报告/28 历史行为对按十一跳，Dual 12 源报告/4 对按九跳，以完整源码和历史 pins 复用；复用新增 JDK/产品调用为 0。当前 r1 五构建全部通过，2735 来源/归档和 34 产物同次绑定，target 已清理；909047401 bytes 仅是构建磁盘峰值。11 tools、2 adapters、4 辅助脚本在构建/产品前冻结，成功产品命令未重跑；父 BC 的首次失败及 r2 修复仍是父历史，不归入本轮。

下一组整体迁移 prove_enum_constant_body_group 的 shape、sidecar、成员展示与已选子类结论消费，root 继续持有读取/查询与提交。raw enum_owner/MUTF、其余 method/BCI/span 与 A1.6 证据闭包继续 2.9；2.10 仍需实际 read、Body/IR、live/peak、RSS/耗时/取消延迟、各类停止/慢 sink/panic、八项旧失败和 API/临时双轨原门。逻辑 usage 与构建磁盘不替代这些门。2.9/2.10/S1 未整体完成，17/59；S0 原记录保持，S2 未进入。

### 2026-10-25 / BE：body-family group 完整政策归属

[结果](results/s1-enum-body-group-policy-v1/README.md)、[验收](results/s1-enum-body-group-policy-v1/acceptance-r1.json)、[下一职责组](results/s1-enum-body-group-policy-v1/next-boundaries.md)。

Java 已拥有 prove_enum_constant_body_group 的完整 shape/API/sidecar/成员与子类资格政策，以及原 GroupShape 单一实体。root wrapper只借用原方法/字段/关系事实，仍区分 raw name/descriptor 与物理 identity；ClassSourceMethod 及其 Arc、两个现有 Proved 输出继续归 root，由原字段装配回调保持源参数、subclass身份与同一child methods handle。Java 不查询、不读Body、不复制报告；没有镜像DTO、新实体、Session/cache或读取路径。

facade 减少441行，Java增加645行；67个拒绝/格式字符串顺序保持，一次 source_methods.len()+relations.len() 计费、两个主循环poll及内层Refused/外层Stop保持。固定String/zero/arbitrary域、same-shape、隐式API、terminal clinit/structured sidecar/write、abstract override、用户隐式成员使用、常量顺序、direct/anonymous关系和child唯一性、assigned-field/构造器展示均按原guard与原顺序检查。String仍要求signature-refused=true，markers为空或len==1，原本不查marker文本；arbitrary保留独立的len<=1/substring门。原entry/物理读/child恢复/census/preview/emission/事务与测试体、Java既有区域逐字节保持，不新增测试/fixture/固定样例分支。恢复41/schema12/解析4/inputs v9保持，实现指纹更新，公开raw/helper不授予可信Engine binding。

六项局部门通过：33 native共1203 passed / 0 failed / 4既有ignored；1740 CLI与独立24入口完整JSON严格一致，零语义例外，只归一化经核验的实际engine元组和直属usage.elapsed_millis。168全文、78声明、136×3核心及408 JSON/outline保持；AA body12/budget130/names3仍仅验原字段子集。当前原流确认20项既有group/child/field/prefix/原子性/行为/停止机制测试，不穷尽全部私有guard。24入口是非enum相邻回归，不计作group编译oracle或独立holdout。

核心实际6编译/12受控VM、真实两类4重编通过，真实目标未执行。AS54源报告/28历史行为对按十二跳，Dual12源报告/4对按十跳，以完整源码和历史pins复用，复用新增JDK/产品调用为0。当前r1五构建全部通过，2735来源/归档和34产物同次绑定，target清理；909621335bytes仅为构建磁盘峰值。11 tools、2 adapters、4辅助脚本在构建/产品前冻结，成功产品门未重跑。源码审查原件误写当前BE/子目录且为英文，原件保留，根层中文整理/JSON复制与精确marker说明另有摘要承接；没有因此修改源码、工具或封存父证据。

下一组先按完整阶段处理 resolve_enum_constant_body_relations 的main-class候选/shape准备政策，保留实际dependency读取、loader/definition、收费与停止位置；随后处理完整 source projection 资格/拼写，root继续持有ClassSource、emission安装与事务。raw enum_owner/MUTF、method/BCI/span和A1.6证据闭包继续2.9。2.10实际read、Body/IR、live/peak、RSS/耗时/取消、各类停止/慢sink/panic、八历史失败与API/临时双轨仍按S0原门；逻辑usage与构建磁盘不能替代。2.9/2.10/S1未整体完成，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BF：child读取前候选与shape完整政策归属

[结果](results/s1-enum-body-candidate-policy-v1/README.md)、[验收](results/s1-enum-body-candidate-policy-v1/acceptance-r1.json)、[下一职责组与复现顺序](results/s1-enum-body-candidate-policy-v1/next-boundaries.md)。

child读取前的完整候选/shape政策已分为Java三个纯阶段：候选预检、已读接口资格、最终Code/factory/prefix/constructor/direct-BCI shape。root在原位置显式读取必要接口，shape完成后才执行原child loop；Java没有content/environment/loader/execution对象，也不通过回调隐藏resolver/reader。原constructor/marker辅助和物理member构造闭包归Java，同一私有构建函数保留原字段clone；root测试只调整导入，测试体不改。

阶段tuple转交原identity、Code、constructions、raw成员/常量、source_ctor/shared参数和可选接口owner，不新增DTO/候选实体/Session/cache、报告深拷贝或Body。仅耗尽的借用索引工作Vec在纯阶段返回时较早释放，拥有的facts/AST/scan payload和后续必需值不复制，也不据此宣称RSS收益。facade减少564行、Java增加706行，七处预child费用与四处poll、候选/常量/API/bridge/abstract/interface原guard及字面值保留。root原child读取/恢复、group/census、preview/emission/诊断/事务及后续测试、Java既有区域逐字节保持。恢复41/schema12/解析4/inputs v9不变，实现指纹更新，公开raw/helper不授予可信Engine binding。

六项局部门通过：33native共1203 passed / 0 failed / 4既有ignored；1740 CLI与独立24入口完整JSON严格一致，零语义例外，只规范已核验的实际engine元组和直属usage.elapsed_millis。168全文/78声明/136×3核心、408 JSON/outline及AA body12/budget130/names3原字段子集保持。当前r2原流确认24项既有candidate/relation/nested/child/group/prefix/行为/停止机制，不穷尽全部私有guard；24非enum入口只作相邻回归，不是编译oracle或独立holdout。

核心实际6编译/12受控VM、真实两类4重编保持，真实目标未执行。AS54源报告/28历史行为对按十三跳，Dual12源报告/4对按十一跳，以完整源码和历史pins复用，复用新增JDK/产品调用为0。当前r2五构建全部通过，2735来源/归档与34产物同次绑定，target清理；910876185bytes仅是构建磁盘峰值。11tools/2adapters/4辅助脚本产品前冻结，成功产品门未重跑。

首次check因descriptor解析None分支仍返回旧空Vec而E0308退出101，0二进制/产品；失败源码、manifest/archive/freeze、plan/预检/原流保留。只将该分支包装改为Ok(None)，root仍立即返回原空关系；四只读预检重跑后用全新r2 target/bin构建，17脚本未改。当前product-build-r1是固定检查点名，build_attempt实际为2，不能写成首次全绿；源审查r1仅静态，r2单独证明一处分支包装修复。

剩余完整source projection资格/拼写按下一职责组推进。std::str::from_utf8(enum_owner)旧门槛及arbitrary constructor Error转String路径仍保留；前者需以完整公开入口覆盖合法supplementary/不可拼写/坏MUTF后修正，后者需区分本地quota先于ledger计费和operation stop闩锁，再用实际read/BodyIR复现，当前不宣称缺陷或修复。2.10继续S0原资源/各类停止/慢sink/panic/八历史失败/API临时双轨门，逻辑usage与构建磁盘不能代替。2.9/2.10/S1未整体完成，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BG：body源码投影完整资格与拼写归属

[结果](results/s1-enum-body-source-policy-v1/README.md)、[验收](results/s1-enum-body-source-policy-v1/acceptance-r1.json)、[后续](results/s1-enum-body-source-policy-v1/next-boundaries.md)。

完整body-source projection资格与拼写归Java，root只用四个纯借用accessor供给原field/method/source/constant facts，并消费五个原局部值组装唯一EnumConstantSourceProjection。ClassSource、child方法Arc、retained/emission安装与事务继续root持有；Java不查询、不读Body、不复制报告。原共享indent与user-tail构造器拼写整体迁Java，全部原消费者共用单一实现，不留重复helper，不新增DTO/候选实体/Session/cache/读取路径。

class_source减少163行、Java增加262行。常量/field/BCI/owner/name、child配对与非空methods、declaration/markers/Recovered/text闭合、backing与implicit原物理字段、SingleString/arbitrary构造器资格与文本保持。MethodItem raw name/descriptor与identity分别借用，不相互替代；原字面值、两处poll、四处OutputBytes charge、先收费再拒绝和单次constructor String clone保留，方法Arc只借用。root其余代码与测试、Java原有区域逐字节保持，0新测试/fixture/源路径。恢复41/schema12/解析4/inputs v9不变，实际实现指纹更新。

六局部门通过：33native共1203 passed / 0 failed / 4既有ignored；1740 CLI+24独立库入口完整JSON严格一致，零语义例外，仅规范经核验engine元组与直属usage.elapsed_millis。168全文/78声明/136×3核心、408完整JSON/outline以及AA ordinary12/budget130/names3原字段子集保持。当前原流确认24项既有body候选/关系/投影/原子性/停止机制；24非enum库入口只是相邻回归，不充编译oracle/独立holdout，也不穷尽所有私有guard或任意伪helper。

核心实际6编译/12受控VM，真实两类4重编通过，真实目标未执行。AS54源报告/28历史行为对按十四跳、Dual12源报告/4对按十二跳，以完整源码和历史pins复用，复用新增JDK/产品调用0。r2五构建/34产物与2735来源/归档同次绑定，target清理；910916341bytes仅是构建磁盘峰值。17脚本构建/产品前冻结，成功产品门不重跑。

r1被主审主动SIGINT中断：适配误用identity中同名字段，原规则应核MethodItem独立raw字段；exit130、0完整命令/二进制/native产品JDK，finally清理target，receipt.failure=null不代表成功。原源码/归档/plan/预检/静态审查/原流保留，r2仅修借用字段、四只读预检重跑并用全新target/bin；当前product-build-r1为固定输出名，实际build_attempt2。原静态source复核未识别差异，主审r2单列补核。工具native resume仅查日志存在的静态风险不声称已修，本轮不恢复既有native前缀，主审终验复算全部33原流计数、1740报告与进程/input/producer、24原report行、17脚本和封存父10532payload。

下一完整组为ordinary enum source projection的构造器/注解/initializer资格与文本，不按单flag堆helper。旧raw enum_owner UTF8门槛与arbitrary Error转String仍需完整公开入口分组复现；2.10实际read/BodyIR/live/RSS/时间/取消/停止/慢sink/panic、八历史失败和API临时双轨继续原门。逻辑usage与构建磁盘不替代实际资源门；2.9/2.10/S1未整体验收，OpenSpec17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BH：ordinary enum源码投影完整政策归属

[结果](results/s1-enum-ordinary-source-policy-v1/README.md)、[验收](results/s1-enum-ordinary-source-policy-v1/acceptance-r1.json)、[下一职责组](results/s1-enum-ordinary-source-policy-v1/next-boundaries.md)。

完整ordinary enum源码投影政策归Java：不同Signature marker、隐藏参数/源码参数注解、delegating/terminal构造器、varargs/user-tail、initializer目标与空enum分号统一在原函数完整阶段内检查和拼写。原prefix_method_annotations整体迁为Java单一实现供全部原消费者共用。root三accessor只借用原FieldItem/MethodItem的独立raw字段、owner、声明/markers及既有Java注解实体，原group借用，terminal/initializer文本move；Java返回六个原局部值，root装配唯一原Projection，继续持有ClassSource、读取/发射、retained与事务。

class_source减少372行、Java增加501行，原字符串顺序、1处poll、1处IrItems与2处OutputBytes收费及先分配/后收费/拒绝顺序保持。普通recognized marker仍只核refused+len1、arbitrary仍核len<=1/substring，未混同或放宽；initializer原totalUnits/Map固定条件保持。root其余字节与测试、Java原有区域保持；无新增DTO/Session/cache/read/clone/test/fixture/源路径，恢复41/schema12/解析4/inputs v9不变，实现指纹更新。

六局部门通过：33native1203/0/4既有ignored，1740CLI+24独立库入口完整JSON严格一致、零语义例外，仅规范实际engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline与AA ordinary12/budget130/names3原字段保持。40项当前既有机制来自本次原流（16普通投影重点+24body相邻），不是新测试或全部私有guard覆盖；24非enum库入口不计编译oracle/独立holdout。核心实际6编译/12受控VM、真实两类4重编通过，真实目标未执行。AS54源报告/28历史行为对按十五跳，Dual12报告/4对按十三跳全文/pins复用，复用新增JDK/产品0。r1五构建/34产物与2735来源/归档同次匹配，target清理，917616228bytes只作构建磁盘峰值；17脚本构建/产品前冻结，无成功产品门重跑。

完整原流审计复算1740报告/进程/input/producer、24入口原行、33stdout计数、17脚本与父10533payload。独立source复核仅静态，不冒充动态验收；本轮不从旧native prefix恢复，原resume日志未pin内容风险仍单列。下一完整组为nested enum引用改写的词法/origins/资格与替换政策，root保留条件IR读取、execution及物理anchors/提交；其recovery.clone成本仅为静态待测事项。raw enum_owner与arbitrary停止传播继续完整公开入口复现，2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨不被局部门替代。2.9/2.10/S1仍开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BI：nested enum方法引用改写完整政策归属

[结果](results/s1-nested-enum-reference-policy-v1/README.md)、[验收](results/s1-nested-enum-reference-policy-v1/acceptance-r1.json)、[下一职责组](results/s1-nested-enum-reference-policy-v1/next-boundaries.md)。

nested enum方法引用改写的完整纯政策按原读取前后分三阶段归Java：源码token/必要origins/声明候选，已读取IR的getstatic/CP/物理字段证明，完整替换与字节范围调整。root保留原owner词法prefix与Recovered事实选择、原位置body edits非空才analyze_method_ir、error/stop与execution/identity合并，原ClassSource span/member放置、四类anchors/MethodText和事务。声明only零IR；回调只供借用fact或装配原输出，不隐藏reader，不新建DTO/Session/cache/报告镜像。

facade减少187行、Java增加331行，三处poll与三处AnalysisSteps费用、15处原clone、原字面值保持；原IR/stop/identity代码块及函数外根层/测试、Java原有区域逐字节保持。MethodItem raw descriptor与field.identity.member按原来源，必要projection_origins不替换为可选source_map；最小覆盖origin并列合并、唯一FieldRef常量匹配、overlap/offset/placement拒绝顺序不变。Code/field纯阶段仅String Refused；prepare预算Stop与render offset硬Error保留。原单声明/RecoveryReport scratch clone不增不减，耗尽的借用索引工作Vec在阶段返回时较旧root迭代末尾早drop，不宣称RSS/时间收益。恢复41/schema12/解析4/inputs v9保持，实现指纹更新。

六局部门通过：33native1203/0/4既有ignored；1740CLI+24库入口完整JSON严格一致、零语义例外，仅规范已核验engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline及AA ordinary12/budget130/names3原字段保持。43既有enum/nested机制取自本次原流（40继承+3当前重点），不计新测试/恢复收益，不穷尽私有guard/任意伪helper；24非enum入口不充编译oracle/独立holdout。核心实际6编译/12受控VM、真实两类4重编通过，真实目标未执行；AS54源报告/28历史行为对按十六跳，Dual12报告/4对按十四跳全文/pins复用，复用新增JDK/产品0。r1五构建/34产物与2735来源/归档绑定，target清理，912416690bytes仅构建磁盘峰值；17脚本构建/产品前冻结，无成功产品门重跑。

首份静态复核因任务相对路径歧义误写repo/results并用HEAD，原件与位置/基线说明保留，不作为前继证明。新复核使用正确BI/pre-edit绝对路径，源摘要与封存BH manifest匹配；其结论仍只静态，主审另外复算1740报告/原流/进程/input/producer、24原report行、33stdout计数、17脚本和父10500payload。原native resume日志未pin内容的风险未混修，本次不恢复旧prefix。下一完整共享机制为static-fold字段/方法引用改写；raw enum_owner、arbitrary停止及2.10实际资源/停止/八历史失败/API双轨仍待原门，不被逻辑usage/构建磁盘或局部门替代。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BJ：static-fold owner源码改写完整政策归属

[结果](results/s1-static-fold-source-policy-v1/README.md)、[验收](results/s1-static-fold-source-policy-v1/acceptance-r1.json)、[下一职责组](results/s1-static-fold-source-policy-v1/next-boundaries.md)。

完整static-fold owner texts纯政策归Java：字段声明、override/retained projected来源组合、所有覆盖origins、声明资格、Code/handler/SSA来源、bridge/capture/PendingEdit和完整替换范围。root保留原override/retained物理事实、原条件惰性owner/Exceptions与IR读取及stop/execution/identity，raw throws身份成员关系供给、原ClassSource放置、anchors/OwnerTexts和事务。原局部PendingEdit单一实体整体移Java，bridge/capture沿既有Java单一helper，无新DTO/Session/cache/读取/报告镜像。

facade减少507行、Java增加750行。原5处poll/5处AnalysisSteps费用、28处clone、全部字面值保持；IR/stop/identity与owner read/headers/pool/attributes代码块、其余根层/测试和Java原区域逐字节保持。static-fold所有覆盖段并集、CP首匹配→handler→SSA不能改用BI最短origin/唯一enum字段规则；projected不支持组合拒绝、text-only投影不回退physical、声明only零IR、descriptor不命中才读Exceptions、原physical-only scratch/text与relation clone保持。descriptor_entry/declaration_edits等耗尽对象阶段返回时较早drop，不充RSS/时间收益。恢复41/schema12/解析4/inputs v9不变，实现指纹更新。

六局部门通过：33native1203/0/4既有ignored，1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范实际engine元组和直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline与AA ordinary12/budget130/names3原字段保持。51既有机制为43enum/nested回归+8static-fold集成，取自本次两个目标原流，不计新测试/恢复收益或穷尽全部guard。24非enum库入口只是相邻回归，不充编译oracle/独立holdout。核心实际6编译/12受控VM、真实两类4重编，真实目标未执行；AS54源报告/28历史行为对按十七跳、Dual12报告/4对按十五跳全文/pins复用，复用新增JDK/产品0。r1五构建/34产物与2735来源/归档同次绑定，target清理，913441918bytes只作构建磁盘峰值；17脚本产品前冻结，成功产品门不重跑。

独立source审查只静态；主审另复算1740报告/原流/进程/input/producer、24原report行、33stdout计数、17脚本、51机制和父10505payload。原native resume日志未pin风险未混修，本次不恢复旧prefix。下一完整组审member-fold整体组合资格与目标准备，root真实恢复/一致性重放/放置/提交不下移；raw enum_owner、arbitrary停止及2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BK：member-fold整体组合资格与目标准备

[结果](results/s1-member-fold-composition-policy-v1/README.md)、[验收](results/s1-member-fold-composition-policy-v1/acceptance-r1.json)、[下一职责组](results/s1-member-fold-composition-policy-v1/next-boundaries.md)。

member-fold的完整组合资格与目标准备归Java：先三轮核对物理complete/声明、Signature承载、enum/initializer承载，再按成员物理顺序准备精确root/child源码名、child重放后row/binary一致性与原targets；re-recovery完整Java质量门和bridge owner MUTF拼写一并归属。root保留真实IR/recovery、execution/identity、原evidence/instance/capture feeds、retained上下文构建及source_text重放供给、物理模型/anchors/writer和最终事务。四纯函数使用借用accessor与原target factory，无新DTO/Session/cache/read/fixture/测试函数。

facade减少64行、Java增加137行；原2处poll/1处OutputBytes/25个clone调用点保持。三轮拒绝顺序、enum短路后才算initializer order、有order才查clinit、child源码名成功后才重放、重放后才核row/binary与完整targets之后才re-recovery保持。实际IR/recovery调用、execution/identity与原selection代码块逐字节相同，其余root/全部测试和Java旧区域逐字节保持。child重放临时对象较原迭代结束提前drop，不充RSS/时间收益；重复expect仍位于原complete不变量之后。当前accessors均纯借用，Fn本身不保证纯度，静态审查记录了重复调用/元组求值边界，不将底层helper当可信artifact构造器。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：主33native1203/0/4既有ignored；覆盖复核后补充既有mixed/instance目标5/0/0，合计34目标1208/0/4，1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范核验后的实际engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline及AA ordinary12/budget130/names3原字段保持。51既有enum/nested/static-fold机制与11项既有generic/header/family相邻回归取自本次成功原流，不增加执行次数，不声称每条穿过全部内部gate或独立holdout。核心实际6编译/12受控VM、真实两类4重编，真实目标未执行；native原fixture受控编译/行为亦通过但无逐JDK调用总数收据。AS54源报告/28历史行为对按十八跳、Dual12报告/4对按十六跳全文/pins复用，该复用新增JDK/产品0。原r1五构建/34产物及独立补充mixed一次构建/1产物均与同一2735来源/归档绑定，两个target清理，932443512bytes仅作构建磁盘峰值；17脚本产品前冻结，成功产品门未重跑。

独立source审查只静态；主审复算1740报告/原流/进程/input/producer、24原report行、主33stdout计数、17原脚本、独立补充脚本/来源/5项mixed原流、51机制和父10500payload。下一完整职责组是nested_static_member_source_text的header/字段方法来源优先级/constructor捕获参数消除/缩进范围政策，root保留原模型/anchors及提交，不按旗标零散打补丁。raw enum_owner、arbitrary停止、2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BL：嵌套成员writer完整源码政策归属

[结果](results/s1-nested-member-writer-policy-v1/README.md)、[验收](results/s1-nested-member-writer-policy-v1/acceptance-r1.json)、[后续](results/s1-nested-member-writer-policy-v1/next-boundaries.md)。

嵌套static/instance member writer完整源码政策归Java：row flags/物理raw parents与重写父类型/header、annotation/marker、字段和方法的staged→retained→physical来源优先级、constructor改名和capture首参数消除、hidden-write clamp、缩进前后UTF-8范围与原source span逐字节核对。两个专用参数扫描/范围位移helper及三个共享declaration_member/comment_member/indented_offset单一实现归Java；根层其余调用保留薄包装。root保留原ClassDeclarationFactsclone、同次ClassSource/MemberFamily对象和全部物理anchors工厂、原NestedClassSourceText封装、真实恢复/停止与事务。泛型accessors只借用现有对象，factories创建原对象，无镜像DTO/Session/cache/read/测试/fixture。

class_source减少225行，Java增加395行；facade文件和Java旧前缀逐字节保持，root除完整writer、两专用helper和三个共享薄包装外其余代码及测试逐字节保持。原15个.clone和1个staged整对象.cloned调用点、原零poll/charge保持；原ClassDeclarationFactsclone留root。header已用尽的父类型String/接口Vec在header返回时早于原writer结束释放，不宣称实际RSS/时间收益。原首参数扫描的泛型/注解/字面值/嵌套括号、位移次序、hidden-write重夹与span不一致拒绝均保持，不修或放宽S5域。恢复41/schema12/解析4/inputs v9保持，实现指纹更新。

六局部门通过：34native1208/0/4既有ignored，mixed目标预先纳入主计划；1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范核验后的actual engine元组与直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline及AA ordinary12/budget130/names3原字段保持。51既有enum/nested/static-fold机制、11generic/header/family相邻回归及5mixed/instance集成取自同次成功原流，不额外执行，不称新恢复收益、全部私有guard覆盖或独立holdout。核心实际6编译/12受控VM、真实两类4重编；native夹具既有受控编译/行为也通过，无逐JDK调用总数收据，真实目标未执行。AS54源报告/28历史行为对十九跳、Dual12源报告/4对十七跳按全文及pins复用，该复用新增JDK/产品0。

r1五构建/35产物与2735原路径来源/归档绑定，target清理，913728667bytes仅作构建磁盘峰值。17脚本构建/产品前冻结，成功门未重跑；独立source只静态，主审另核1740原报告/交付流/进程/input/producer、24原report行、34stdout计数、67既有测试行、17脚本及直接BK父10516payload。下一完整职责组审nested enum/annotation两个writer，沿不同接受域整体迁移并复用已有纯helper，不为表面相似统一资格；raw owner/arbitrary停止、2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BM：嵌套enum writer与annotation源码阶段

[结果](results/s1-nested-enum-annotation-writer-policy-v1/README.md)、[验收](results/s1-nested-enum-annotation-writer-policy-v1/acceptance-r1.json)、[后续](results/s1-nested-enum-annotation-writer-policy-v1/next-boundaries.md)。

完整nested enum writer及annotation纯源码阶段归Java。迁移前源码没有nested_annotation_source_text，annotation在facade::project_class_source_nested_annotation内：本轮迁移原单元素资格（raw Item name/descriptor、value()F、default、NoBody、单一no-body marker）、owner源码引用排除和annotation块拼写；root保留family/complete/声明、child/root原source_text重放、原facts clone、poll/anchors/OutputBytes/事务。enum constants/字段/initializer/constructor/implicit/clinit/method/descendant次序及header完整纯政策归Java，root保留原ClassDeclarationFacts clone/模型/anchors factory/NestedClassSourceText封装；仅失用的root header适配删除，复用既有Java声明/indent/成员拼写/indented_offset。无新DTO/Session/cache/read/测试/fixture。

class_source减少87行，facade减少20行，Java增加233行。旧Java前缀、两个root文件除完整enum writer/annotation projector和唯一失用wrapper外所有代码/测试逐字节相同。原6个.clone、annotation的1poll/1OutputBytes及两个ClassDeclarationFacts clone保留；child/root replay及poll→anchors→commit代码块逐字节相同。enum method span保留原文相等，descendant只沿原非空与UTF-8边界合同，不强合并；annotation逐行补四空格包括空行，不改用跳空行indent。annotation binary_source_name/header/header_text阶段返回时更早drop，不充RSS/时间收益。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

六局部门通过：35native1215/0/4既有ignored，既有annotation七项与mixed预先纳入主计划；1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范已核验actual engine元组和直属usage.elapsed_millis；168全文/78声明/136×3核心、408JSON/outline与AA12/130/3原字段保持。51enum/nested/static-fold机制、11generic/header/family相邻、5mixed及7annotation为74条本次原生成功原流，不增加执行次数，不称全部guard、任意伪helper、独立holdout或新恢复收益。核心实际6编译/12受控VM、真实两类4重编；原native夹具受控编译/行为亦通过，无逐JDK调用总数收据，真实目标未执行。AS54源报告/28历史对二十跳、Dual12报告/4对十八跳全文/pins复用，该复用新增JDK/产品0。

r1五构建/36产物与2735同路径来源/归档绑定，target清理，913762419bytes只是构建磁盘峰值。17脚本产品前冻结，成功门不重跑。独立source只静态，主审复算1740报告/交付原流/进程/input/producer、24原report行、35stdout计数、74测试行、17脚本与父BL10520payload。下一完整职责组是render_member_class与append_family_method的narrow family源码与范围政策，root保留物理/capture/generic事实和anchors/事务，不照抄static-fold接受域；raw owner/arbitrary stop、2.10实际read/BodyIR/live/RSS/时间/取消/停止/八历史失败/API双轨仍待原门。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BN：narrow member writer与方法放置完整政策

[结果](results/s1-narrow-member-writer-policy-v1/README.md)、[验收](results/s1-narrow-member-writer-policy-v1/acceptance-r1.json)、[后续](results/s1-narrow-member-writer-policy-v1/next-boundaries.md)。

完整render_member_class和append_family_method源码与范围政策归Java。Java决定narrow/generic/capture的row flags、annotations/header、static插入、capture/generic/hidden来源位置、字段/bridge/omitted与projected方法选择、无capture时constructor改名，以及nested/flat方法放置。root保留原validated child declaration expect、ClassDeclarationFacts clone、原MemberFamily/GenericStatic/capture对象与全部anchors factories；capture field查找/原expect通过lazy factory仍在header成功后，真实读取/恢复/停止/外层事务不下移。复用已有Java声明/indent/成员/range helper，无新DTO/Session/cache/read/测试/fixture。

class_source减少80行、Java增加217行。facade文件、Java旧前缀和root除完整renderer/append外所有代码及测试逐字节相同。原13个.clone、generic.header与staged整对象clone和D clone、原零poll/charge保持；普通class_declaration语义未换成更严格nested header。generic字段仅在原Some declaration分支替换并保留空markers，其anchor仍独立guard；bridge用原identity，omitted仅当hidden_fields非空；nested核缩进source_span原文相等，flat只原加偏移，拒绝/原expect时点/clone顺序不变，不放宽S5合同。没有实测资源收益声明。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

六项局部核验中五项通过；原生门36目标1233通过/2既有失败/4既有ignored，新增纳入member_family_identity的20项为18通过/2失败。r1流水线因此中断，原流保留；从直属BM封存2735源码单独重建同一target，在相同JDK/default stack实测同名18/2，未改产品/测试/预期。只补运行未执行的最后5项，35个已运行目标不重跑；两失败仍归2.10，不将六门称为全绿；1740CLI+24库入口完整JSON严格一致/零语义例外，仅规范已核验actual engine及直属usage.elapsed_millis；168全文/78声明/136×3核心、408 JSON/outline与AA12/130/3原字段保持。既有51 enum/nested/static-fold、11 generic/family相邻、5 mixed、7 annotation与20 family identity共94条本次原流（92成功、2同前继失败），机制提取不重复执行当前native（另一次前继对照分列）、不称全部guard/任意假helper、新恢复收益或holdout。核心实际6编译/12受控VM、真实两类4重编；native原受控编译/行为也通过，无逐JDK调用总数收据，真实目标未执行。AS54源报告/28历史对二十一跳、Dual12报告/4对十九跳全文/pins复用，该复用新增JDK/产品0。

当前r1五构建/37产物与2735同路径来源/归档绑定，另有BM源码对照的一次受控构建及一个产物，两个target清理，914279854bytes只作构建磁盘峰值。原17脚本构建/产品前冻结，后补4脚本另行冻结；r2在前继对照完成后误断剩余target名称，错误记录保留，r3按构建receipt修正编排且不重跑前继；r3原产品runner亦因native全绿前置未满足而停止，r4只独立运行尚未启动的八个保留检查，不称原六门流水线通过；成功门未重跑；独立source仅静态，主审另复算1740报告/交付流/进程/input/producer、24原report行、36stdout计数、94测试行、17脚本和直接BM父10518payload。下一步按2.9对raw enum owner/descriptor及arbitrary stop做完整公开入口复现，剩余职责审查确认Java7整类拒绝门是下一最小迁移候选；物理报告装配/anchors/事务留root，不机械搬整个writer。2.10实际read/BodyIR/live/RSS/时间/取消/停止/八旧失败/API双轨仍待S0原门；本次新纳入的两项旧形状/过宽断言及同夹具fold效果覆盖缺口单列known-native-failures-r1.md，不混改。2.9/2.10/S1开放，17/59，S0原记录保持，S2未进入。

### 2026-10-25 / BO：整类源码目标拒绝政策归 Java

[结果](results/s1-source-unit-target-policy-v1/README.md) · [验收](results/s1-source-unit-target-policy-v1/acceptance-r1.json)。

Java 7 整类源码拒绝政策已迁至现有 Java declaration::source_unit_target_refusal；root 的 source_text_with_member 只借用现有字段/方法事实。保持 release<8 短路、interface default/static+Code 判断、type_uses 原计算位置、interface 理由优先，以及 escaped→comment_text 的原文与换行。root 保留物理报告、anchors、预算/停止、可靠前缀和外层事务，不机械迁移报告装配。无新 DTO/Session/cache/read/测试/fixture；class_source -10 行，declaration +37 行；facade、source_proof、测试和其他来源逐字节保持，clone/poll/charge 调用点不增。恢复41/schema12/解析4/inputs v9保持，实际实现指纹更新。

保留核验完成：36原生目标1233通过/2同名既有失败/4既有忽略，与直属BN的逐目标计数/退出码/原流结果相同；两失败不改为通过，仍归2.10的测试形状/过宽断言及同夹具fold效果覆盖。1740CLI+24库入口完整JSON严格保持；额外四真实class×Java7/8×essential/all共16对完整CLI报告/32次调用，拒绝理由与正文、物理身份/方法报告保持。只规范已核验actual engine及直属usage.elapsed_millis，零语义例外。168完整源码/78声明/136×3核心与408 JSON/outline、AA12/130/3保持。核心实际6编译/12受控VM及真实两类4重编通过；真实目标未执行。AS54＋Dual12共66源码连接/32历史行为对按22/20跳复用，该复用新增JDK/产品0，不冒充新执行或holdout。

2735来源同路径/归档与五构建37产物同次绑定，target清理；原19脚本及补充1脚本在各自产品执行前核pin（报告工具schema在只读预检中修正，初始脚本/pins/错误记录保留；目标CLI对照r1误配PlainJar给standalone被BN拒绝，r2按既有测试改用SingleClass，错误调用另列，不称所有脚本均在构建前最终冻结）。主审另从原流复算1780完整JSON对、36原生结果/同名旧失败、20脚本及直属BN10544封存payload；独立复核仅静态。当前功能测试未单独分离static-only和两原因同类的输入，逐式源码等价支撑该边界，不冒充穷尽分支或实际资源收益。

本轮只收口这一处剩余语言政策；其余报告混排/物理anchors/提交留root。后续优先2.9 raw enum owner/descriptor三组的完整公开入口复现，再处理arbitrary constructor Stop与2.10实际读取/IR/live/RSS/时间/取消/全部停止门、八旧失败/API双轨。2.9/2.10/S1未整体验收，OpenSpec17/59，S0原记录保持，S2未进入。

### 2026-10-26 / BP：enum constant-body raw owner与descriptor机制

[结果](results/s1-raw-enum-owner-boundary-v1/README.md) · [验收](results/s1-raw-enum-owner-boundary-v1/acceptance-r1.json)。

BP关闭一个已公开复现的共享机制缺口：constant-body enum candidate把MUTF raw owner当标准UTF8，四处descriptor连接与物理原字节不在同一编码域。现在统一复用已有object_descriptor/array_descriptor/values_descriptor/value_of_descriptor；source拼写仍由严格MUTF names处理，reader/物理选择、同次Code/scan、flags/属性/构造链、interface→shape→child读取、预算与事务守卫保持。无新命名模型、DTO/Session/reader/resolver/cache或读取编排，不按样例名放行；facade和enum_constants源码逐字节不变。恢复语义41→42，schema12/解析4/inputs v9保持，实际Java实现更新；root实现指纹不变。

精确BO生产实现加同一永久测试/fixture的baseline为13通过/1失败，唯一新增失败是合法supplementary owner完整投影；当前raw目标8/0、lookup边界4/0、原enum-string2/0。公开三组/ASCII控制×SourceTarget7/8×essential/source_map/all：合法owner投影0/6→6/6；ASCII6、不可拼写6拒绝、坏MUTF6 Failed保持，其中18完整JSON对严格一致；六新恢复报告的root物理身份、声明、字段与方法报告保持。原样完整生成类编译0/3→3/3（JDK8 source7/8、JDK23 release8），共9次javac；ASCII当前全文等于旧已编译源，三次编译按pins复用。真实LabeledValue源码作依赖、无stub/原目标classpath；新变体未执行，不充行为oracle或holdout。

继承1780完整JSON对（1740CLI/24库/16目标入口）仅规范核验actual engine及直属usage.elapsed_millis，零语义例外；原生39目标1247通过/2同BO既有失败/4既有忽略，两旧红不改测试或预期。168全文/78声明/136×3核心、408 JSON/outline与AA12/130/3保持；核心实际6编译/12受控VM、真实两类4重编通过，真实包未执行。AS54＋Dual12共66源码/32历史行为对按23/21跳全文/pins复用，该复用新增JDK/产品0。旧五个raw测试和check正文逐字节保持，仅抽取result适配以断言拒绝/Failed并追加三测试。

2735封存BO来源+4显式新增fixture路径形成2739测量baseline；产品与其同路径、测试/fixture相同，仅两生产源码不同。五成功构建/40产物绑定actual producer，baseline/产品target均清理；21工具current构建/产品前核pin。主审另复算报告/原流/进程/input、39native原流、三组矩阵和编译、21脚本与BO10608payload；独立审查仅静态。初期摘要器读未序列化字段和空projection_inputs省略的验证器错误原流保留，修正读取后复用已有进程，不重跑成功前缀。

合法owner新成功沿既有有理由依赖链增加Header/Body/IR推进，逻辑usage另列，不能声称零读取增量或通过S0实际资源门。当前态校核已排除旧draft三项过时缺口（enum child方法名、integer raw去重、array声明匹配）。2.9仍待A1.6全消费者证据闭包审计，不凭本例勾选；arbitrary constructor Error→String属A1.3/2.10，须实际复现/不可达证明。2.10实际资源/全部停止/八旧失败/API双轨仍开放；17/59，S1未整体验收，S0原记录保持，S2未进入。


### 2026-10-26：第三轮六项验收证据复核

[六门逐项结果](results/s1-enum-constructor-stop-v1/six-gates.md)已将第三轮 generic declaration R3 与最新 BP 的结果分列。第三轮六门完成：14 native 957/0/1、2 正例 6 报告及 selected-leaf oracle、6 负例 36 报告、168 全文、78 声明、136×3 核心及受控编译/VM/真实两类重编；本轮逐项重核其 6218 个封存载荷与 BP 的 10872 个载荷，没有重跑成功历史门。BP 扩大 native 回归后的两项既有失败仍为失败，不能借“第三轮完成”宣布当前 S1 全绿。2.9 全消费者证据闭包、2.10 原资源/停止/API 门保持开放，S2 未进入。


### 2026-10-26 / BQ：enum 构造链类型化停止与第三轮六门复核

[结果](results/s1-enum-constructor-stop-v1/README.md)、[收据](results/s1-enum-constructor-stop-v1/acceptance-r1.json)、[第三轮六门](results/s1-enum-constructor-stop-v1/six-gates.md)、[剩余职责及验收](results/s1-enum-constructor-stop-v1/next-boundaries.md)。Java shape proof 保留外层 Budget/Cancelled，源码拒绝留内层；无新模型/read/cache，guard/拒绝文案/charge/poll/clone顺序保持，恢复42→43/schema12/解析4/inputs v9。实际helper反例由Ok(Some字符串拒绝)转为typed BudgetExceeded；公开local96点保留父方法文字/身份，10点避免后继Body解码（8/10→7），IR启动均7，不冒充S0资源或任意全局零读门。

当前6 native目标296通过/1 EM11 cap-two相邻断言失败，原r1退出101保留，r2只续未执行目标；旧/当前CLI同EM11请求均Complete且全JSON相等，旧原生精确测试未重跑，归2.10核对，不改预期或宣称全绿。1765完整报告严格保持、零语义例外；168全文/78声明、136×3核心及408全JSON、核心6编译12受控VM、真实两类4重编通过，真实目标未执行。新夹具只编译，未执行或充holdout。2743源路径/归档/7产物与三成功构建绑定，target清理；峰值538557236bytes仅构建磁盘。

第三轮六门历史通过及6218载荷重核已同步。strict验证器两次错误原流保留：138进程后的路径字符串误比、763进程处沿用输出路径误写BP一份交付；原文件已按封存hash精确恢复，10872个BP载荷与原索引重核。r3校核输入与输出重定向、复用763进程，只补1002，不重跑成功168/78/核心门。剩余普通静态初始化整组及enum外部字段源码资格仍按既有S1职责迁移，root保留物理事实/读取/停止/事务；2.9闭包与2.10资源/全部停止/历史失败/API双轨未完成，17/59，S0记录保持，S2未进入。


### 2026-10-26 / BR：普通静态初始化完整政策职责迁移

[结果](results/s1-static-initializer-source-policy-v1/README.md)、[收据](results/s1-static-initializer-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](results/s1-static-initializer-source-policy-v1/next-boundaries.md)。普通class/interface初始化的完整proof、RHS field@1/constant-expression phase/效果与顺序政策、全部发射staging归Java；root只供同次借用事实，保留读取/IR、执行诊断与完整成功后的原子提交。7函数/5原有类型归Java，无新模型/Session/cache/read；物理字段顺序、class/interface不同接受域、原拒绝优先级和3charge/0显式poll/8clone调用保持，删除旧无用私有class_internal参数。报告两类型re-export；doc-hidden pub错误/helper扩大Rust可见面，但不提供新artifact绑定权或Engine注入入口。恢复43/schema12/解析4/inputs v9保持，实际Java/root摘要更新。

旧生产加两新对抗测试先实跑5目标24/0；当前9选定目标315/0/0，新增8类field@1错配拒绝理由逐字保持，late mismatch在首RHS已发射计费后仍零提交。1765继承+60专项完整JSON严格相同、零语义例外；专项10class×两个SourceTarget×三evidence为24 Proved/30 Refused/6 NotApplicable，不充60完整编译或holdout。168全文/78声明、136×3核心/408全JSON、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2743源/归档/10产物与两正常构建绑定，target清理，583731025bytes仅构建磁盘；首次baseline测试Option访问编译错误保留，修正后不改变生产判据。

BQ相邻EM11断言与BP两旧native失败继续原记录，不因本轮315/0改变其状态。静态组已完整迁移，不重复计为待迁；下一完整组为enum外部static int字段资格，然后核instance-array constructor族政策。2.9raw消费者闭包与2.10实际资源/全部停止/API双轨仍开放，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / BS：enum 外部静态 int 字段资格完整职责迁移

[结果](results/s1-enum-static-field-source-policy-v1/README.md)、[收据](results/s1-enum-static-field-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](results/s1-enum-static-field-source-policy-v1/next-boundaries.md)。enum 有限 int 构造表达式的完整外部字段资格、owner/shadow 政策归 Java；root 保留同次 enum 属性、selected read、owner pool/属性供给与执行停止编排。3 函数整体迁移、两个生产文件，root -183 行/Java +221 行，无新模型/测试/fixture/路径或读取/cache。原 raw 三元组去重/首次 source_owner、守卫/拒绝次序、3 poll/3 charge/5 clone 保持；owner mask 不含 ACC_ENUM，未扩大或收缩接受域。3 doc-hidden pub 扩大 Rust 可见面但不授予 Engine artifact 绑定权；恢复43/schema12/解析4/inputs v9 不变，实际实现摘要更新。

直属 BR 的 10420 封存载荷/2743 源及旧 315/0 原流重核后复用，不重复旧构建。当前 9 目标315/0/0，8 项字段机制测试通过；1825 继承与72字段专项完整 JSON/退出码保持、零语义例外，仅规范核验后的实际 engine 和直属 usage.elapsed_millis。72 为3输入×目标7/8×三evidence×四预算模式，18充分预算投影/54有限预算；不声称全部到达资格函数或独立 holdout/完整编译。168全文/78声明、136×3核心/408全JSON保持，核心6编译12受控VM及真实两类4重编通过，真实目标未执行。2743源/归档/10产物与两正常构建绑定，target清理，584082894bytes仅构建磁盘。

精确字段循环取消、缺失字段/owner及所有版本/flags未逐branch动态穷尽，原状态分流保持。BQ EM11 相邻断言与 BP 两旧 native 失败未重跑/豁免。BR静态组与BS字段组不重复列为待迁；下一完整组为 instance-array constructor 政策，要求同次 AST/读事实、原子性、效果/放置和停止验收。2.9消费者闭包与2.10实际资源/全部停止/历史失败/API双轨仍开放，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / BT：共同 instance-array 构造族政策与暂存职责迁移

[结果](results/s1-instance-array-source-policy-v1/README.md)、[验收](results/s1-instance-array-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](results/s1-instance-array-source-policy-v1/next-boundaries.md)。整个共同primitive-array constructor group的census、AST/member配对、所有构造器RHS/写入一致性、raw字段/调用descriptor资格、字段/成员冲突及全部暂存归Java；root供借用事实、原共享writer适配，并保留最终整组提交。4函数/两生产文件，root生产-419行/Java+517行，无新模型/fixture/read/cache；20poll/26charge/0clone与最终poll保持。共享ClassSourceMethod current-body/markers/annotations writer仍在root，下一组按完整writer迁移，不冒称所有callback是纯事实。恢复43/schema12/解析4/inputs v9保持，两个doc-hidden pub入口不授予Engine artifact绑定权。

baseline新增2事务测试后9/0；当前10目标427/0/0，late conflict付费853、末尾output cap wide1583/stopped1217/cap1582均零提交，迁移前后实测相同。1897继承+192专项完整JSON/退出码保持、零语义例外，仅规范核验后的actual engine及直属usage.elapsed_millis。专项8既有byte-array输入×目标7/8×三evidence×四预算：30wide投影/18不提升、144有限预算，不称全部到达group或所有primitive/独立holdout。168全文/78声明、136×3核心及408完整JSON保持，核心6编译12受控VM、真实两类4重编通过，真实目标未执行。2743源/归档/11产物、两成功构建绑定并清理target；601003696bytes仅构建磁盘。

baseline r1测试Debug误用、current r2错误Quality路径两次编译失败原流保留；分别定点修正后baseline r2/current r3成功，生产判据不变。native旧后处理多行匹配失败只重读原9/0流，无测试重放。BQ EM11及BP两旧native失败未重跑/豁免。下一组共享member writer，其余assert政策与2.9/2.10完整门仍开放；17/59，S1未整体验收，S0原记录保持，S2未进入。

### 2026-10-26 / BU：共享成员 artifact/block writer 职责闭合

[结果](results/s1-member-writer-source-policy-v1/README.md)、[验收](results/s1-member-writer-source-policy-v1/acceptance-r1.json)、[后续职责/依赖/验收](results/s1-member-writer-source-policy-v1/next-boundaries.md)。共享strict artifact parser、quote/block writer、精确UTF-8长度、raw marker label、current body/explanation及span mapper统一归Java，Artifact/Placed两原类型同迁；root只适配这些数据及提交。BT构造族去掉2个反向writer callback，改为第四个借用metadata getter，Java直接判断。13函数/三生产路径，无新模型/fixture/read/cache；class_source生产-243行、facade+8/Java+321行；20poll/26charge/0clone和最终poll保持。Rust-public面扩大但不授予Engine绑定/认证/提交权，恢复43/schema12/解析4/v9保持。

加同一单测的baseline root279/0，当前10目标428/0/0。72字符串组合实际UTF-8长度/annotation前缀与预算预估一致；原artifact正负/三placement/真实explanation当前正文冲突及span、BT两late failure继续通过，853/1583/1217/1582原流数值相同。2089继承完整JSON/退出码严格保持，零语义例外，仅规范核验后的actual engine及直属usage.elapsed_millis；168全文/78声明、136×3核心及408完整JSON、核心6编译12受控VM、真实两类4重编保持，真实目标未执行。72不是独立程序/报告/holdout，未新增恢复接受域。

2743源/归档与11产物、baseline1/current2正常构建绑定并清理target；601170834bytes仅构建磁盘。共同writer不重列待迁；剩余projector资格/marker事务和assert整组政策需真实消费者清点，2.9及2.10原资源/全部停止/历史失败/API双轨门保持。BQ EM11/BP两旧native失败未重跑/豁免；17/59，S1未整体验收，S0记录保持，S2未进入。

### 2026-10-26 / BV：assert整组政策与nested分类职责迁移

[结果](results/s1-assert-group-source-policy-v1/README.md)、[验收](results/s1-assert-group-source-policy-v1/acceptance-r1.json)、[后续机制/依赖/验收](results/s1-assert-group-source-policy-v1/next-boundaries.md)。完整assert准入、synthetic field/clinit/outermost、Fieldref census、全fold/current-text、clinit stripping及暂存归Java，两个nested classifier/两旧候选类型同迁；root留3借用getter与同slice position发布，去掉逐项find后可半提交的分支，真实physical index保持。4函数/三生产文件，无新模型/read/cache，原接受域/拒绝/clone及0显式poll/2charge顺序保持，恢复43/schema12/解析4/v9不变，Rust-public面扩大不授予Engine绑定权。

旧生产+同一3新测：2原子性desired测试及1含2stop情形的audit；当前11目标438/0/0，assert7原样家族重编/default禁用与-ea启用及guard/detail效果顺序通过。late conflict691及wide1101/stopped1099/cap1100均无半提交。2089继承+96专项完整JSON/退出码严格保持，零语义例外；12wide投影/12不投影、72有限预算不冒充全部到达阶段或holdout。168全文/78声明、136×3及408完整JSON、核心6编译12受控VM、真实两类4重编保持，真实目标不执行。已有A1资产首次进入include清单，baseline/current2744（BU2743），无新fixture文件；源/归档/12产物和两当前成功构建绑定，target清理，616275429bytes仅构建磁盘。

**已实测但未修的停止缺口**：budget/cancel scanner分别Err，private assert边界均refused；emitter output stop仍为Unsupported assert_projection_stopped。S1_stop_gate=false，不能用438/0覆盖。outermost无预算/环检测仅静态风险，未执行环/变异目标，未外推Engine假Complete。下一组先闭合typed stop/outer链有界化，再清其余projector/raw消费者与2.10原资源/全部故障/API门。BQ EM11/BP两旧native失败未重跑/豁免；17/59，S1未整体验收，S0原记录保持，S2未进入。

### 2026-10-26 / BW：assert 类型化停止与 enclosing 链有界化

[结果](results/s1-assert-stop-boundary-v1/README.md)、[收据](results/s1-assert-stop-boundary-v1/acceptance-r1.json)、[六门](results/s1-assert-stop-boundary-v1/six-gates.md)、[后继职责/依赖/验收](results/s1-assert-stop-boundary-v1/next-boundaries.md)。复用既有失败通道，scanner 不再吞 budget/cancel，emitter 保留 typed stop，root 合并状态；outermost 每行计费并有限拒绝环/重复出边，成功计划前 poll。无新生产模型/read/cache，恢复43→44。441/0/0，四个实际公开边界控制保持五个已完成成员、逻辑Body数与六阶段实际IR starts；ir_items的13点暂存增量不伪装IR启动。2089+408完整报告相等；assert96中78相等、18仅明确的analysis_steps增加；168全文/78声明/136×3核心及核心6编译12受控VM、真实两类4重编保持。

r2新测试把逻辑ir_items当实际IR启动而失败，修正仅在cfg观测，r3生产代码与r2逐字相同、三构建通过；失败原流及68次CLI探索均保留，CLI探索没有复现旧公共假Complete。第三轮历史六门保持，不豁免旧native失败，不作全S1停止/资源通过。下一组迁移整数常量名称投影的完整资格/span/暂存事务，其后enum-switch同方法整组及匿名族专用事务；依赖同次facts、原子性及效果/完整输出验收，不按单样例拆补丁。2.9/2.10未勾选，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / BX：整数常量名称资格、placement 与整组暂存归 Java

[结果](results/s1-integer-name-source-policy-v1/README.md)、[收据](results/s1-integer-name-source-policy-v1/acceptance-r1.json)、[六门](results/s1-integer-name-source-policy-v1/six-gates.md)、[后续任务/依赖/验收](results/s1-integer-name-source-policy-v1/next-boundaries.md)。Java持有整组/方法资格、原descriptor/AST匹配、prior staged冲突、专用text/span、唯一源范围与字段匹配、普通整类重放证书和发布前计费暂存；root只适配同次事实/锚点、原物理组装器及最终无失败发布，外层family编排保持。5函数政策、3生产文件，无新模型/read/cache；恢复44/schema12/解析4/v9不变。基线同4新测试4/0，当前16目标463/0/0，4公开控制前缀与实际reader Body decoder及六阶段IR starts零增量；late conflict/output/cancel无半提交，重复needle/重复调用保持。

2185直属报告（含BW96 assert）+96整数专项+408核心全JSON/退出码保持，零语义/计费例外；168全文/78声明/136×3核心及核心6编译12受控VM、真实两类4重编保持。额外匿名/double-brace相邻4目标18项覆盖共享证书，不混计JDK分母。elapsed仍Unsupported(jre_budget_interrupted)已实际复现，单列2.10共享错误消费闭包，不将4/0视为停止类型正确；原历史失败不豁免。名称Vec在stage返回时提前drop、重放String在投影组装前drop仅静态范围，不充资源门。r2主动终止/r3编译命名空间失败原流保留，r4三构建通过，2744源/资产及最终17二进制绑定，target清理。下一组enum-switch同方法整组事务，共享Interrupted原因独立修复；2.9/2.10未勾选，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / BY：枚举同方法整组政策与相邻停止守卫

[结果](results/s1-enum-group-source-policy-v1/README.md)、[验收](results/s1-enum-group-source-policy-v1/acceptance-r1.json)、[六项](results/s1-enum-group-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](results/s1-enum-group-source-policy-v1/next-boundaries.md)。Java接管完整方法分组/资格/extra-use/同方法拒绝/marker/AST emission/placement/整类克隆暂存和纯map label/coverage，三个既有模型同迁；root供事实/跨类读取IR、执行诊断及最终发布，无新生产模型/read/cache。helper准备/source资格和shared-helper table-set仍待审，不计已迁。

先复现预算被后续cancelled覆盖（baseline3/1、第一层守卫fixed-r3仍3/1），补enum后的ended、array与initializer入口守卫后fixed-r4=4/0，再迁移。恢复44→45；当前16目标467/0/0，固定内联/迁移后三marker记录相同，真实两map之后停止保留原方法/markers、无proof投影，实际Body decoder及六IR starts零增量；output/cancel付1257、elapsed0。2281继承中2275全JSON相同/6份停止修正、168枚举中126相同/42份header2停止修正、408核心相同，退出码全部保持；48份差异逐项审查，包括明确Refused、停止后IrItems减少及3个compact交付requested+68，未屏蔽费用字段；168全文/78声明/136×3核心、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2744源/资产/归档与17产物绑定，r7三构建target清理，r5/r6主动终止及失败原流保留。第三轮历史六项保持原限制；共享Interrupted、bridge等全部停止、raw闭包、实际资源/API与旧失败仍开放，2.9/2.10不勾选，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / BZ：enum helper准备与shared-helper整组职责归Java

[结果](results/s1-enum-helper-source-policy-v1/README.md)、[收据](results/s1-enum-helper-source-policy-v1/acceptance-r1.json)、[六项](results/s1-enum-helper-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](results/s1-enum-helper-source-policy-v1/next-boundaries.md)。Java持有完整selector/ordinal/helper/enum/values与常量身份准备，及helper table-set、同table enum一致性/key合并、单map/group证明选择和结果展开；原Prepared记录同迁，root只供原scope/target、ConfirmedRead借用facts与实际resolve/read/IR/执行合并和按index收集，无新模型/trait/read/cache。两pub政策入口不提供Engine artifact绑定权，恢复45/schema12/解析4/v9不变。

原政策同6测试6/0；当前16目标469/0/0，新两控制实际closed-set在IR前Body/六IR全0，定义冲突在一次共享IR后拒绝，Body1/六IR各1，与正例相同；无方法/marker半提交，三marker停止记录保持。2449继承（含BY168枚举）全JSON/退出码相等、零语义/计费例外，不重复跑矩阵；168全文/78声明/136×3核心与408全JSON、核心6编译12受控VM和真实两类4重编保持，真实目标未执行。BY raw summary false/48份修正按封存acceptance核对，不改历史。2744源/资产/归档、17产物与正常构建绑定，target清理。helper IR报告型停止消费/共享Interrupted及其余全停止/资源/API、raw与匿名专用族/旧失败仍归2.9/2.10，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / CA：enum helper typed-stop 与后继需求截断

[结果](results/s1-enum-helper-stop-v1/README.md)、[收据](results/s1-enum-helper-stop-v1/acceptance-r1.json)、[六项](results/s1-enum-helper-stop-v1/six-gates.md)、[后继任务/依赖/验收](results/s1-enum-helper-stop-v1/next-boundaries.md)。复用原DependencyProofFailure Error/Report，Java三政策入口透传泛型失败，root opt-in read与IR回调在全局停止时传原execution/diagnostics，外层消费一次；形状拒绝仍inner String、局部接受域保持，无新模型/trait/read/cache。恢复45→46/schema12/解析4/v9保持，不捏造报告缺失的Error quota。

baseline r2同7测试6pass/1fail，完整复现四Report路径2helper请求及停止后raw IR+1、三个预算首因被Cancelled覆盖、原报告/诊断丢失；两Error路径原正确。当前16目标470/0/0，六控制均1helper请求、停止后实际Body/六IR全0，无method/marker/proof半提交，四原Report/诊断保留，两原Error及BY三marker/BZ两helper保持。r1 proof控制误用AnalysisSteps未触发，第五断言失败/第六未跑原流保留；r2只改IrItems控制，不改预期。2449继承（含168枚举）中2407全JSON相同/42份Header停止metadata修正，408核心相同，退出码全部保持；42份仅追加原预算诊断与明确停止拒绝，正文/身份/execution/usage不变、零未审查差异，168全文/78声明/136×3核心及核心6编译12VM、真实两类4重编保持，真实目标未执行。2744源/资产/归档与17产物绑定，target清理。共享Interrupted、其他后继/read切点/资源/API、raw/匿名族及旧失败仍开放，17/59，S0原记录保持，S2未进入。

### 2026-10-26 / CB：共享成员放置政策归 Java

[结果](results/s1-anonymous-member-placement-v1/README.md)、[验收](results/s1-anonymous-member-placement-v1/acceptance-r1.json)、[六项](results/s1-anonymous-member-placement-v1/six-gates.md)、[后继职责/依赖/验收](results/s1-anonymous-member-placement-v1/next-boundaries.md)。八个关联body/span/匿名成员方法的声明/marker/注解准入、child flags、默认缩进、root-return/current-prefix放置与UTF-8范围核验归Java；root仅供原字段/Recovered变体事实并构造原index/kind/anchors，私有parts移除，无新模型/trait/read/cache。各接受域差异、clone/文本顺序和预算/发布保持，恢复46/schema12/解析4/v9不变。高层匿名interface/nested/super/double-brace候选、Header/owner/capture资格与专用暂存/冲突决策仍在root逐成员编排中，不能称整族已迁完；物理提交继续留root。

原政策同三新边界测试3/0；当前16目标473/0/0，原11政策/停止控制完整保持。2449继承（已含168枚举）及408核心全JSON/退出码严格相等，零语义/计费例外；CA42份metadata修正沿接受基线核对，不改其原false流。168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编通过，真实目标未执行；新测试只复用旧JAR和布局事实变体，不计新恢复或holdout。2744源/资产、归档、17产物与正常构建绑定，target清理，磁盘仅构建护栏。第三轮历史六项与原JDK8/旧native失败限制保持。共享Interrupted、其他停止/资源/API、raw闭包及高层匿名族仍按2.9/2.10承接，17/59，S1未整体验收，S0原记录保持，S2未进入。

### 2026-10-26 / CC：匿名接口跨 IO 阶段资格与 emission 归 Java

[结果](results/s1-anonymous-interface-source-policy-v1/README.md)、[验收](results/s1-anonymous-interface-source-policy-v1/acceptance-r1.json)、[六项](results/s1-anonymous-interface-source-policy-v1/six-gates.md)、[后继职责/依赖/验收](results/s1-anonymous-interface-source-policy-v1/next-boundaries.md)。site/root-child typed nesting、capture参数与返回类型、平台/实际接口契约、构造器/完整方法/scan闭集和捕获AST重写—child文本—root匿名返回emission归Java；共享唯一allocation/nested-site/complete-method政策及三个既有模型同迁。root仍负责实际resolve/read/IR准备、执行合并、锚点/物理装配和发布；flags/facts/bytes getter只借用原数据，placement调用CB已迁政策，无新DTO/trait/read/cache，恢复46/schema12/解析4/v9保持。共享capture/owner census仍含政策、nested/super/double-brace专用决策仍待迁，不称纯IO或整族完成。

旧政策同三新测试3/0，当前16目标476/0/0；4正例/7早晚拒绝/3停止共14控制逐项保持，原11控制保持，三停止后实际Body/六IR全0且无半提交；elapsed仍Unsupported(jre_budget_interrupted)，单列2.10，不冒称首因正确或全部read点通过。2449继承+20匿名profile（5既有JAR×source7/8×essential/all，40CLI）=2469完整报告/退出码严格相等，408核心相同，零语义/计费例外；168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编保持，真实目标未执行。新控制只用既有冻结class/JAR和内存facts变体，不计恢复增量/holdout。r1选错capture夹具2pass/1fail原流保留，r2修正输入不改断言；2744源/资产/归档、17当前产物及正常构建绑定，target清理，磁盘不充运行资源。临时row/definition所有权范围提前仅静态观察，S0原门/历史失败不豁免。2.9/2.10/S1开放，17/59，S2未进入。

### 2026-10-26 / CD：完整共享 capture 证明簇归 Java

[结果](results/s1-member-capture-source-policy-v1/README.md)、[验收](results/s1-member-capture-source-policy-v1/acceptance-r1.json)、[六项](results/s1-member-capture-source-policy-v1/six-gates.md)、[后继任务/依赖/验收](results/s1-member-capture-source-policy-v1/next-boundaries.md)。member_capture模块接管完整Outer/instance/double/val捕获、constructor参数分区、local单写、SSA使用/唯一写/MethodHandle/exception及两receiver域，两个原类型同迁，root仅重导出同一实现；anonymous闭表/Code预检、method调度与三分流归Java，root callback留实际analyze/执行合并/可选原Report relay。无镜像DTO/trait/runtime/cache/read，恢复46/schema12/解析4/v9保持；一般family/instance IR编排、call-site/bridge/owner census与专用匿名族仍待迁，不称整族完成。

原代码同三测3/0，当前16目标479/0/0；新14控制（2成功/4早拒/8真实IR stop）与原25控制逐项保持。四早拒wrapped-reader/Body/六IR为0；八stop均1请求、停止后三类计数全0。legacy四次Ok(None)无原Report/diag sink已复现，opt-in四次原execution/diagnostics保留；两种局部execution均已合并，尚未证明公开假Complete或修typed消费者，2.10继续开放。2469继承全JSON/退出码及408核心严格保持，零语义/计费例外，含168枚举/20匿名子集不另跑矩阵；168全文/78声明/136×3及核心6编译12受控VM、真实两类4重编保持，真实目标未执行，不计新holdout。2745源/资产（仅新增机制模块）、归档/17当前产物与构建绑定，target清理。r1 lib递归格式化三无关文件被scope门拦在Cargo前并按封存字节恢复；r2仅旧单测缺两helper导入而编译失败，r3补同实现别名/隐藏pub，不改证明或断言，原流保留。S0原门/历史失败不豁免，2.9/2.10/S1开放，17/59，S2未进入。

### 2026-10-26 / CE：capture 原始停止报告消费者闭包

[结果](results/s1-capture-stop-consumers-v1/README.md)、[验收](results/s1-capture-stop-consumers-v1/acceptance-r1.json)、[六项](results/s1-capture-stop-consumers-v1/six-gates.md)、[后继任务/依赖/验收](results/s1-capture-stop-consumers-v1/next-boundaries.md)。统一 capture 依赖的实际 IR/原始停止返回通道。Java anonymous capture 调度透传泛型供应方错误；Root 复用同一 DependencyProofFailure，集中实际分析、执行合并及原 Report/diagnostics relay，anonymous/val 的 optional sink 与三个 legacy/strict 包装删除。普通 member、instance、匿名 interface/nested/super 和 double-brace constructor partition 的公开消费者均接收同一通道；源码形状拒绝保持原域，capture certificate 判断未改变。恢复46→47，schema12/解析4/inputs9，无新DTO/Runtime/cache/read。

先在原代码同三测试复现2pass/1fail：48公开控制中8充分预算正例、8后置proof Error及double-brace两路8个Report stop原已通过；其余24个Report stop失败。member/instance原总体停止类型正确但原diag丢失，interface/nested/super另有停止原因被Unsupported覆盖；没有公开假Complete证据。修后16目标482/0/0；48控制全通过，32真实IR Report逐字消费原execution/diag一次，8 proof Error保留AnalysisSteps，均保持已发布源码/方法prefix，停止后actual wrapped-reader calls/bytes、Body decoder及六IR starts为0。原25政策控制保持，CD原6成功/早拒与4 strict报告保留，删除4 legacy可选sink控制，现CD10；不是新增独立程序/holdout。

2469继承全JSON/退出码严格保持，含168枚举与20匿名profile子集，无额外矩阵CLI；408核心全JSON保持，零语义/计费例外。168完整源码/78声明、136×3核心、核心6编译12受控VM、真实两类4重编保持，真实目标未执行。2745源/资产、归档与17当前产物及三正常构建绑定，target清理；baseline-r1切点修正和current-r2静态scope纠正均主动终止，原流/归档/exit -15保留，无native运行。构建磁盘护栏不作S0运行资源证据。

共享capture证明簇已归Java，typed供应/消费闭包现补齐；剩余ordinary/instance方法需求编排、owner census、call-site/bridge与nested/super/double-brace专用资格/暂存事务仍按机制迁移。shared emitter Interrupted、其他read/bridge/sink/资源/API及旧native失败保持开放，第三轮历史六项原JDK8/失败限制不豁免。2.9/2.10未勾选，OpenSpec17/59，S1未整体验收，S0原记录保持，S2未进入。

### 2026-10-26 / CF：普通 member capture 需求政策归 Java

[结果](results/s1-member-capture-demand-policy-v1/README.md)、[验收](results/s1-member-capture-demand-policy-v1/acceptance-r1.json)、[六项](results/s1-member-capture-demand-policy-v1/six-gates.md)、[后继依赖/验收](results/s1-member-capture-demand-policy-v1/next-boundaries.md)。承接CE已封存的capture原始Report消费者闭包，继续迁移普通member capture的方法需求政策。Java member_capture::prove_member_capture接管原Vec::new、物理methods原顺序、每项先poll再筛Code、PhysicalMethodId构造、IR身份/Complete校验、IR借用及同一prove_family_capture调用。Root仅保留同一实际analyze_capture_dependency callback、typed停止/执行diag合并与ClassSourceMemberCapture枚举拼装。无新DTO/trait/Runtime/cache/read，原拒绝文案、严格六指令Outer证明域、计费/poll/clone顺序保持；所有旧Java证明与anonymous pipeline、Root其他函数/测试逐字未动。恢复47/schema12/解析4/inputs9保持，instance的普通构造体扩展域未混入。普通证明的Root重导出现仅测试使用，新增unused import警告已逐项对照记账，随member_inner剩余调用者/重导出收口清理，不声称零警告。

当前16目标482/0/0，原25政策、CD10 typed与CE48公开控制共83份完整记录严格保持；48包括8正例、32真实IR Report及8 proof Error，原报告/诊断一次消费、prefix与实际停止工作截断保持。CE原代码2/1和24公开Report传递失败、修后全通过的原证据不改，不重复算新恢复/holdout。本轮无新测试/目标输入。

2469继承全JSON/退出码、408核心全JSON严格相同，零语义/计费例外，含168枚举/20匿名profile子集不另跑矩阵；168完整源码/78声明、136×3、核心6编译12受控VM及真实两类4重编保持，真实目标未执行。2745源/资产、归档与17当前产物及三正常构建绑定，target清理。r1脚本parent路径预检错误发生在Cargo/native启动前，原流保留；r2纠正路径后正常构建，不改生产代码。构建磁盘不充S0运行资源。

下一组依次处理instance需求编排及IR持有期、owner census/call-site/bridge资格与nested/super/double-brace专用事务。共享Interrupted、全部read/bridge/sink停止/资源/API和历史native失败仍归2.9/2.10，不能用482/0覆盖。第三轮六项原JDK8/历史限制、S0记录保持；2.9/2.10/S1开放，OpenSpec17/59，S2未进入。

### 2026-10-26 / CG：instance capture需求与IR持有期

[结果](results/s1-instance-capture-demand-policy-v1/README.md)、[验收](results/s1-instance-capture-demand-policy-v1/acceptance-r1.json)、[六项](results/s1-instance-capture-demand-policy-v1/six-gates.md)、[后继依赖/验收](results/s1-instance-capture-demand-policy-v1/next-boundaries.md)。Java member_capture::prove_instance_capture接管instance物理methods的原顺序需求、每项先poll再筛Code、PhysicalMethodId、IR身份/Complete门与同一prove_family_instance_capture调用；Root保留物理重读、实际IR callback/typed执行诊断、fold target/bridge/census与提交。原Vec::new、拒绝文案及instance capture refused前缀、允许普通构造体/实际capture write BCI的扩展域保持，不误用普通member六指令证明。所有旧Java证明/ordinary/anonymous政策及Root其他流程逐字不变；普通proof的Root导入限定cfg(test)，instance proof不再Root重导出，CF新增unused import已清。恢复47/schema12/解析4/inputs9保持，无新产品DTO/Runtime/cache/read。

先增加仅test-support的真实MethodIr构造/drop计数，正常构建无该字段/状态；已有事件Counts和产品报告不扩展。基线同两新测试1pass/1fail：完整N1捕获后的2个IR及2个ClassFacts在proof、bridge、census到done仍存活，returned才归0；五种实际IR/后置proof停止已释放且停止后实际reader calls/bytes、Body、六IR全0。迁移后Java只返回自有MemberCaptureProof，分析/借用缓冲在证明返回前释放；同一五阶段IR和ClassFacts均2→0，五stop保持、所有六控制通过。ClassFacts为值载荷数而非物理类数/Arc引用数；只证明该受控路径的对象持有期，不当heap字节、RSS/时间或完整S0资源改善。

当前17目标489/0/0：Root308、原14 integration174、Java assert2、追加既有inner_class_static_mixed_folding5；其中只有两项是本轮新测试，五项为已有整族源码/重编/受控行为/停止回归。原83完整控制记录严格保持，CG6的请求、身份、执行、diagnostics、usage及fold全文保持；仅normal五阶段的两个存活计数变化逐点保存，原预算/取消与已发布prefix保留。复用旧N1三class及原fixture，不计新holdout或恢复增量。

核验实际engine、registry.jvm来源指纹并仅规范直属usage.elapsed_millis后，2469继承完整JSON/退出码与408核心完整JSON保持一致，零语义/计费例外；原r1规范下377份相同、2092份共6538处仅JVM来源身份变化，均与父/当前实际Cargo绑定；r2复用原2469次CLI输出，不增加CLI。168枚举/20匿名profile已含，无额外矩阵CLI；168源码全文/78声明、136×3、核心6编译12受控VM和真实两类4重编保持，真实目标未执行。2745源/资产、归档、18当前产物与三正常构建绑定，target清理。r1迁移静态断言误拒report.execution读取，source scope门在Cargo前拦截未完成转移，原脚本/错误和空目录保留；r2修正断言后构建，无证明或测试断言调整。观测基线与迁移后用同一测试源码，21项纯文本迁移核对及正常CLI无test-support features验证通过。两次审查脚本误判及原false输出、纠正收据分列保留，不改生产/测试断言；构建磁盘不充运行资源。

下一组owner census/自调用准入与call-site/bridge资格，然后nested/super/double-brace完整资格及暂存事务；shared emitter Interrupted、其余read/bridge/sink停止、资源/API、raw消费者闭包和历史native失败仍开放。第三轮六项原JDK8/历史限制、S0原记录保持；2.9/2.10/S1未关闭，OpenSpec17/59，S2未进入。
