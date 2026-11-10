> **当前规划（2026-10-26修订）**：推进顺序为 `S1有限退出 → S2a源码变量合法化 → S3a顺序恢复基线`；S2b、S3b和独立职责清理不作为S3a前置。S1有限范围已验收，S2a已按A2.5有限域验收，主线33/65；S3a的4.1/4.2/4.5/4.6组件已完成，4.8阶段门待验，全部架构尚未完成。证据及未完成项见[当前检查点](results/s2a-variable-legalization-v1/reference-constraints-checkpoint.md)。带日期历史记录不改写当前执行顺序，旧17/59等计数保留历史含义。

> 历史 CG 局部验收（规划修订前口径）：[instance capture需求与持有期（2026-10-26，CG）](results/s1-instance-capture-demand-policy-v1/README.md)。当前489/0/0、83旧控制及2469全JSON/408核心保持；实测proof后2 IR/2 ClassFacts归0，恢复47，2.9/2.10开放，17/59，S2未进入。



## Context

动机与范围见 [proposal.md](proposal.md)。本设计基于 2026-10-20 的静态审查，源码基线为 `30464b083`；最近产品验收与历史任务状态仍以 [handoff](../../../handoff.md) 为准。以下内容是拟实施设计，本文没有产生新的恢复、编译或行为通过率。

本次依据[对抗性审计复核稿](../../../docs/tmp-architecture-adversarial-audit-2026-10-20.md)同步方案。2026-10-20 **S0 已按声明测量域验收**，见 [冻结结果](results/s0-observed-v3/README.md)。用户随后授权继续 S1，已完成 2.1–2.8，见 [职责/evidence 验收](results/s1-core-v1/README.md)、[需求复用验收](results/s1-demand-v1/README.md)与[状态/停止验收](results/s1-demand-state-v1/README.md)与[源码目标验收](results/s1-source-target-v1/README.md)与[环境/loader 验收](results/s1-environment-v1/README.md)及[语义绑定验收](results/s1-semantics-v1/README.md)及[输入来源验收](results/s1-trust-v1/README.md)；S1 阶段门未通过，S2 未进入。产品失败与未覆盖域继续保留。本文的 `D1–D11` 是设计决策编号，审计编号一律加“审计”前缀，避免混淆。

当前可复用的基础已经明确：不可变 snapshot、PhysicalDefinition/MethodId、显式 runtime/loader 环境、HeaderClosure、PreparedClassRead、CanonicalCFG、Frame/SSA/effects、Region、Java AST、预算及独立结果平面。审计发现的压力是这些事实在恢复层被重复解释：

| 证据入口 | 当前事实 | 调整落点 |
| --- | --- | --- |
| `src/facade.rs::proved_reference_widening`、泛型调用证明 | 门面执行源码语义与平台特例判断 | S1/S2：门面供给事实，Java 层作源码决定 |
| `crates/jarde-java/src/report.rs::recover_inner` | 多个候选输出与保留开关贯穿方法、类级入口 | S1/S5：按阶段产物组织内部结果，能力由事实与请求范围决定 |
| `build.rs::decide_types`、`release_reference_argument_widens` | 已有局部规划与有限传播，类型关系还分散 | S2：扩展已有规划的约束域和共同关系查询 |
| `region.rs::Walker`、`LoopBodyTrialState`、`finally_body_supported` | 组合依赖 walker 上下文和专用允许表 | S4：共享归属与出口合同，迁移一个完整组合域 |
| `build.rs` 的 postfix 消费记账和整方法降级守卫 | 已吸收效果可能因消费者拒绝而失去呈现位置，现有守卫负责阻止错误交付 | S3：转换提交前核对效果与消费闭合，保留守卫 |
| `pass.rs::RecoveryProfile` | 源码门控复用 RuntimeProfile | S1：独立输出配置 |
| `jarde-query/src/query.rs::analysis_of` | 独立 query 的两类语义关系仍为 UnsupportedAnalysis | S6：显式组合操作连接现有解析能力 |
| `fuzz/fuzz_targets/method_analysis.rs` | fuzz 只到方法 IR | S0/S3/S5：把 Java 与类级呈现纳入受控检查 |

这些是职责和能力边界证据，不能仅凭文件行数推断实现有错。主规格中 `Independent analysis planes`、`Bounded Header closure`、`BCI-to-source evidence` 和独立验证状态继续适用。

审计新增的确定事实是：`EnvironmentIdentity` 没有保存非调用方 domain 的完整配置；binding 使用包版本、注册 major 上限及规则表，不能自行证明实现语义相同。环境身份遗漏导致旧 expected artifact 被误认的端到端反例已在 2.6 复现并修正；独立语义版本的同包版本/同文本反例已在 2.7 用实际构建复现并修正，源码身份与合同 revision 分开记录。接口 super 已在方法入口证明；类级局部 derived spans 已存在；公开底层 proof 类型也不等于 Engine 存在任意注入路径。方案以这些修正后的证据为准。

## Goals / Non-Goals

**Goals:**

- 一个共同机制覆盖一组输入形态；每个阶段同时给出适用域、拒绝边界、真实包影响和未解决范围。
- 物理事实、运行时解释、源码表示分别保有身份。查询和源码消费同一事实，源码改写不反向覆盖原始引用。
- 保守 Java 与高级语法共享正确性约束；局部糖化失败可以保留已证明的顺序表达，全局取消与预算停止仍立即传播。
- 阶段迁移保持当前已验收能力，遇到已知错误则显式更正并记录，不以文本稳定性保留错误。

**Non-Goals:**

- 本轮不构建新的通用 IR。复用 CFG/SSA/Region/AST，仅为实际需要增加内部记录。
- 第一轮类型域不含完整 JLS 泛型方法推断、任意 wildcard capture 或 intersection 求解；区域域不含任意不可约图与任意交叠异常范围；类族域不保证外部未知调用者闭合。
- S1 先支持明确的源码目标 7/8；已有解析和运行时选择能力按自身支持矩阵保留，不借此宣称新版 classfile 或全部 Java 7/8 语法都已恢复。未注册源码目标明确拒绝。
- 补全“事实缺失”的路径只读取调用方提供的输入；目标执行仅出现在明确选择的受控测试中。

## Decisions

### D1. 保留固定分析层次，以请求内事实连接各层

```text
Explicit operation + Snapshot + RuntimeView + Budget
    + SourceTarget only when source delivery is requested
    |
    +--> Physical query / selection
    +--> Header / declaration resolution
    +--> Selected body -> CFG / Frame / SSA / effects
                            |
                            v
                    Java source decisions
                    types / regions / expressions
                            |
                            v
                    method or class projection
                            |
                            v
                    source + origins + gaps
```

上述分支由请求的阶段集合决定。纯物理查询不进入 resolver 或恢复；纯方法恢复也不建立全包 XRef。固定请求编排由现有 Engine 承担，核心 crate 不依赖 CLI 或 host 协议。

S0 已复现一个跨层耦合：类族 static fold 的无 projected emission 路径使用公开 SourceMap 作为必要 BCI 证明，而该 map 仅在请求可选 evidence 时物化；essential 因而丢失有效嵌套类，all/source_map-only 成功。两个编译器和受控行为对照确认差异具有恢复影响，见 [诊断](results/s0-observed-v3/evidence-selection/analysis.md)。S1 / 2.2 必须分开内部按需证明与对外 evidence 发布，S5 继续验证类族共同提交；保留现有有效成功，不以强制 all 或常驻完整 AST 解决此问题。

| 所有者 | 输入与输出 | 允许做的决定 |
| --- | --- | --- |
| reader | bytes → 不可变物理事实 | 解码、边界、物理身份 |
| query | physical scope → 引用候选与范围 | 已声明关系的结构匹配 |
| jvm/resolver | 环境、定义、方法 → Header/CFG/SSA/effects | 类选择、声明解析、有界派发和 JVM 事实 |
| java | 事实、源码目标 → 源码决定、候选、AST | Java 类型、结构、转换与呈现 |
| facade | 请求、预算 → 各阶段调用与报告 | 供给已请求事实、复用、停止和交付 |

内部方法结果按现有阶段产物组织，候选持有物理方法与 BCI、所依赖事实、预期改写范围；按请求保留需要的候选。避免继续为每个语法向 `recover_inner` 增加一个可变输出参数。证明构造与消费先归同一所有者，再缩小内部类型可见性；不能在跨 crate 调用仍存在时机械改为 `pub(crate)`。公开底层事实入口只保留真实消费者需要的参数，对方法/facts/subject/profile 的矛盾拒绝或隔离为不可用于可信绑定的来源，不把调用方断言伪装成本次读取所得。具体类型不预设新的 token、公共 Session 或任意类型容器。

跨入口一致性比较相同事实、完整环境、源码目标和等价词法/投影上下文下的语义决定。独立方法路径可比较同一正文；类源码的缩进、声明折叠、匿名投影与 map 坐标允许有明确来源的差异。可选 evidence 不改变已决定源码；低预算下证据物化和主体产物各自报告停止，不能要求工作量不同的请求 usage 相等。

恢复边界保留 MUTF-8 原始名称/descriptor 到精确匹配；合法 supplementary 名称按 JVM 编码解码，源码不可表达名称采用已有确定性别名及 raw evidence，损坏编码明确拒绝。`from_utf8_lossy` 只能用于可标识的诊断展示，不能反过来决定成员/类型身份。

备选是只拆文件或建设动态 pass 图。前者不改变语义归属，后者增加调度和失效复杂度；先迁移真实职责与固定阶段即可。

### D2. 按需上下文记录“需要什么、为什么、得到了什么”

继续使用 HeaderClosure、ReadReason、PreparedClassRead 与同次请求的预算。Java 层提出有限需求，门面通过 resolver 供给。需求包括类型声明/父接口、成员签名、某个已定位 synthetic helper 的 Body、某个类族候选的创建点范围。Body 需求必须绑定具体物理成员及 reason，禁止因类型关系查询遍历全部 Body。

需求结果区分：已有事实、完整搜索后缺失、歧义、关系仍未知、执行停止。前三者都不能冒充不同定义的同名事实；执行停止不能变成“不是这个模式”。同次操作中的相同请求复用已有不可变结果，候选事实不足时保留调用或声明，提供可定位缺口。

类型与成员身份使用定义 loader 和物理定义，符号名只作为解析输入。平台知识由同一事实供给边界提供：可以复用已提供的 JDK headers，也可以消费版本固定、适用环境明确的现有有限平台表。平台表不得覆盖实际选中的输入定义，也不得把没有提供的类伪造成已读取物理 Header。

候选需求图以物理身份与需求类别去重，读入同一事实不触发无限重试；规则工作队列和层次遍历沿用现有计费、深度及取消轮询。缓存只保留现有物理层，本提案先验证请求内复用。循环需求的“正在求解”与完整搜索后的缺失分开；同 bytes 的共享事实不合并不同 origins。完整物理事实、环境绑定的解析结论、方法 IR 和源码产物分别有自己的复用前提，不把 SourceTarget 加进物理 parse key。

同次请求去重不得依赖跨请求 cache 存在；禁用/冷/热/容量不足路径在足够预算下语义相同，受限预算下完成范围可不同但停止仍真实。局部候选拒绝、成员损坏与全局停止分开：全局停止后不得启动下一个 Body，也不得把 incomplete/Unknown 写成负缓存。现有只保留完整物理事实的原则继续保留，不新增派生结果缓存。

### D3. 分开输入版本、运行时环境、输出源码与验证

请求中的源码配置记为 `SourceTarget { java_release }`（设计名，最终类型按仓库命名确定），独立于 RuntimeProfile。输出报告同时回显二者。SourceTarget 进入恢复请求、恢复/类报告、产物绑定及明确请求源码的组合计划；纯 JVM 分析和物理事实不增加无关依赖。源码目标只限制呈现语法，不决定 MR entry、类搜索顺序或平台 API 身份。

| 维度 | 负责的问题 |
| --- | --- |
| classfile major/minor 与 feature registry | 可否解析相应格式 |
| dialect/指令校验 | 所读字节是否满足对应规则 |
| runtime view 与 loader | 选哪个物理定义、如何解析依赖 |
| source target | 此次允许输出哪些 Java 语法 |
| verifier / compiler / differential evidence | 实际执行了什么验证 |

S1 验证同一 Java 8 字节码在源码目标 7/8 下物理身份一致。目标 7 无法表达的 lambda 等语法只有在存在已证明替代时才转写，否则降级；不能仅改报告中的版本。编译验收使用与目标匹配的 source/release 设置及声明的依赖，不通过改目标来绕过失败。

备选是继续复用一个 release。它会把输出偏好和输入选择绑在一起，所以本阶段明确修改 API，不保留长期双轨配置。

### D4. 在现有声明规划中统一源码类型约束

实施先后按 D12：S2a 先完成变量合法化及其必要类型/调用约束，S2b 再扩展已声明泛型；本节完整目标保留，不能把全部泛型域作为 S3a 前置。

先确定源码变量的分组，再为每个变量决定一次类型。物理槽、SSA 值和源码变量有不同生命周期；只有 SSA 使用与作用域证明两个生命周期独立时才拆变量，不能靠改名消除类型冲突。

约束源包括描述符、经校验的泛型 Signature、赋值、全部使用点、phi、方法调用参数/返回和同次 Header 关系。求解结果与原始 JVM 类型并存。引用类型用解析身份表示，格式化到 Java 名称发生在呈现阶段。

本轮闭合的域：

- null 与具体引用合流、引用数组、显式声明的父类/接口可赋值关系；
- 参数化类/接口的已声明类型实参沿继承路径替换；覆盖 `List<T>` 与自定义泛型容器，避免把库类名写进恢复规则；
- 字段、局部、参数和返回的同一约束共同消费；重载检查绑定最终发布的声明；
- 沿用已有 boolean/char/窄数值转换规则，`0/1` 独立出现仍不能推导原始 boolean 类型；
- 未决或冲突留有事实原因，允许经证明的必要显式转换；不通过任意 cast、Object 替换或泛型擦除伪造成功。

推导以有界工作队列收敛，先收集约束再发射。有限平台关系属于事实后端，所有消费点走统一 assignability；同一输入事实下，不因入口或分支遍历顺序改变声明。JADX 的 SSA use bounds 与 phi 合并提供算法参照，保留我们的预算、身份和缺事实语义。

变量合法化属于同一个 S2 闭环：声明支配全部使用、每条到达边满足 definite assignment，phi 边赋值保持同时复制语义（循环交换值需要必要临时值）；catch 参数、循环变量和捕获各守自己的作用域，宽/窄槽复用按生命周期分组。LVT/debug 只提供名称线索，缺少 debug 不能使值流失效；不能通过默认初始化或改名掩盖缺入边。

备选是全面接入 Java 编译器级求解器，或继续增加参数类型特例。当前域已有足够事实可复用，完整 JLS 求解超出目标，特例会继续分裂声明与使用规则。

### D5. 先有顺序表达，再原子提交表达式转换

按 D12 分开 S3a 顺序基线与 S3b 糖化验收。首个恢复收益须来自能原样编译/执行的顺序表示，不等待所有表达式压缩策略迁移。

在已有 AST builder 中保留可合法呈现的顺序语句及必要临时变量。候选保存：消费的 SSA 定义/使用、吸收的效果、输出位置、可能抛出位置及 origin 映射。检查通过后一起提交；拒绝则回到同一候选开始前的有效表示。

一次提交必须回答：原来每个可观察操作现在在哪里执行、执行多少次、在什么控制/异常边界内执行。普通内存读、调用、可能抛出的操作不能因看似“纯表达式”而重复计算或跨 handler 移动。字段/数组地址及旧值需要分别保存；volatile、monitor、类初始化、构造器未初始化对象相关行为继续受现有强约束，不依靠普通临时变量规则放宽。

首批域覆盖局部前后缀更新、赋值值、多消费点、字段/数组左值及普通调用参数的顺序组合，窄数值截断必须在原语义位置保留。按候选实际涉及的操作核对 null/越界/cast/调用抛错位置，byte/short/char 截断、boolean 上下文、装箱拆箱及整数溢出；涉及浮点的搬移/合并须保留 NaN 与有符号零，未支持操作保持边界。短路与三元分支中的操作保持在原分支内，不能为线性化而提前执行。

状态只有“检查中、已提交、拒绝、停止”四种设计含义，复用现有结果/停止类型表达。局部规则不适用可以回退；全局预算耗尽或取消必须结束工作。已提交且已完成输出准入的内容可按原合同保留，禁止在停止后再重建/发射一个所谓基线。已有整方法正确性守卫继续运行，直到共同记账覆盖其全部条件并有等效回归。

### D6. 让 Region 组合围绕归属和出口成立

复用 Region 与现有 owner/continuation 事实，收拢每个候选需要的摘要：拥有块集合、唯一入口、正常出口、带目的地的 break/continue、return/throw、按顺序绑定的异常边界、跨边界 SSA 值和效果要求。

父结构组合时检查入口覆盖、块不重叠、出口目的地匹配、异常优先级和 handler 归属、值定义可达。结构名字本身不足以决定是否允许嵌套。试探期间只提交候选拥有的变动；失败恢复 owner/visited/异常上下文，不复制或污染整轮无关状态。

S4 新增承诺域为：可约 CFG、闭合普通 named catch、唯一可证明入口与延续的循环/分支组合，以及这些结构内的 break/continue/return/throw。交叉保护范围、外部进入子区域、未决出口保持拒绝。现有 finally/TWR/monitor 能力作为不可退化基线；不在 S4 顺带承诺任意嵌套 finally/TWR/monitor。已有 finally 子结构只有能满足相同边界合同时才进入迁移域。handler 顺序和每个 throw site 的保护归属必须保留；跨区域值同时满足 S2 声明/边复制合同和 S3 效果顺序。已有 finally 的 return/throw 覆盖、TWR cause/suppressed、monitor 释放均作回归或拒绝边界，不由普通 catch 通过推导普遍支持。

固定 BCI/opcode/CFG/SSA 证书按接受域列迁移清单，逐项用相同机制替代；BCI 继续作坐标与来源。名称/常量/槽位/编译 lowering 变形须记录实际命中规则，不能把 100→101 假定为 BCI 改变。源码门扫描须区分生产准入、测试常量和合法平台名称，避免仅凭 grep 数量宣布泛化。

按方法构造并复用已经需要的边索引/归属信息，减少候选对整图反复扫描；复杂度与取消轮询采用实测，不能把 AnalysisSteps 称为 CPU 精确计费。

### D7. 类级转换共同提交声明与已覆盖使用

保留原始物理声明和方法恢复结果，在类级按候选构建有限修改计划。计划列出受影响声明、调用/创建点、隐藏成员、初始化步骤、依赖和映射。方法类型决定来自 S2，表达式效果核对来自 S3，结构边界来自 S4。

删除或隐藏成员需覆盖所声明范围内所有相关使用；存在外部未知调用者时保留可访问成员，或明确把输出限制为已声明的封闭范围，不能把本类没有引用当作全包没有引用。报告区分选定调用的行为覆盖与 ABI、反射、序列化、类名可观察性；本阶段不承诺未知动态使用闭合或生成包可原位替换。扫描停止/依赖缺失不等于使用数为零。匿名类有多个创建点时必须共同核对，任何一个捕获或构造顺序不满足则撤销整项匿名投影。泛型声明改变与调用点重载检查一起提交。

字段初始化提取按 `<clinit>`、构造器链及可观察操作顺序判断。预算停止或任一相关检查失败，保留未投影的有效类表示；如果原始成员自身无法表达则继续显式 fallback。物理查询仍能看到 synthetic 成员，源码映射可以解释其被哪个投影吸收。

事务单位是一个声明与其全部相关使用的闭合投影；有交叠修改或共享隐藏成员的候选先检查冲突。第二个相关创建点失败须撤销整项候选；另一个独立成员失败可以保留已经提交的投影，并报告真实 partial。全局停止后的收尾不读取新 Body、不重建整类基线。若仍使用文本编辑，计划绑定文本版本/摘要、唯一 span、重叠检查及偏移变换，不预设一个新的通用编辑器。

S5 交付统一的整类 source-unit map，复用现有方法 map 与 `MemberFamilyDerivedProjection` 等 derived spans：每段绑定本份 UTF-8 文本的 byte 半开区间；声明/属性使用 metadata origin，方法语句使用完整物理方法与 BCI，投影可保留多个物理来源。方法 map 仍只适用于自己的正文。合并、缩进、替换后重定位已提交段；未映射范围及原因显式记录，不因少量 derived spans 宣称整类来源完整。可选 map 后置物化，选择与否不改变正文；停止时不发布越界或指向另一份文本的映射。

### D8. 显式组合查询通过统一入口调度

组合请求由 typed 阶段选项表达：物理范围与谓词、可选声明/有界派发条件、可选恢复交付、runtime 环境、source target、预算。复用 Engine 的现有任务导向操作组织参数，不给 query crate 引入 java 依赖，不扩展为查询语言。

执行顺序为安全结构筛选、保留未知候选、所需解析、选定成员分析、请求的源码输出。纯结构或声明查询不构建 Java AST。有界派发始终是已知候选加开放世界状态；不把结果称为穷尽运行图。源码内容不能决定原始引用是否存在。

分页绑定完整阶段配置、谓词、snapshot/view、D11 定义的完整环境、源码目标、词法/投影语义选项、稳定顺序，以及相关游标/解析/恢复语义版本。源码目标仅在请求恢复时适用；budget 与 page size 仍是执行参数，不能因改变页大小而拒绝合法续页。每页可复用已有物理 facts cache，但恢复语义不依赖缓存存在。每个阶段独立报告结果和 coverage，整体 execution 汇总请求的阶段。全局停止后不启动后续 Body，已提交前缀可保留，未分析范围不等于不匹配。

验证时同一显式计划可用已有分立操作构造参照。两者在足够预算下比较语义结果、物理身份与映射，允许经过解释的读取/计费差异。相同执行路径重复运行仍遵守现有仅排除 elapsed 的确定性合同。

### D9. 验收按机制和完成层次计数

S0 冻结清单：输入 SHA、来源与授权、compiler/flags、引擎 SHA、完整 runtime/loader/provider 声明、有效源码配置、预算、oracle、适用域及排除原因。当前实现尚无独立 SourceTarget 时如实记录其别名关系和配置限制，不能在基线中伪造 S1 能力。默认复用已有 bcprov 类库 JAR 与 S2-009.war 的可获得原件，另加可再生 nested/MR/duplicate-origin 对照；原件缺失时阶段记未完成，替换必须公开修订清单并对新旧产品重跑同一输入，不能悄悄换分母。

每项记录物理方法、归属类、首个阻塞阶段、已观察到的其他阻塞、输出/编译/行为状态。诊断标签可以重叠，静态推测的根因单列；首个错误码不等于唯一根因，分类标签总数不得当互斥分母。

分母至少区分物理 class/method 记录、具有 Code 的方法、当前环境可绑定的方法、实际尝试恢复的方法；NoBody、shadowed/unbound、输入拒绝和执行停止分别记账。全包方法统计、选中类的完整编译、全包重建是不同证据；类抽样必须公开算法和样本量，不能外推全量类成功率。一次失败允许多个原因；已证实原先被遮蔽的后续阻塞不误记为新回归，尚不能归因的变化单列待查，也不能因第一条诊断消失就算恢复成功。

S2–S5 的机制矩阵必须包含参数化变形：类型/名称/局部槽变化、消费者数量、前后语句、结构嵌套、debug/no-debug、适用的 javac 8 与 javac 23 target 8；既有 ECJ/legacy 样本用于对应边界回归。按来源和语义族分组保留未用于开发判据的 holdout，避免同一源码的重命名版本泄漏到两侧。保留集一旦用于调准入规则即记录污染，并补充独立评估集；仅改名、常量或 flags 的同源样例不能分置为独立开发/保留集。每阶段至少覆盖两个独立问题族，并在 holdout 中出现实测恢复改善；输入名字、固定 BCI、fixture hash 不得成为生产恢复条件。

原 class 是受控输入的行为参照，JADX 是固定对照实现；JADX 的编译失败/行为分歧独立保存，不作为 Jarde 的正确预期。运行生成类时 classpath 隔离原始目标类，依赖可用范围单列，防止回落执行原 class。已知错误不能因为仍标 Unproven/NotPerformed 就计为成功。

验收分成三层：

1. 永久机制矩阵：承诺域内正例的完整生成类按原样重编、执行受控输入，返回、可见写入/调用序列、异常类型与清理顺序符合原 class；负例保持必要拒绝，不能靠扩大接受域通过。
2. 固定真实包：报告方法和完整类两种分母，分别统计含语句、可编译、已执行对照和停止。S2–S5 每阶段报告至少一个固定包中的净恢复收益或明确未通过；当前有效成功集合不得无解释退化。无 oracle 的方法只能计恢复/编译，不能计行为一致。
3. 资源与按需：单方法、重复交互查询、类级恢复、显式组合、批量工作负载各自测读取/Body/IR 次数、时间、RSS、预算与取消。S1/S6 必须验证排除分支不启动 Body/AST，S4 必须增加大图与深嵌套成本对照。

阶段性能阈值在 S0 用重复基线冻结，后续变动不得为了通过回填。先声明采样顺序、机器配置与阈值计算方法，再测每种负载 10 次交错重复及噪声范围，冻结用量上界和可接受时延/RSS 增量；本规划不虚构当前性能数值。新增候选的真实成本允许改变计数，但必须同步计费模型、解释精确预算门，不能直接更新 golden 数值掩盖额外全图扫描。

资源验收区分 cache 保留量、live handles、候选/AST/文本及序列化缓冲的峰值；慢 sink 的背压和并发窗口配置单列。取消 token 置位到请求返回、sink stop、预算/elapsed stop、worker panic 分别测量，不能互相替代。中断批次保留原始记录并标未完成，后续恢复按冻结协议复跑完整批次，不能混合不同配置补足十次。S0 只建立可观测基线；已知不合约失败必须记录，不能为完成基线而顺手修产品。

用户已确认将缺失端口补入 S0（任务 1.8–1.9）：只增加 reader/JVM/Java 的 test-support 观测。流读取在实际 `Read::read` 返回处记录调用/字节/EOF/错误，切片解析与 OS block I/O 单列。IR 在实际驱动阶段记录开始/完成，不能用 facade Body 次数替代。facts 以共享 payload 实例计数，Arc 克隆不增加 payload；AST 节点和明确列出的候选对象以构造、深 clone 和 drop 记录 live/peak，计数单位是对象而非堆字节。cache 权重、bulk 窗口权重、输出文本和编码缓冲继续独立记录，不把各维度相加冒充总内存。

观测使用固定大小原子状态，不持有 payload；只在 test-support 开启，normal build 无观测状态。每个正式样本独立进程；hot warmup 结束后静止时重置峰值至当前 live，累计工作读差值。生命周期与观测开/关对照先通过，再用新版本输出目录完整重跑 10 次交错协议。v2 470 次采集封存为端口补齐前的中间证据，不混入新批次；已触发 RSS 守卫的产品失败继续保留，不为补端口修复恢复逻辑。

fuzz 延伸到 recovery 与 class-source，输入始终有界，不在 fuzz 内编译或执行任意目标。检查完整性、终止、无 panic、origin、效果归属及预算；受控源码变形另行编译执行。必须记录实际进入 Java/投影阶段的有效输入数，不能只统计因格式损坏提前拒绝的运行数。

### D10. 复用与依赖选择

依赖基线沿用 [dependencies.md](../../dependencies.md)：reader 的 noak/rawzip/flate2、现有图算法和预算接口不变；本提案不增加生产依赖。Rust 标准容器和已有图/工作队列足以支撑本轮有限域。

JADX 参照固定为 `2fb1b16386941660fda07e9017285aec40fcb37f`：`Jadx.java` 的阶段划分、`TypeInferenceVisitor` 的 use/phi bounds、区域构造与后续收缩作为算法阅读入口。上游 LICENSE 为 Apache-2.0；移植具体代码或测试时保留来源、版本、许可证/NOTICE，并记录我们的语义和预算适配。仅从机制学习不等于整包复制，不能用 JVM/DEX 差别直接否定算法复用。

已有 ASC/common 候选评估保留在依赖文档；其接口适配、异常模型、MSRV 和依赖裁剪尚不足以替换当前层。未来若某个阶段出现明确缺能力，再以该阶段矩阵评测一个具体模块；不得为本提案预先升级整套工具链或引入通用图 IR。

### D11. 环境身份与产物绑定覆盖真正改变结论的输入

S1 先为审计 D1/D2 的字段遗漏建立最小反例，再修正合同；不预先把强推断记成已复现。直接复用现有环境声明和绑定结构，不增加全局身份服务。

- **环境身份：** 保存调用方 runtime、全部参与 domain 的 loader/parent/delegation、有序 roots/raw prefix/container origin、module/uncertainty、provider 的内容声明及提供的 snapshot 身份。provider 不额外决定搜索顺序。采用完整可比较声明作为基线，不因压缩成 ID 列表丢掉非调用方环境。
- **产物身份：** 方法正文与类单元分别绑定物理主体、实际环境、SourceTarget、词法/投影配置与确切文本；产生或解释派生源码的相关物理依赖保留来源。可选 evidence 的选择不改变核心产物身份，证据载荷范围单独报告。
- **版本域：** binding schema、外部 JSON 格式、cursor schema、解析语义与恢复规则语义分别定义变更条件。沿用 binding 的既有版本承载位置，把 registry/engine 的语义明确为能区分解析/恢复实现的标识；注册 major 上限与包版本不再充当它们的充分证据。规则语义变化必须改变相应标识/版本。
- **复现来源：** 引擎源码 revision 与二进制摘要进入运行收据；不要求每个内部类型都重复存 Git SHA。日志/文档变化不需要冒充语义变化；同包版本、同文本但证明语义变化的构建不得静默互认 evidence。

任何影响解析的父 domain 配置改变、依赖 snapshot 改变、源码语义配置或证明版本改变，旧 expected artifact 都须按适用维度拒绝；只有展示标签变化不应改变物理选择。相同 bytes 的物理缓存仍可复用，与环境相关的结论不可借用该缓存身份跳过校验。

备选是只增加 Git SHA 或把所有缓存键都塞入完整环境。前者不能修正同构建内的环境遗漏，后者会无谓失效纯物理事实；本方案按层绑定实际依赖。

### D12. S1 有限退出与恢复主线优先级

本次为 2026-10-26 的规划修订，替代旧文中“所有职责迁移与全路径收尾后才进入 S2”的执行口径；历史输入、失败、阈值、收据和已完成勾选不变。长期所有权目标保留，但纯归属整理从 S1 前置拆至 E7。只调整规划不会使任何门自动通过。

**S1 范围固定。** 以 CG 检查点的公开方法恢复、完整类恢复、其已有 proof/事实供给链及纯结构查询对照为入口边界；沿同次请求检查 raw 名称/descriptor、物理方法/loader/BCI/origin、必要证明、停止及提交。2.9 实施前登记这些入口实际可达的消费者、现有正反例和适用 S0 配置，形成有限清单，不再以 root 尚有 Java 判据或新迁移机会扩大清单。尚不能证明可达性或影响的风险标未决，不能当成无影响；新问题只有证明破坏该边界不变量时才阻塞 S1，并只修最小闭包。

| S1 退出项 | 必须通过的证据 | 明确不作为退出条件 |
| --- | --- | --- |
| 2.9 / A1.6 | 固定消费者逐项保留 raw 身份；supplementary、源码不可表达、损坏 MUTF 三类及跨 method/loader 来源不串用；现有有效输出保留 | 把每个纯拼写 helper 搬出 root；扩展所有旧编译器 lowering |
| 2.10 / A1.3 | 固定需求链的 read、IR、proof、emission/提交切点分别保留原 typed stop/首因/diagnostics；全局停止后零新 Body/IR、无半项提交，已交付合法前缀保留 | 未触达 bulk 协调器的 worker panic、慢 sink/并发窗口重构 |
| 2.11 / A1.1、A1.4–A1.6 | 现有 136×3 核心、完整类/来源和当前有效成功集；八项历史 native 失败逐项绑定原名/输入/基线，区分产品错、过时测试与后续能力缺口 | 把旧失败改成通过或静默删项；用 E7 的源码位置替代行为证据 |
| 2.12 / A0.3、A1.7 | 先冻结原 S0 配置逐行归属，再按原公式通过适用时间/RSS/实际 read/Body/IR/live/peak/取消门；每配置沿用 10 次交错协议及相同 cache/evidence 条件 | 要求所有无关 bulk/组合负载在 S1 一并完成；以计数变少、构建磁盘或同一失败基线充当通过 |
| 2.13 / A1.2、A1.5 | 当前输入/源码目标/环境/停止/版本在库、薄 CLI、示例及所属说明一致；移除本次路径中会分叉语义的临时实现 | 删除每个转发 wrapper、缩小全部隐藏 pub 或重写全局报告格式 |
| 2.14 / A1.7 | 2.9–2.13 全通过，A1.1–A1.6 的实际证据与有限范围对应；E1–E7 归属和 S2a 输入清楚 | 凭规划变更、局部全绿或历史 17/59 宣告 S1 完成 |

**资源门拆分不改历史结果。** 2.12 的归属清单覆盖原 S0 全部行：方法、类、重复结构查询及这些路径的依赖/呈现成本留 S1；只由 bulk 协调、worker panic、慢 sink/背压引出的行关联 E2 与 S6 / 7.6–7.7，并在原分母中保留失败/未验收状态。不得按运行结果好坏挑选范围；S1 若修改或复用该 bulk 路径作为自身证据，对应行立即成为适用阻塞门。原 S1 时间超门只要属于方法/类范围就必须修复并按原阈值重测；不得放宽阈值或仅要求“不比已失败基线更差”。S1 的有限验收不等于原全工作负载门或 E2 已通过。

**职责整理的独立归属。** E7 是本提案外的独立后续工作项，当前仅在下表登记，不创建新产品抽象，也不把整项完成加入本 change 的复选分母。每组需单独明确实际消费者、最小迁移边界及保持验收后再实施。

| E7 工作组 | 承接的剩余整理 | 独立验收与阻塞规则 |
| --- | --- | --- |
| E7.1 | owner census、自调用、call-site/bridge 的纯源码准入所有权 | 同 facts/决定/输出/计费/停止保持；缺陷与迁移分列。只影响源码位置的不阻塞 S1/S2a/S3a |
| E7.2 | nested/super/double-brace 专用策略与暂存政策归 Java | 保留原成员、全部已覆盖使用与事务边界；未知使用、错误隐藏或半提交仍阻塞实际受影响路径，不以迁移代替修复 |
| E7.3 | 薄适配、可见性及剩余持有期的纯整理 | 依真实调用收口，不以 root 零判据/零 wrapper/文件行数为目标；实际资源超门仍归 2.12 或所属阶段，不可移交豁免 |

**恢复主线分成可交付的两个早期里程碑。**

- **S2a / A2.5：** 3.1、3.2、3.3、3.6、3.7、3.9。复用现有 MethodIr/SSA 与声明规划，先形成值/槽/源码变量分组、全写入/使用类型约束、声明支配、definite assignment、普通边 phi 复制及复制环临时值；catch 输入按现有异常边事实保存 throw 前可见局部与 handler 绑定，不把 catch 参数当作 throw 前的值。复制仅在所属边执行，临界边不能把赋值放到共享前驱/合流点而影响兄弟路径。初始域为现有 Region 可表达的顺序、分支、可约循环和已支持 catch；不为完成变量规划重写 S4 区域算法。缺前驱、跨界使用及未证明的异常赋值继续拒绝。调用仍须满足最终声明的既有绑定/重载合同；需要新增泛型传播或推断的输入留 S2b，不靠 cast/擦除放行。
- **S3a / A3.5：** S2a 实际通过后，执行 4.1、4.2、4.5、4.6、4.8。先交付局部更新/赋值值多消费，以及普通字段/数组地址/RHS/调用参数两族的有类型顺序语句与临时值，保持左到右求值、次数、旧值、窄化与异常优先级；短路和三元效果仍在原条件内。糖化失败可保留本次已经完成准入的基线，全局停止后不能额外构建或发射。volatile/monitor/类初始化/未初始化对象只沿现有已证明路径处理，新增普通线性化能力不放宽其边界。
- **S2b / S3b：** 已声明泛型传播与调用扩展（3.4、3.5、3.8）、前后缀/赋值/左值糖化（4.3、4.4、4.7）分别承接后续；S2a/S3a 通过不勾选这些任务，不宣称整个 S2/S3 已完成。S4 仍待 S2/S3 全部验收，S5/S6 依赖不变。

A2.5 的两问题族是“分支合流/互斥生命周期”与“循环边复制/handler 可见值”；A3.5 沿用上述局部多消费与字段/数组/参数顺序两族。每个里程碑沿用 S0 已冻结来源/保留集，若不足则在实现前补齐并记录关联；须给出完整生成类原样重编、隔离受控行为、拒绝/停止、独立保留集改善及至少一个固定真实包净恢复收益，不能只比较 lowering plan 或诊断数。资源使用沿用适用原门；未满足则该里程碑不通过。

**固定验证入口与停止规则。** 复用既有 harness、输入/producer 绑定和证据格式：纯职责整理运行直接消费者及相邻停止回归，跨机制变更与里程碑退出运行冻结完整保留门；新差异或失败才追加有理由检查，成功命令不因摘要脚本修复反复执行。门集及覆盖关系必须在运行前声明，省去重复执行不等于省去已要求证据。凡正确性、身份或全局停止反例仍实际成立，禁止以移到 E7、已知失败标签或保守 quality 字段绕过。

### D13. 语法恢复问题先记录，架构变更后复核

2026-10-26 用户明确要求：潜在语法恢复问题先保留证据，等当前架构变更完成、恢复职责和共享接口稳定后再处理，避免在旧 helper/writer 上修完后又迁移或重写。本节只登记问题，不增加当前实现任务或架构退出前置，不改历史失败与通过状态。

| 编号 | 待复核问题 | 现有证据与确定程度 | 架构完成后的处理位置与验证 |
| --- | --- | --- | --- |
| SYN01 | 源码不可表达字段的声明别名与方法体引用可能不一致 | [R07物理子类报告](results/s1-synthetic-field-origin-v1/current/n1-unspellable-N1_Inner-8-all.json)中，字段声明为`this$bad_name`，构造器和`total()`仍使用`this.this$bad-name`。已观察到文本不一致；presentation明确不保证编译，本轮没有该物理类的独立编译记录，不外推其他变体。根类投影消除此捕获字段是另一条路径，不能替代物理子类验证 | 沿最终Java成员命名与引用生成入口重新复现，核对raw身份、声明及所有使用；完整生成类原样编译后再决定共同重命名或明确拒绝，不在旧字段发射点先补局部分支 |
| SYN02 | SSA值/复用槽到源码变量时，可能出现作用域、声明支配、未赋值使用或phi边复制不合法 | 属D12 / S2a已列的机制风险，本轮没有新增公开入口反例；不记成已确认缺陷，也不另建逐语法补丁清单 | 架构完成后在最终变量规划接口上核对分支合流、循环回边、handler可见值及完整类编译；S2a共享机制建设仍按原任务实施 |
| SYN03 | 更新、赋值值、多消费及字段/数组/参数组合可能出现求值顺序、次数、窄化或异常优先级偏差 | 属D12 / S3a已列的机制风险，本轮没有新增公开入口反例；不以当前局部绿例推断所有组合安全 | 架构完成后在最终有类型顺序表示和表达式候选接口上复现，比较完整类运行的值、可见效果与异常；S3a共享机制建设仍按原任务实施，具体语法修补与糖化不提前回填旧路径 |

后续遇到同类问题，只补最小输入位置、当前producer/源码目标、实际输出或原有失败记录、预期约束及确定程度；已有收据可复用，不为暂存记录另跑大矩阵。架构完成后先在最终实现上重新复现，已被共享机制自然消除的条目补证关闭，仍存在的再单独安排语法修复，不逐项搬迁旧补丁。

S1的raw身份/必要origin、typed-stop和适用资源门继续按D12执行，不混进语法暂存。R07的supplementary必要origin缺口已修，不重新列为语法待办；损坏MUTF子类Partial而N1根类仍可Complete的状态传播边界继续归2.10。暂存不等于验收豁免，不删除固定失败、不把未验证输出当编译或行为成功；当前架构任务只实现其原定共享机制和验收，不因本表扩大为语法修复专题。

## Risks / Trade-offs

- **类型约束扩大接受域** → 有界域显式列出，所有写入和使用共同检查；保留 boolean 字面量边界与缺依赖负例。
- **显式临时变量改变抛错或初始化时机** → 效果记账同时包含异常/控制归属，构造器和 monitor 特殊状态保持原有约束。
- **组合合同只换名而未减少特判** → S4 审查要求列出实际删除/替代的重复判据，holdout 通过共同入口；纯搬文件不算机制完成。
- **类族查询范围不足却删除可见成员** → 使用范围单独报告；开放世界保留可被外部调用的成员。
- **同一事实由不同 loader 解释** → 解析键绑定完整环境，引用身份包含 defining loader；原始内容缓存只共享不变事实。
- **基线或负例本身有错误** → 保留原 class、编译参数和实际执行结果；发现错误单独修订，不强迫新算法重现错误。
- **阶段范围扩张** → 每阶段只迁移表中机制域；旁支债务记录在阶段结果，不能以修复整个 71 单元作为收尾条件。
- **审计成本压过恢复收益** → 复用现有执行与证据工具，保存输入/命令/完整生成源码/摘要索引及必要原流；不增加重复的验收包装层或手工互相抄计数。

## Migration Plan

### 审计结论与设计承接

| 审计项 | 处理决定 | 阶段 / 验收 |
| --- | --- | --- |
| A1/A2/B3 | 归并证明与消费所有权，等价上下文一致，底层输入矛盾可拒绝；不默认新增 token/crate | S1 / A1.1、A1.6 |
| B1 | SourceTarget 只进入源码请求/产物，不污染纯 IR | S1 / A1.2 |
| 审计 D1/D2 | 完整环境身份、独立语义版本、错误 evidence 反例 | S1 / A1.4、A1.5；S6 / A6.2 |
| 审计 D3 | Header/Body 需求闭包、循环与去重、cache 分层 | S1 / A1.1、A1.3；S6 / A6.1、A6.3 |
| 审计 D4 | 类型、作用域、definite assignment、phi 边赋值共同验收 | S2 / A2.1、A2.4 |
| 审计 D5 | 效果/数值/异常时机与区域 handler 合同 | S3 / A3.1–A3.4；S4 / A4.1–A4.3 |
| B2/审计 D6 | 单候选事务、开放世界与可观察域、整类 map | S5 / A5.1–A5.4 |
| A3/C3/审计 D7 | 固定形状迁移、独立 holdout、真实分母、原 class oracle、恢复 fuzz | S0 / A0.1–A0.2；S2–S5 的收益门 |
| C7/审计 D8 | 实际资源与计费分开，取消/sink/预算/panic 各自验收 | S0 / A0.3；各阶段资源门 |
| C2 | 精确恢复事实保留 raw；合法名解码/别名、损坏拒绝 | S1 / A1.6 |
| B4/B5/C1/C4/C5/C6/C8 | 见 E1–E6 独立债务；相关版本/注释随所属实现最小同步 | 条件前置，不扩张全部阶段 |

### 阶段门与交付物

所有 ID 在 [tasks.md](tasks.md) 被引用；每一阶段先完成定义域和失败边界，再修改共同机制，最后记录实际验收。阶段验收不会自动关闭既有 change。

| 阶段 / 验收 ID | 必须交付的机制与恢复收益 | 退出标准 |
| --- | --- | --- |
| S0 / A0.1 | 冻结输入、分母、失败分布与历史任务承接 | 摘要和实际有效配置可重建，物理/Code/绑定/尝试/类样本分母分开，原流与证据等级保留，缺项明确 |
| S0 / A0.2 | 机制矩阵、独立 holdout 和 oracle | S2–S5 各两问题族与负例/组合轴，保留集无同源泄漏，完整生成类原样编译、隔离受控执行，JADX 分歧独立；无 oracle 不计行为一致 |
| S0 / A0.3 | 可复现资源与停止基线 | 预声明协议与阈值公式、各负载 10 次交错样本，token/sink/budget/panic 分开，缺测不通过；已测产品失败可冻结为失败基线，不能称产品合约通过 |
| S1 / A1.1 | 保持已验收源码职责与按需事实边界，剩余纯整理归 E7 | 方法/类入口在同上下文下语义决定一致；无候选时零新增依赖 Body；重复需求不重复解码 |
| S1 / A1.2–A1.3 | 输出配置独立、需求结果与停止可区分 | runtime/MR 不随 source target 改变；纯 IR 不受呈现配置影响；Missing/Ambiguous/Unknown/Stop 独立；循环不等于缺失；库/CLI 一致 |
| S1 / A1.4 | 完整环境身份 | 同 caller/content/loader IDs 下变更父 roots 顺序、prefix、delegation，身份按有效声明改变；旧 expected artifact 拒绝；同名不同 loader 不合并 |
| S1 / A1.5 | 语义版本与证据绑定 | 同包版本/同文本而解析或证明语义不同的构建不互认证据；只改 evidence 选择不改主体；格式/cursor/语义版本域明确 |
| S1 / A1.6 | 同次输入一致性与 raw 名称 | 错方法/facts/subject 的底层输入被拒绝或隔离为不可用于可信绑定；Engine 无新增注入面；supplementary、不可拼写、损坏名称分组验收 |
| S1 / A1.7 | D12 的有限退出与适用资源/债务映射 | 2.9–2.13 和对应 A1.1–A1.6 证据全通过；不以 root 策略清零或无关 bulk 修复扩张；旧失败不冒称已修 |
| S2a / A2.5 | SSA 到源码变量合法化先交付 | 3.1/3.2/3.3/3.6/3.7/3.9；两族、独立保留集及固定包净收益，完整类/行为/来源/停止与适用资源门通过；不等待 3.4–3.5 |
| S3a / A3.5 | 保持求值顺序的可用恢复基线 | 4.1/4.2/4.5/4.6/4.8；两族及保留集/固定包有实测收益；无糖化也保持效果和异常，不等待 4.3–4.4 |
| S2 / A2.1–A2.3 | 统一变量与使用点类型约束，覆盖引用合流及已声明泛型传递两族 | 类型矩阵与 holdout 完整类重编/行为一致；平台与用户容器走共同机制；缺依赖、冲突、重载负例有效；真实包净收益与退化清单通过 |
| S2 / A2.4 | 源码变量合法化 | 声明支配全部使用、到达边 definite assignment、循环 phi 同时复制、catch/捕获范围与槽复用成立；debug 变形不改变值流；不能填默认值过编译 |
| S3 / A3.1–A3.3 | 自增/赋值多消费与字段/数组调用参数两族的顺序基线和原子转换 | 原 class 的效果/异常轨迹一致；拒绝糖化仍可保留有效基线；消费者失败不丢效果；全局停止、恢复 fuzz 和预算检查通过 |
| S3 / A3.4 | 顺序与数值边界 | 已涉及操作的 null/越界/cast/调用抛错轨迹与窄化/溢出一致；浮点、装箱、类初始化及 monitor 未支持域保持拒绝；受控 oracle 含必要 cause/suppressed |
| S4 / A4.1–A4.3 | 循环/分支/普通 catch/非局部出口合同组合 | 域内组合与 holdout 通过；外来入口、重叠归属、错误 handler/出口拒绝；现有 finally/TWR/monitor 不退化；大图成本和取消满足 S0 门 |
| S5 / A5.1–A5.3 | 泛型声明/调用与 synthetic/类族投影共同提交 | 完整类原样重编、构造/初始化/异常轨迹一致；多创建点、外部可见成员、晚期拒绝/停止无半项改写；物理引用与映射保留 |
| S5 / A5.4 | 绑定整份类文本的来源映射 | 复用方法 map/derived spans，UTF-8 byte 区间和文本摘要匹配；声明有 metadata origin、投影保留多源；缺口显式；独立投影 partial 与同项半提交区分 |
| S6 / A6.1–A6.3 | 结构筛选、解析和源码交付显式组合 | 同分立执行的语义结果一致；纯查询零 AST；分页身份/前缀/停止正确；交互和批量工作负载通过资源门，原始引用不因投影消失 |

### 实施与回退

默认优先顺序 `S0 -> S1 有限退出 -> S2a -> S3a`，再完成 S2b/S3b，之后 `S4 -> S5 -> S6`。S0 已验收，S1 已完成 2.1–2.8，其余任务未完成；规划检查与局部实施通过本身不代表后续阶段已完成。S1 必须通过 D12 / A1.7（含 A1.1–A1.6 的有限闭包）才向后交付；S2a / A2.5 是 S3a 的前置，S4 仍等待完整 S2/S3，S5 同时消费 S2 类型/变量、S3 效果及 S4 区域合同；S6 消费 S1 的完整身份和 S5 的来源绑定。不得以迁移代码或删除固定数组替代这些验收。每阶段独立提交可审查的改动；阶段中先以现有通过集验证职责迁移，再扩展其机制域。内部临时参照路径只用于迁移验证，阶段交付前移除，避免维护两套判据。

若阶段出现正确性回归，回退该阶段尚未验收的实现，保留前一阶段及已记录反例；不得降低 fallback 守卫或改测试预期掩盖问题。无后向兼容要求，API 变更在对应阶段一次迁移库、CLI、示例与测试。

主架构文档的最小语义闭包、类型/投影边界和独立源码目标在 S1/S2 更新；类级与查询组合在 S5/S6 更新。只有对应实施验收后才同步其主规格。最后归档前重读已变化的主规格，避免本提案的 MODIFIED 段覆盖期间新增场景。

### S0 交付与失败承接

[最终报告](results/s0-observed-v3/README.md)与 [机器验收记录](results/s0-observed-v3/acceptance.json)冻结真实包分母、136 组机制配置及 47 配置各 10 次资源实验。470 次已收集，460 次返回，10 次 WAR bulk 因 RSS 守卫终止；终止前峰值只作下界，未给出完成耗时阈值。观测补齐实际 read/Body/IR 与声明对象的 live/peak，未修改恢复语义。

恢复差距为后续机制任务的输入：24 个真实类隔离抽样仅 2 个双编译器通过；受控矩阵 Jarde 36/136 双 runtime 一致，JADX 116/136。可选 SourceMap 改变类族决定归 S1 / 2.2，类级共同提交继续归 S5；panic 缺 Prepared 归独立债务 E2，资源/计费仍按独立债务 E1 的消费路径约束。未覆盖域、旧任务逐项承接和阈值公式见最终报告及其证据链接，未将这些失败标成产品合同通过。

### 与现有工作的关系

| 现有记录 | 本提案如何承接 |
| --- | --- |
| `recover-proved-local-source-types`、`prove-generic-field-write-source-types` | 已记录完成；S2 扩展共同约束，原回归作为基线 |
| `add-demand-driven-core-results` | 复用核心结果与可选证据分离，不重新实现按需 evidence |
| `assemble-proved-member-class-family` | S5 复用已交付家族组装，统一不同候选的提交条件 |
| `present-proved-java-structure` | 仍有未完任务；S0 逐项标明已覆盖、由本提案承接或独立保留，不能整项宣称取代 |
| `audit-corpus-gates` | 属 census/fingerprint 工具债务；复用有效工具和清单，S0 不顺手完成其全部任务 |
| post-test return leaves / prefixed catch retry 等旧 change | 产品与交付状态按 handoff 独立保留；新机制必须保留已验收行为 |
| goto 来源缺口、runner observed_counts、重复扫描债务 | 前两者单独记录；S4 只处理其实际替代路径上的成本债务，不推断来源缺口同根因 |

### 独立债务与条件前置

下表记录修复归属和依赖门，不把独立产品债务塞入本 change。E7 的非阻塞职责清理从 S1 拆出，独立范围与验收见 D12；不计作本 change 的实现任务，也不是 S2a/S3a 前置。S0 记录当前失败，不要求先修产品才能冻结失败基线。后续阶段若依赖该路径，关联独立修复并等待其证据；不能把已知违规当成可接受预算或删掉不利样本。

| ID / 审计项 | 独立处理范围 | 何时阻塞本提案 | 所需退出证据 |
| --- | --- | --- | --- |
| E1 / B4、C7 | X1/读取/输出计费、D05 坐标与成本模型 | 某阶段使用该 usage 或坐标作为验收依据时 | 维度单位、实际工作、坐标所属字节流可核对；阶段自身新增路径必须正确计费 |
| E2 / B5 | bulk 关闭、Prepared 协议及其慢 sink/背压/峰值保留的独立收尾 | S6 批量故障/停止交付验收；更早阶段若改 bulk 同样适用 | 非 Refused 正常结束有已交付 Prepared；准备失败 Refused 合法直达结束；sink 拒绝不强补事件，panic/cancel/sink/预算分别验证 |
| E3 / C1 | 所有公共报告的格式版本策略与旧文档 schema 漂移 | 对应 JSON/cursor/binding 的变更阶段 | S1/S6 先完成自己消费的版本门；更广泛 envelope 重设计单独评估，不强迫所有版本常量相同 |
| E4 / C4、C5 | noak 升级/替换与缺少消费者的扩展点 | 具体 parser 反例或所需能力确实挡住某阶段 | 确认反例、现有适配/oracle 回归；没有必要不引第二 parser 或删除已有库能力 |
| E5 / C6 | 大文件/测试机械拆分 | 本身不阻塞机制验收 | 仅确有职责/导航收益时独立提交；不以行数或减少文件大小算恢复收益 |
| E6 / C8、历史账本 | 无关文档、runner observed_counts、旧 goto 来源缺口 | 阶段使用对应收据或映射来证明新结论时 | 使用可复核原流和正确身份，明确受影响范围；新文档/新映射本身不能沿用错误，旧债务不冒称已修 |

| E7 / 剩余源码职责 | D12 的 owner census/call-site/bridge、专用类族政策与薄适配/可见性纯整理 | 纯迁移不阻塞 S1/S2a/S3a；实际破坏身份、停止、正确性或适用资源门时只阻塞对应最小修复 | 分组独立验证同事实/输出/来源/停止，不能用减少 root 行数声明恢复收益或已修产品缺陷 |

## Open Questions

- S0 具体冻结哪些可获得真实包与编译器二进制摘要，由已有授权语料的实际可用性决定；必须在任何机制实施前落盘，缺失不得标验收完成。
- 各阶段私有记录的具体字段与 S4 owner/continuation 的合并位置可在理解周边实现后确定；D12 已固定 S1 退出、S2a/S3a 域和依赖，不属于待定问题。
- S2/S4 新需求的实际内存与扫描成本需由 S0/S1 数据确定，现阶段没有可据以承诺的加速比例。


## S1 2.1–2.2 implementation receipt（2026-10-20）

本节记录已实施边界，不改后续阶段的验收门：

- Java 的 `source_proof`、`generic_source_proof` 持有源码语义与拼写判据；root 提供原环境选定 Header、物理事实和 staged/raw contract，保留事务、回滚与停止编排。没有新增 crate、Session、跨请求事实缓存或另一套接受规则。
- `ClassSourceRecovery` / `ClassSourceCandidates` 作为 owning 结果传递至 root 的实际类消费者。必要 `ProjectionProof` 在同次 emission 按候选保留 origin 与所需 allocation；可选 SourceMap/RuleDetails/RegionDetails 独立发布。普通无候选恢复不因此保留完整 AST。候选名称过滤仅决定保留范围，不作为类型关系证明。
- 2.2 已消费冻结 E2 反例：三种 evidence 的正文、决定、完整类编译与受控行为一致；有/无 projected emission 以及真实缺 origin 的拒绝均验证。all 的既有有效产物和公开来源保留。
- [逐项证据](results/s1-core-v1/README.md)包括 983 项通过、136×3 evidence 比较、6 编译/12 JVM 对照和真实包既有成功类重编；[资源收据](results/s1-core-v1/resources.md)明确同期 2/24、冻结 S0 5/24 时间门失败，RSS 均通过。新增同次证明计费与候选保留成本已观测，不能据此宣布 2.10 通过。
- 2.3–2.8 的后继实施与验收见下文；raw 名称和全阶段门仍由 2.9–2.10 承接。S2 不进入，独立债务 E1–E6 不并入本次实现。

## S1 2.3 implementation receipt（2026-10-20）

PreparedClass 请求内仅保留已需求且完整的 Body Arc，JVM driver/MethodIr/callee 共享该事实；普通 owning-read 也以同次 MethodIr 交接。原始 execution 是解码来源，当前费用独立报告。四类 Java 证明共用同方法环境的成功 Header，缺失/未完成不负缓存。单类任务顺序拥有，释放后不全局保留。

[验收](results/s1-demand-v1/README.md)：334 项通过，136×3 文本/决定零差异，136 个完整公开报告排除 usage/elapsed 后相等，6 编译/12 受控 JVM 对照、真实成功两类 4 重编通过。R1 旧字段停止测试失效和 R2 仅测试输入修正原流保留。首个 Header 需求的 resolve+confirm 物理读取未合并，2.4 继续统一状态/循环/停止，2.10 仍承接原时间超门。S2 未进入。

## S1 2.4 implementation receipt（2026-10-20）

同次 Header 需求以 DependencyResult 区分 Found/Missing/Ambiguous/Unknown，外层保留原 Error/Report Stop，只有 Found 进入门面正缓存。平台只消费有效搜索顺序的 clean Missing；完整目录确认缺失可能零 Header 读取。同步读取不另建 InProgress registry，环沿已有 ancestor/visited 与 Cyclic gap 保留未决边界。

全局停止由拥有流程的门面传播：HeaderClosure 两条 memo 入口先 poll，已停止 driver 不再准备辅助 Body，类族阶段和 pool-rerun 多候选循环在合并 Stop 后返回可靠前缀。局部损坏/普通拒绝仍可保留独立结果。

[验收](results/s1-demand-state-v1/README.md)：1140 passed / 0 failed / 0 ignored；136×3 文本/决定零差异，136 个完整公开报告除 usage/elapsed 相等，6 编译/12 隔离 JVM、真实两类 4 重编通过。R3 产品、R5 根库/Java 与 R6 集成测试后继分开绑定；原失败均保留。S1 2.5–2.10 尚未验收，2.10 原时间超门未调整，S2 未进入。

以下为 2.4 时点的历史待验证输入，已在后续 2.6 复现并修复（见下方收据）：同名不同 loader 门还需核对 Java Header 供给的定义 loader 传递及 cache key：当前 trait 按 raw owner 请求，后继父链的解析起点不能只凭最初 caller 推定。此为后续需构造的静态风险，不在此宣称已复现或修复；已有 JVM HeaderClosure 的 loader 规则不能替代 Java 消费路径的实际验收。

## S1 2.5 implementation receipt（2026-10-20）

`SourceTarget` 是 Java 层的闭合配置，只注册 7/8，构造与反序列化均拒绝未注册值。恢复请求/报告分别携带 source_target 与 runtime_profile；Engine、类源码、类族重恢复、批量与薄 CLI 显式传递源码目标。纯解析/IR 请求不增加此字段，平台引用拓宽继续使用运行时 release。`RecoveryProfile = RuntimeProfile` 别名移除；产物 binding schema 升为 2，源码目标进入 Configuration，运行时仍属 Environment。

目标 7 保留 foreach、try-with-resources 等已有合法路径；lambda/method reference、接口 default/static/super 调用和类型注解源码在缺少已证明替代时明确拒绝或降级，保留原始物理事实。MR 验收先用已有容器内选择 API 选出 MR11，再显式声明对应 loader roots；不把容器内 MR 选择虚构成自动跨 root 加载策略。固定 runtime 下目标 7/8 的选中物理方法、共同符号及环境身份相同。

[验收](results/s1-source-target-v1/README.md)：28 个目标 1145 passed / 0 failed / 4 既有 ignored；全工作区 all-targets/all-features check 通过。136×3 正文无退化；旧字段比较器的 64 个嵌套路径差异原流保留，完整公开报告仅应用明确的配置/schema 迁移后 136/136 等价。源码 8 的 6 编译/12 JVM 对照、真实两类 4 次重编通过；源码 7 使用 JDK 8 source/target 7 编译并在 JVM 8/23 对照通过，JDK 23 不支持 release 7 的工具链限制单独记录。R1 产品与 R2 五文件纯测试修正分开绑定，两个产品 CLI 逐字节相同。后继环境身份与语义版本验收见下方；底层输入后继验收见 2.8；raw 名称及原资源门仍由 2.9–2.10 承接，S2 未进入。

## S1 2.6 implementation receipt（2026-10-20）

2026-10-24 [当前构建复核 v25](results/s1-environment-recheck-v25/README.md)：旧 2+7 失败实际复现，当前 A1.4 的 11 项包含在 288 passed / 0 failed / 1 ignored 中。2729 当前来源/构建前清单/归档及同次 V R1 七二进制匹配，5895 冻结 payload 核验；有效恢复引用同源码/CLI 已通过原流，本轮未重跑 CLI/JDK、未新建 Cargo 或修改产品/测试体。2.6 保持完成，2.9–2.10/S1 未完成，S2 未进入。

2026-10-24 [当前源码复核 v24](results/s1-environment-recheck-v24/README.md)：实际重现旧 2+7 失败，当前 A1.4 全 11 项及相邻 288 passed / 0 failed / 1 ignored。2726 来源/归档匹配，七个 R1 二进制对应当前相同生产源码；R2 仅三项外置测试/fixture 差异，不冒称全产品重建。136×3、6 编译/12 受控 JVM、真实两类 4 重编本轮实际重跑保持；无新增 2.6 产品或测试体修改，target 缺席。既有 2.9 专项失败及修正后的待验收状态保留，本次止于 2.6，S1 总门未通过，S2 未进入。

`EnvironmentIdentity` 直接保存完整有序 `domains: Vec<LoadDomain>`、`providers: Vec<HeaderProvider>` 及提供的 snapshot ID；移除简化的 domain_loaders/provider-ID 列表。非 caller 的 parent/delegation/roots/raw prefix/module/uncertainty、provider 内容声明都可比较，provider 仍不增加搜索位置。binding schema 升至 3，只说明 wire 合同变化，不替代 2.7 语义版本。

复用现有 resolver，Header 供给保留选中的 defining loader 与物理 definition；后继需求通过已有 domain 声明从该 loader 发起。请求内 Java Header memo 与层级需求 memo 按 `(initiating loader, raw name)` 区分，当前 IR 快捷路径限定 caller；节点身份比较 `(defining loader, physical definition)`，层级图使用请求内节点索引。类型关系抵达同名目标时，跨 loader 必须与 caller 的实际目标绑定一致；桥接返回、直接父类和重载父链消费同一上下文。没有新增 Session、跨请求语义 cache 或多 loader 整类入口。

[验收](results/s1-environment-v1/README.md)：修复前真实复现有效关系误拒绝、同名类型错误接受及旧 expected artifact 被判 Agreed。A1.4 的 11 项新增测试通过；34 目标 1497 passed / 2 既有 failed / 4 既有 ignored，新增退化为零。两项失败在 2.5 源码重建后同样出现：EM11 cap2 的旧成本断言失效、预取消 skipped 范围遗漏；原失败保留并由 2.10 承接，没有改断言掩盖取消合同缺口。136×3 正文/核心决定不变，完整报告仅应用明确的环境/schema 迁移后 136/136 等价；6 编译/12 隔离 JVM、真实两类 4 重编通过，真实包未执行。R1 产品 + R2 两文件 cfg(test)/测试后继分别绑定，全工作区 all-targets/all-features 检查通过。

[对抗复核](results/s1-environment-v1/adversarial-review.md)将非 caller uncertainty 的平台契约消费列为 3.3 的静态待复现输入；完整身份保留声明不代表所有语义消费者已完成复核。2.7 后继验收见下方；后继 2.8 见下方；2.9–2.10 与原资源门仍未通过，S2 未进入。

2026-10-21 [当前构建 A1.4 复核 / v6](results/s1-environment-recheck-v6/README.md)：旧构建 2 个 loader 与 7 个身份失败再次复现；当前全部 11 项及相邻回归 286 passed / 0 failed / 1 ignored。当前 532 项源码与 raw-class-names R4 构建逐项相同，target 已清理，无新增产品修改。恢复保留引用同构建既有 136×3、6 编译/12 受控 JVM、真实两类 4 重编并验证证据 pins，本轮未重跑 oracle。已有 2.9 overlay、2.10 和 S1 整体仍未验收，S2 未进入。

2026-10-21 [当前工作区 A1.4 复核 / v7](results/s1-environment-recheck-v7/README.md)：旧构建 2 个 loader / 7 个身份失败实际重现；当前 A1.4 全 11 项与相邻回归 286 passed / 0 failed / 1 ignored。本轮实际重跑 136×3 正文/核心决定、6 编译/12 受控 JVM、真实两类 4 重编，全部保持。当前 534 项源码、归档及 42 个二进制匹配 source-name-boundary R1，构建 target 已删除；A1.4 六段源码与 v6 对应机制相同，无新增 2.6 产品修复。此前名称边界 overlay 保留但不授予 2.9 验收；2.10 和 S1 阶段门保持未通过，S2 未进入。

此前 2.6 的后继 [v8 当前工作区复核](results/s1-environment-recheck-v8/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项及相邻回归 286 passed / 0 failed / 1 默认 ignored。545 项源码/资产匹配 raw-member R4；核验同构建冻结的 8,382 个证据 payload，恢复保留引用其中既有 oracle，本轮没有重跑 Cargo/恢复矩阵/JDK。只读复核未见新增 2.6 缺口，无产品改动；仅路径变化测试可表达“展示标签”边界，API 无独立标签字段。2.9–2.10 和 S1 整体未验收，S2 未进入。

本次指定 2.6 的 [v9 复核与测试补齐](results/s1-environment-recheck-v9/README.md)重跑旧二进制 2+7 失败；当前 A1.4 的 11 项及相邻回归 286 passed / 0 failed / 1 默认 ignored。补齐 bridge admission 缓存的跨 loader、准备 facts 快捷路径、零 Header 热命中和取消热命中定向断言，只改 cfg(test)，不改恢复语义。555 项源码/资产与本轮两条成功构建、六个二进制匹配，target 清理；中断重建收据保留。既有恢复 oracle 仅引用生产源码相同的 raw-casts R3 并核验 5737 个 payload，本轮未重跑矩阵/JDK。2.9–2.10 与 S1 整体不额外验收，S2 未进入。

本次指定 2.6 的 [v10 当前源码复核](results/s1-environment-recheck-v10/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 的 11 项及相邻回归合计 286 passed / 0 failed / 1 默认 ignored。577 项源码/资产重新构建六个目标，两条命令成功，target 已清理。生产源码与 raw-projections R2 逐文件相同，仅待验收匿名类测试/资产不同；本轮无产品或测试体修改。核验其全部 6786 个冻结 payload，恢复保留引用既有 oracle，本轮未重跑恢复矩阵/JDK。止于 2.6，不继续 2.9，不授予 S1 整体验收，S2 未进入。

本次指定 2.6 的 [v11 当前来源复核](results/s1-environment-recheck-v11/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全 11 项与相邻回归 286 passed / 0 failed / 1 默认 ignored。复用进入本轮时已启动的 R3 构建，589 项源码/资产与 56 二进制匹配，七构建命令成功且 target 清理；本轮无生产代码或测试体修改。实际重跑 136×3 正文/核心、6 编译/12 受控 JVM、真实成功两类 4 重编，全部保持，未执行真实目标。此前已启动的 2.9 回归在两项新增 enum-switch 测试失败后终止，原流保留，不宣称完整回归或 2.9 验收；相关事项仍归 2.9/2.10。本次止于 2.6，S1 整体未验收，S2 未进入。

本次指定 2.6 的 [v12 当前源码验收复核](results/s1-environment-recheck-v12/README.md)重新运行旧构建，实际复现 2 个 loader / 7 个身份失败；当前 601 项源码/资产重新构建六个目标，两条命令成功，A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 既有 ignored**。本轮未改生产代码或测试体；与 enum-switch R4 差异仅此前待验收的 Outer.super 两测试/12资产，未继续 2.9。核验 R4 全部 9537 个冻结 payload，恢复保留引用其既有 oracle，本轮未重跑矩阵/JDK。target 已清理，2.6 保持通过；2.9–2.10/S1 未验收，S2 未进入。

本次指定 2.6 的 [v13 当前构建复核](results/s1-environment-recheck-v13/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 既有 ignored**。当前601项源码/资产、源码包及56二进制匹配Outer.super R3，核验12121个冻结payload，target当前不存在；本轮复用该固定构建，无新增Cargo、生产代码或测试体修改。恢复保留引用同源码/二进制既有oracle，本轮未重跑矩阵/JDK；未继续2.9，2.9–2.10/S1仍未验收，S2未进入。

本次指定 2.6 的 [v16 当前构建复核](results/s1-environment-recheck-v16/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全部 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。616 项当前源码/资产、归档及 57 二进制匹配 metadata-refusal R1，5079 个冻结载荷逐项校验，target 不存在；无新增 Cargo、产品代码或测试体修改。恢复保留引用同源码/二进制的既有 oracle，本轮未重跑矩阵/JDK。2.6 保持通过；本次止于 2.6，2.9–2.10/S1 未验收，S2 未进入。

## S1 2.7 implementation receipt（2026-10-20）

方法 binding schema 4 分开恢复合同（Java/facade 实现 ID）与解析合同（reader/query/JVM 实现 ID）；包版本、major 上限、规则表、外部格式与 cursor 各保留自身含义。源码 ID 是保守来源摘要，不是语义等价或二进制摘要；实际源码包/编译命令/二进制 SHA-256 由收据独立固定，test-support 开关与绝对构建路径不进入有效身份。

词法/投影配置显式保留 pool 拼写、raw projection 名称与数组构造器方法引用开关；固定顺序的 facts/member/proof handoff 流式摘要参与绑定。物理依赖保留实际 defining loader 与完整 definition，包含被拒候选已经读取的事实，复用原解析/分析报告而不重读。当前方法正文先提交，旧 expected 只门控证据附着，mismatch 保留当前有效结果。

[验收](results/s1-semantics-v1/README.md)：实际语义变体的 proof witness false → true，包版本/schema/规则/方法/环境/配置/依赖/正文相同，binding 仅 Java ID 改变，拒绝旧 evidence。独立目录 normal/test-support 的完整 binding、逻辑预算和除计时/expected 声明外的报告相同，五个 ID 与 R3 CLI 一致。34 目标 1501 passed / 2 既有 failed / 4 既有 ignored，新增退化零；136×3 及完整报告、6 编译/12 JVM、真实两类 4 重编通过。原 debug-name 单元断言迁移原流和 R3 重建来源均保留。

2.8 的输入一致性后继验收见下方，输入摘要仍不能证明真实性；2.9 raw 名称、2.10 两项既有失败和资源门未完成。整类 binding/map 与组合 cursor 语义由 S5/S6 承接，未进入 S2。

## S1 2.8 实施与局部验收（2026-10-20）

[输入来源验收](results/s1-trust-v1/README.md)将 public Java handoff 明确命名为 `*Input`，所有公开底层恢复入口返回 caller-supplied/unbound 预览，subject 标签、metadata 构造器和 publication callback 均不授予来源。Engine 私有适配器复用 immutable analysis/已有读取，核对 payload、声明 facts、物理方法/成员 owner、完整环境和 runtime，在准确正文提交边界构造 binding 并门控可选 evidence。proof 早停仍标明 selected reads；必要阶段失败不附着暂存 binding，可选证据停止保留正文。不新增 token、Session、crate 或每条 proof 的重读。

真实消费位置的不存在构造器反例、错 facts/subject/runtime，原先 Produced 且 bound，现在预览正文与来源保留而不能获得可信 binding。五种公开入口包括总返回允许的 callback 仍不能验证 expected；Engine 能验证自己的产物。41 目标 1532 passed / 2 既有 failed / 4 既有 ignored；136×3、完整报告、6 编译/12 JVM 和真实两类 4 重编均通过，新增退化零。方法 binding schema 保持 4，恢复合同 revision 升为 2，解析合同保持 1。

2.8 仅完成 A1.6 的输入来源部分；raw 名称与解码仍由 2.9 承接。2.10 的已知失败/资源门未通过，S1 整体尚未验收，S2 未进入。

## S1 2.9 进展：reader 与名称/ZIP 边界（2026-10-21，尚未整项验收）

[进展记录](results/s1-raw-names-v1/README.md)先冻结 21 个受控名称输入与 34 次原始 JVM oracle，分别记录 supplementary 成员/descriptor、类名选择、不可拼写 debug 名称与损坏 MUTF-8。reader 的容错成员表改用严格编码校验，合法非 scalar UTF-16 仍无损保留，183 单元与 2 边界测试通过。名称查询/CLI 文本转 MUTF-8；ZIP 只对 JVM 名称作无损 UTF-8 转码，raw prefix、entry 与物理身份保持原样。实际修复错误选中 MUTF-8 路径诱饵及将不可表示名称报告为 Missing；serde 注入的查询在扫描前明确拒绝。

路径修正 R1 的 12 目标 659 passed / 1 既有取消 failed；R2 的 20 导航与 4 边界测试通过，实际 CLI 的两组 supplementary class/member 可定位。R1/R2 的来源与测试范围分开绑定，不将重复结果累加，也不将物理定位修复当作完整源码恢复收益。Java facts、精确匹配、callee handoff、raw alias evidence、源码类型拼写与全矩阵回归尚未完成；2.9/完整 A1.6 保持未验收，进度 17/59，S2 未进入。


## S1 2.9 进展：raw 声明事实交接（2026-10-21，尚未整项验收）

[检查点](results/s1-raw-declarations-v1/README.md)复现合法孤立 surrogate 方法名被 lossy 同次校验误认，改为 MethodFacts/DeclaringClass 保存 raw 名称与 descriptor，Unicode 构造经 canonical MUTF-8；私有校验比较 raw declaration，参数/返回及 generic constructor 使用 raw descriptor，对象类型在源码边界严格解码。原文本 getter 只提供派生展示视图；其他精确消费者仍待迁移，不将此视图提升为身份。

raw facts 改变语义输入的固定序列化，按既有合同升 binding schema 至 5、摘要域至 v2，恢复合同至 3，解析合同仍为 2；外部报告/cursor 版本独立。R2 的 12 目标 875 passed / 2 个既有源码断言 failed，测试及其读取文件与改动前相同；2.10 承接，未改断言换绿。136×3 正文/核心决定、6 编译/12 受控 JVM、真实两类 4 重编无退化。本轮不声称完整公开报告逐字段迁移已验证。

21 个冻结名称样本保留原 5 项可编译/行为成功，4 损坏输入仍 Failed/classfile_decode，12 合法输入仍不能完整类编译。后继继续 raw member/callee/call/field/dynamic 引用和精确消费者、raw alias evidence、完整类类型/声明拼写；make/class/field owner 与嵌套类等消费者未全部迁移。2.9/完整 A1.6/S1 阶段门保持未完成，S2 未进入。

## S1 2.9 进展：raw 成员交接与调用签名消费（2026-10-21，整项未验收）

[成员交接](results/s1-raw-members-v1/README.md)用实际 callee 反例修正 owner 的 lossy 合并，保留 callee report 的 raw class 与 Java CallTarget/MemberBody/ClassMembers 的原始身份，accessor 与 MethodHandle 沿 raw 交接。[调用签名检查点](results/s1-raw-signatures-v1/README.md)进一步复现“raw 已保存但类型仍从诊断文本重建”的缺口：孤立 surrogate 被转义后伪造为 `p.\\uD801`。生产判断改用已有字节解析器，调用/构造参数、返回与原始参数切片共用 descriptor 事实；类级参数检查复用同一类型拼写，删除重复解释，没有增加样例专用判据。

最新 schema 6、恢复合同 5、解析合同 3、输入摘要域 v3，各版本域独立。当前 R1 的 14 目标 1144 passed / 3 项本轮基线已有 failed / 1 ignored；两项源码字符串断言及 prepared/direct 来源字段比较继续计失败，2.10 按入口来源合同承接。136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编通过；本轮不声称完整公开报告迁移已验证。21 名称中原 5 项编译/行为成功保持，4 损坏拒绝，12 合法输入仍不能整类编译。

Field/DynamicSite/SAM MethodType、剩余 owner/name 精确消费者、raw alias evidence 与完整类拼写继续按 2.9 推进；2.9/完整 A1.6、2.10 和 S1 阶段门均未完成，S2 未进入。源码包、反例、构建、测试与清理收据分开绑定，不能用本检查点替代未覆盖路径验收。

## S1 当前检查点：A1.4 保持与 raw Field 消费（2026-10-21）

[Field 检查点](results/s1-raw-fields-v1/README.md)承接已完成的 DynamicSite/SAM raw 消费，Field 三元组从 CP 到 Operations、字段/accessor 判断、capture/initializer/enum sidecar 和类级物理校验保持原始字节。SSA receiver 只在 raw 上剥 descriptor 外壳；字段声明拼写先无损解码 MUTF-8，诊断与可选 raw evidence 均在发布边界派生。未新增事实层、Session、cache 或按样例放行。字段输入编码改变，schema 为 7、恢复合同为 7、摘要域为 v4；解析合同保持 3，历史版本仅适用于各自检查点。

三轴 raw 身份反例实际由失败转绿，四个真实编译字段 JAR 的完整生成类由 0/4 变为 4/4 编译并通过 8 次受控 JVM。21 名称样本由上个检查点 5 项编译增为 9 项，18 次行为一致；4 损坏编码仍拒绝，8 合法输入仍不能整类编译。136×3 正文/核心、6 编译/12 受控 JVM 和真实两类 4 次重编保持；本轮不声称完整公开报告迁移通过，真实包未执行。

当前 31 目标 1429 passed / 8 failed / 3 默认 ignored，原 14 目标的三项失败不变；扩大五项失败在迁移前 532 项源码的独立构建中同样出现，不将源码相等当作复现。成员类投影/outer-super 断言须核对真实合同，WA main 的整类失败仍是恢复缺口，2.9/2.10 依根因承接。A1.4 环境身份八项与根库 loader/cache 三项在当前构建全部通过，相关实现与迁移前一致。2.6 无新增确认缺口，完整 A1.6、2.10 与 S1 阶段门保持未通过，S2 未进入。


## S1 2.9 进展：类名与嵌套关系（2026-10-21，整项未验收）

[检查点](results/s1-raw-class-names-v1/README.md)用严格 MUTF-8 拼写 class/simple/package 和父类型，失败沿既有 Option/Result 传播。InnerClasses 的源码简单名与物理字节分开，候选、子类自述、组装/caller relation 使用 raw/canonical MUTF-8；排序取真实 child 声明，static fold 的字段/方法 descriptor 和数组类引用直接匹配 raw bytes。没有新增事实层或按输入放行，package-info 原投影保留。恢复合同升至 8；schema 7、解析合同 3、摘要域 v4 不变。

21 名称由 9 编译增为 13 编译/26 次行为通过，4 损坏仍拒绝。四个嵌套子类的独立查询由 0/4 变为 4/4 编译；四个根类投影仍失败，同源 ASCII 控制也失败。后续类族来源证明不能靠放宽 cast/BCI 准入替代，单独子类编译不等于根类组装或反射/ABI 等价。31 目标 1431 passed / 8 基线失败 / 3 默认 ignored，失败集保持；A1.4 11 项、136×3 正文/核心、既有完整类/受控行为及真实成功两类重编通过。所有当前 532 项源码/归档/二进制绑定 R4，五个本阶段 target 清理。raw alias evidence、其余精确消费者与 S1 停止/资源门仍未完成，2.9–2.10 未验收，S2 未进入。

## S1 2.9 检查点：raw 局部名与别名证据（2026-10-21）

[实施/验收](results/s1-raw-alias-v1/README.md)让同次 reader 的 raw 局部/参数名贯穿 DebugLocal、SlotEvidence、reuse 和 RenderedName；精确同名判据按字节，NameTable 负责最后的 Unicode/alias/collision 源码拼写。MethodParameters 的槽/flags/descriptor 证明与名字能否原样拼写分开，正文/声明复用最终名字。NameDetails 复用 RenderedName 保留 raw 与变量身份，按请求发布，bulk 保有量计入其真实分配容量。

旧 surrogate 名称证据/inputs_digest 碰撞已实际复现并转绿；完整物理 binding 原本仍不同。6 组 MP 完整类 18 编译/36 受控 JVM，21 名称 13 编译/26 JVM 与 136×3 及字段/真实成功类保持。40 目标 1697 passed / 8 基线 failed / 8 默认 ignored；R3 产品、R4 仅一测试文件后继分别绑定。schema 8、恢复合同 9、输入域 v5、解析合同 3。仍有其余精确消费者与名称语法，2.9–2.10 不勾选，S1 整体未验收，S2 未进入。

## S1 2.9 当前检查点：共享源码名称边界（2026-10-21）

[实施与证据](results/s1-source-name-boundary-v1/README.md)收拢 raw 内部类名到严格 MUTF-8/Java 8 Character 拼写门；静态区间由冻结 JDK 8 生成并逐项核对，无产品 JVM 或新依赖。类/包/descriptor/引用 owner 共用门，保留 `$` 字面身份、7/8 的 `_` 类型名、原数组 parser 及裸 `Lfoo`；局部别名仍与外部类型身份分开。producer census 按 raw descriptor，Frame 与 catch 不经 lossy 形成类型，未放宽投影/类型关系/BCI 证明。

17 名称原件在 JVM 8/23 加载，旧产品 7 组非法/身份不保真输出、2 组合法包名被拒；当前 8 合法组双编译器与二进制名保持、9 组明确拒绝。受控 supplementary catch 从 0/2 到 2/2 编译、4 JVM 对照通过；不可表达字段 owner 从错误 Structured 变为 Mixed/Fallback、raw 字段与具体原因保留。旧 U+FFFD 类型断言按实际编译器反例迁移，原失败与生产不变的消费探针基线保留，不删除 raw 身份负例。

当前 41 目标 1706 passed / 8 基线 failed / 8 默认 ignored，A1.4 全 11 项保持；136×3 正文/核心、既有名称/字段/alias/参数名及真实两类重编保持。R2 的 534 源码/归档/42 二进制匹配，target 清理；schema 8、恢复合同 10、解析合同 3、输入域 v5。class literal/array 的 raw 消费及 member-inner/bridge 等精确交接尚未完成，不授予完整 A1.6 或 2.9 验收。2.10 停止/资源门仍未通过，S2 未进入。

## S1 2.9 检查点：raw class literal 与数组（2026-10-21）

[实施及验收](results/s1-raw-arrays-v1/README.md)修正 decoder 将 Unicode UTF-8 当作 MUTF-8 descriptor 重读，以及 builder 两处 Named array lossy 消费。对象叶到源码拼写时才严格解码，继续复用 reader descriptor facts 和既有名称门；保留 nested literal 的 `$` 段策略、类型 `_` 的 7/8 合同、primitive/newarray 与维数规则，不新增类型推断/来源投影/BCI 放行。

修复前实际 Java 572 passed / 3 新增 failed，后继同套测试 575 全通过；12 同源输入 × 两生成编译器达到 24/24 完整类原样编译、48/48 受控 JVM 对照，旧产品仅 8/24 编译。41 目标 1709 passed / 8 原失败 / 8 默认 ignored，A1.4 11 项、136×3 正文/核心及真实成功类重编保持。新矩阵不是独立 holdout；真实包未执行；此前完整 alias/MP/字段外部 oracle 未重复累计。534 Rust/manifest + 3 embedded 测试资产共 537 项与归档/构建/42 二进制匹配，target 清理。

这是既定 raw 名称合同的消费补齐，schema 8/恢复合同 10/解析合同 3/输入域 v5 保持，Java 实现摘要变化。member-inner enclosing owner、bridge cast 与其余精确链路仍由 2.9 承接，2.10 旧失败/停止/资源门不免除；OpenSpec 17/59，2.9/完整 A1.6/S1 未验收，S2 未进入。

## S1 2.9 检查点：allocation 与成员源码路径（2026-10-21）

后继 [allocation/成员源码路径检查点](results/s1-raw-member-paths-v1/README.md)沿 CP raw owner/descriptor、enclosing/capture/构造器、source path 与声明投影保留 JvmBytes；Allocate/Site/NewRecord/census 及相关消费者同步迁移，最后由既有严格 MUTF-8/源码名称门拼写。两项真实 IR 回归从 0/2 到 2/2 通过；32 同源输入完整类编译由 16/64，经 R1 的 56/64，最终到 64/64、128 JVM 与 32 raw NewRecord 对照通过；2 个 varargs 补充输入达到4编译/8JVM，从R3两项假partial转为当前complete，保留ellipsis。R1/R2 的 supplementary package 8 次编译失败保留；R4 修正原始解码、完整类型 token/已投影路径计数及 varargs ellipsis 边界，R3 probe 与 release 中断分列。最终 51 目标 1755 passed / 8 基线 failed / 9 默认 ignored：42 根层以 R4 重跑，9 个无根库依赖的下层目标按相同源码/依赖输入复用 R1，不重复计数。A1.4 11 项、136×3、旧数组 24 编译/48 JVM、21 名称与真实两类 4 重编保持。545 项当前源码/资产与 R4 构建相符；选定 43 个 R4 + 9 个 R1 二进制有各自收据，全部 11 个 target 清理。schema 9、恢复合同 11、解析合同 3、输入域 v6；NewRecord.class JSON 改为 byte array，外部报告格式合同仍由 E3/2.10 承接。Bridge 与其余精确消费者及 2.10 门未完成，2.9 不勾选，S2 未进入。

## S1 2.9 检查点：cast/type-test 与 bridge 身份（2026-10-21）

[本检查点](results/s1-raw-casts-v1/README.md)将 CP CheckCast/InstanceOf、BridgeParameterCast、canonical/recheck 与 bridge 目标声明定位贯通为 raw 字节；普通源码拼写在严格边界进行，generic interface Header 不再用标准 UTF-8 解释 MUTF-8。身份 helper 保持 bitwise，bridge 计划另外保留整个调用 descriptor 的源码门；对象类型 cast 域、SSA/顺序/BCI 与 evidence 守卫不扩大。迁移类型不序列化，schema 9/恢复合同 12/解析合同 3/输入域 v6，无新事实层或类型推断。

永久整类测试实际从 0/1 转为通过；24 同源输入完整类编译 40/48→48/48、96 JVM 对照与 24 raw descriptor 校验通过，非独立 holdout。52 目标以当前 R3 全部重跑，1764 passed / 8 基线 failed / 10 默认 ignored；新增 3 Java + 1 整类测试，另 5 passed/1 ignored 是扩入的既有引用 cast 目标。A1.4 11 项、136×3、原成员/varargs/数组/名称及真实两类重编保持，真实包未执行。549 项源码/资产匹配归档/构建及 53 二进制，7 构建命令成功，四个 target 清理；R1/R2 主动中断的职责审查修正与原流保留。补充平面显式返回 checkcast 的完整 shape 组合未单独执行；raw 方法跨度与静态初始化等剩余精确消费者待继续复现。2.9–2.10 与 S1 整体未验收，S2 未进入。

## S1 2.9 检查点：raw 初始化与字段投影（2026-10-21）

后继 [raw 初始化/字段投影检查点](results/s1-raw-projections-v1/README.md)修正实例数组 helper 的 raw owner/name/descriptor 精确交接，access bridge 的源码字段名复用字段声明 written_name，物理证明仍按 raw FieldRef 与字段表索引。两项实际旧失败的永久测试转绿；28同源变体预期投影12/28→28/28，完整类编译48/56→56/56，112受控JVM一致。数组只计投影误拒绝修复，8次新编译成功来自supplementary字段组，不称holdout。R2全部53目标1770 passed / 8既有failed / 10默认ignored；新增2测试，另4通过是扩入既有N1。A1.4的11项、cast/成员/varargs/数组/名称、136×3正文核心和真实成功两类重编保持，真实包未执行。R1 census失败由旧reader实际复现，R2只更新新增两class/八Code的cfg(test)精确人口，全部目标重新构建。555源码/资产、54二进制与R2来源匹配；本phase三个target清理。schema9/恢复合同13/解析合同3/inputs v6；匿名捕获、enum switch和Outer.super剩余消费者仍待复现，2.9–2.10/S1不勾选，S2未进入。


## S1 2.9 匿名类 raw 身份检查点（2026-10-21）

后继 [匿名类 raw 身份检查点](results/s1-raw-anonymous-v1/README.md)在现有 tuple/nested site 保存已验证 candidate.class，让 typed rows、resolver、capture/xref 与扫描对位消费 raw，最终接口/outer/super 拼写复用严格 MUTF-8。完整 census/唯一verified/BCI/descriptor/形状和原子提交守卫保持；super 两处 UTF-8 消费同时修正，旧两次 CLI panic 消失。三项永久反例0/3→3/3；48同源变体投影20→48、完整类编译40/96→96/96、192JVM一致。super16组内联2→8、编译4/32→16/32、32JVM一致，8组javac8原shape拒绝保留并由S5/6.9矩阵承接，不称holdout。R1全部55目标1783 passed / 8既有failed / 10默认ignored；新增3测试，另10通过来自扩入既有super目标。A1.4的11项、136×3、旧cast/初始化/成员/varargs/数组/名称与真实成功两类4重编保持，真实包未执行。577源码/资产、56二进制与7成功构建匹配，两个target清理；schema9/恢复合同14/解析合同3/inputs v6。enum-switch、Outer.super及2.10门仍未完成，2.9/S1不勾选，S2未进入。

后继 [enum-switch raw 检查点](results/s1-raw-enum-switch-v1/README.md)让 IndexCall、enum owner、values field/factory 与 descriptor 直接传递 JvmBytes；用同次IR的anewarray CP raw类名核对元素，最终label严格MUTF解码，保留ordinal/对象/构造器/闭合helper/handler/BCI守卫。28同源输入投影8→14、完整类编译16/56→28/56、56生成JVM一致；12个raw拒绝中仅6个取得整类恢复，另6个显露后继canonical拒绝，当前14组javac8原形状缺口归S5/6.9。两组private metadata名字变化在JVM8/23先校验，再由旧0/2投影、0/4编译到当前2/2、4/4和8JVM一致；不称holdout。R3两项末尾断言失败原流保留；R4修正注释显示与可执行源码断言，并用修订后的同一测试源码在旧实现重建得到0/2、当前2/2。统一R4全部55目标1785 passed / 8既有failed / 10默认ignored；A1.4 11项、4×3 enum evidence、136×3、既有保留矩阵与真实成功两类4重编保持，真实包未执行。589源码/资产、56二进制及7构建相符，六个phase target清理；schema9/恢复合同15/解析合同3/inputs v6。IndexCall报告raw数组格式门归E3/2.10；Outer.super、annotation/default、generic/constructor等仍有静态raw边界待实际复现，2.9–2.10/S1保持未验收，S2未进入。

后继 [Outer.super raw 交接检查点](results/s1-raw-outer-super-v1/README.md)把目标 owner/name/descriptor 直接作为 JvmBytes 交给 Java，validator/参数/返回消费 raw，最终方法名与 needle 严格 MUTF-8 拼写；共用完整 descriptor 与规范类名门，保留 physical/SSA/BCI/闭合/overload/异常/事务守卫，无 String/raw 双轨。六源码文件，schema10/恢复合同16/解析合同3/inputs v7。两项同测试源码旧0/2→当前2/2；effects28完整类编译24/56→56/56、112生成JVM一致；补齐实际分析 Header 的reference12完整输入8/24→24/24、48JVM一致，12raw descriptor/physical method及UTF-8跨度核对通过。首次误查Member及随后缺Parent/Token的旧探针原流保留，不充作完整环境基线；合法`_`父类复用规范拼写后4组8编译/16JVM已含effects。统一R3全部55目标1788 passed / 8既有failed / 10ignored，A1.4全11、7×3 evidence、136×3及既有保留矩阵/真实两类4重编保持，未执行真实包。601源码/资产、56二进制和7成功构建匹配，四target清理；R1类型错误与R2中间收据保留。两矩阵是开发输入，不称holdout。其他raw边界及facade内剩余源码判据职责继续承接，2.9–2.10/S1未验收，S2未进入。


## S1 A1.1 / 2.10 Outer.super 职责承接（2026-10-21，局部验收）

[Outer.super 源码职责迁移](results/s1-outer-super-source-policy-v1/README.md)完成本项 A1.1 局部承接：泛型/Exceptions、源码重载、已发布参数和 SSA direct-load/严格子类判据归 Java；root 仅保留物理一致性、选定事实供给和原 execution 合并，census/事务保持。只改两生产源码，原测试体与接受域不变；55目标1788 passed / 8既有failed / 10默认ignored。40完整输入80编译/160受控JVM保持；40报告及36开发输入的42调用在校验两个确切实现摘要变更后，其余字段及逻辑usage一致，7×3 evidence、136×3与真实两类4重编保持，真实目标未执行。首次比较器未处理源码实现身份变化的失败原流保留，补跑后验收；schema10/恢复合同16/解析合同3/inputs v7不变。601源码/资产、56二进制与七成功构建匹配，target清理；实际Body/IR复用和所有职责的总盘点仍归本项后续，2.9–2.10/S1不勾选，S2未进入。


### S1 2.6 当前构建验收复核（2026-10-21，v14）

本次指定 2.6 的 [v14 当前构建验收](results/s1-environment-recheck-v14/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全部 11 项及相邻回归 **286 passed / 0 failed / 1 既有 ignored**。三处 memo、defining loader 父链、caller IR/prepared 快捷路径与热命中取消保持。本轮实际重跑136×3正文/核心、6编译/12受控JVM、真实成功两类4重编，全部通过，未执行真实目标。601源码/资产及56二进制匹配源码职责迁移R1，核验其10529个冻结payload；target缺席，无新增Cargo、产品代码或测试体修改。本次止于2.6，未继续2.9；2.9–2.10/S1未验收，S2未进入。


### S1 2.9 interface-super raw 身份检查点（2026-10-21）

后继 [interface-super raw 检查点](results/s1-raw-interface-super-v1/README.md)删除String候选实体，CP三元组到Java消费保留JvmBytes，最终拼写复用canonical类名和共享完整method门；default/层级/loader/entry-this/停止守卫保持。两项同源码永久反例0/2→2/2；56同源开发输入qualified-super32→56，完整类编译64/112→112/112、224生成JVM一致；JVM合法bad-name的错误Structured改为带raw/alias/BCI的拒绝，孤立surrogate和损坏MUTF拒绝保持，后者未执行。55目标1790 passed / 8既有failed / 10默认ignored；A1.4全11、24新evidence、80旧Outer编译/160JVM、136×3及真实两类4重编保持。首次比较器混入可选详情失败原流保留，五门补跑后独立核验九门；不称原流水线全绿。606源码/资产、56二进制和七成功构建匹配，两个target清理，峰值958433241 bytes。六生产文件，schema11/恢复合同17/解析合同3/inputs v8；facade其余源码策略、annotation/default/generic/constructor等与2.10门继续承接，2.9–2.10/S1不勾选，S2未进入。


### S1 2.9 metadata字面量职责与raw检查点（2026-10-21）

后继 [metadata字面量职责/raw检查点](results/s1-raw-metadata-literals-v1/README.md)将既有MemberDefault、ConstantValue/default与注解元素纯解析/拼写归Java，root保留同一属性读取/停止/隔离/类组装；复用JvmString和emitter转义表，区分名称标识符门与字符串UTF-16值、损坏MUTF的Err。两项同源码永久反例0/2→2/2，另8旧项保持；56双目标同源开发输入编译96/112批次→112/112（224完整source units）、行为32/56→56/56、224生成JVM一致。修复NoBody损坏default被替换字符且Complete；合法不可表达enum名不写非法token，raw属性保留，但外层default Option的“缺省/源码拒绝”合并仍是确认盲区，须由本项后继闭合，不能宣布A1.6完成。56目标1801 passed / 8既有failed / 10默认ignored；42evidence、112raw来源/普通body-map、136×3、6编译/12受控JVM及真实两类4重编保持。613源码/资产、57二进制与R2七成功构建匹配，三个target清理，峰值974082806 bytes；R1缺导入失败原流保留。schema11/恢复合同18/解析合同3/inputs v8；2.9–2.10/S1不勾选，S2未进入。


### S1 2.6 当前构建复核（2026-10-21，v15）

本次指定 2.6 的 [v15 当前构建复核](results/s1-environment-recheck-v15/README.md)实际重现旧构建 2 个 loader / 7 个环境身份失败；当前 A1.4 全 11 项和相邻回归 **288 passed / 0 failed / 1 既有 ignored**。完整环境声明、defining loader 后继、三处缓存/caller 快捷路径及热命中取消保持，无新增 2.6 产品修改。613 项源码/资产及 57 二进制匹配 metadata R2，核验 8,452 个冻结 payload；target 缺席，本轮无新增 Cargo。实际重跑 136×3 正文/核心、6 编译/12 受控 JVM、真实成功两类 4 重编均保持，未执行真实目标。首次旧脚本数量断言失败与修正后完整复跑分别保留。2.6 保持通过，2.9–2.10/S1 未验收，本轮止于 2.6，S2 未进入。


### S1 metadata 缺省与拒绝状态闭合（2026-10-21，局部验收）

后继 [metadata 拒绝状态检查点](results/s1-metadata-refusal-v1/README.md)让 Java 的已存在值 resolver 返回 Result<MemberDefault>，root None 仅表示属性缺失；方法复用 Refused，字段复用标记/诊断并隔离局部错误，全局预算/取消仍截断。四生产文件，无新 DTO/Session/cache/crate；恢复合同 19，schema11/解析3/inputs v8。相同测试源新两项旧0/2→当前2/2，原10项保持；24开发输入的16个错误Complete改为Partial，8损坏仍Failed且保留邻接成员。56目标1803 passed / 8既有failed / 10ignored，A1.4全11、既有metadata112编译批次/224JVM、42evidence与112raw/body-map、136×3、6编译12受控JVM及真实两类4重编保持，未执行真实目标。616源码/资产、57二进制与七成功构建匹配，三个target清理；四紧预算探针通过，只声称逻辑Body。首次测试API错误、状态验证器摘要算法假设及预算成本假设原流保留，纠正后六门独立验收，不称首次流水线全绿。2.9–2.10/S1仍未验收，S2未进入。


### S1 2.6 当前源码验收（2026-10-21，v17）

本次指定 2.6 的 [v17 当前源码验收](results/s1-environment-recheck-v17/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。当前 618 项源码/资产与归档、三成功构建命令和 7 二进制匹配，target 清理；136×3 正文/核心零差异、6 完整类编译/12 受控 JVM、真实成功两类 4 重编全部实际重跑通过，未执行真实目标。5079 前继冻结载荷校验通过。既有 2.9 overlay 的 R4 构建中断、无 receipt，未重启或追认通过，仅清理废弃 target；本轮未改生产源码或测试体。2.6 保持通过；本轮止于 2.6，2.9–2.10/S1 未验收，S2 未进入。


### S1 2.9 metadata 类型路径职责与 raw 上下文（2026-10-21，局部验收）

后继 [metadata 类型路径检查点](results/s1-metadata-type-paths-v1/README.md)将已存在 ResolvedInnerClass 迁入 Java naming，借用同次 raw rows 为普通 descriptor 声明、注解使用及递归默认值提供统一路径；root 只读取 raw default、供给上下文并保留停止/隔离/类族事务。五生产文件，无新增resolver/read/cache/Session/crate；恢复合同20、schema11/解析3/inputs v8。20同源输入的40完整编译批次0/40→40/40（80 source units）、80生成JVM一致；保留真实 `$`、深层及supplementary边界，40 raw报告/30 evidence对照保持，60处type annotation spelling逐项验证。两root旧反例转绿，另两Java unit；新重复/矛盾row拒绝与BodyProbe逻辑Body4→3通过，不声称actual IR。R5七构建成功，56目标1807 passed / 8既有failed / 10ignored；78完整类文本、24旧状态/四预算、旧metadata112编译/224JVM及42 evidence、136×3与真实两类4重编保持，真实包未执行。618源码/资产、57二进制匹配，六target缺席，5079前继载荷核验。R1/R2错误、R3新增失败、R4中断及验证器错误原流保留；R5首流水线因raw/spelling DTO比较退出，修正后补跑七门并独立验收十门，不称首次全绿。Signature/generic、Body类型上下文、constructor/private owner及2.10总门继续按[next-boundaries](results/s1-metadata-type-paths-v1/next-boundaries.md)承接，2.9–2.10/S1不勾选，S2未进入。


### S1 2.6 当前源码验收（2026-10-21，v18）

本次指定 2.6 的 [v18 当前源码验收](results/s1-environment-recheck-v18/README.md)实际重现旧构建 2 个 loader / 7 个身份失败；当前 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。三处 memo、defining loader 后继、caller 快捷路径及取消热命中保持，本轮无生产代码或测试体修改。进入本轮时在途的 Signature R1 正常完成，621 源码/资产、58 二进制与七成功构建匹配，target 清理；仅借该构建验收 2.6，未执行 Signature 产品矩阵或追认 2.9。136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持，8405 前继载荷核验，真实目标未执行。本轮止于 2.6；2.9–2.10/S1 未验收，S2 未进入。


### S1 2.9 Signature raw 名称与拼写职责（2026-10-21，局部验收）

后继 [Signature raw 名称检查点](results/s1-raw-signature-names-v1/README.md)将递归 Signature 拼写及 body TypeVariable 出口迁入现有 Java 模块，全部类型变量声明/引用共享严格 MUTF-8 标识符决定；root 保留 raw scope/erasure、成员事实、停止与类事务。四生产文件，无新读取/resolver/cache/crate；恢复合同21，schema11/解析3/inputs v8。四项同源码永久反例0/4→4/4；48同源javac输入泛型32/48→48/48、反射行为128/192→192/192，96/96编译保持，不称编译净收益或holdout。57目标1813 passed / 8既有failed / 10ignored；36 evidence、48 raw/body/map、完整报告预期差异及逻辑Header/Body零增量通过，12负例拒绝泛型/损坏Failed且未执行。旧metadata112编译224JVM、20类型路径40批次80JVM、78完整源码、24状态/四预算、136×3及真实两类4重编保持，真实包未执行。621来源/资产、58二进制与七成功构建匹配，baseline/product target清理，8405前继载荷核验。首负向门因验证器误要求损坏输入exit0失败，原流保留并独立核对exit4后补跑八门，不称首次全绿。后续物理field/method名、constructor/private owner、Body类型上下文及2.10门沿[next-boundaries](results/s1-raw-signature-names-v1/next-boundaries.md)承接；2.9–2.10/S1不勾选，S2未进入。


### S1 2.6 当前源码复核（2026-10-21，v19）

本次指定 2.6 的 [v19 当前源码验收](results/s1-environment-recheck-v19/README.md)先实际重现固定旧构建 2 个 loader / 7 个身份失败；当前源码新构建 A1.4 全 11 项及相邻回归 **288 passed / 0 failed / 1 既有 ignored**。四个环境/loader/facade/身份测试文件与 v18 归档逐字节一致，未发现新增可复现 2.6 缺口，本轮未改生产代码或测试体。624源码/资产、归档及7二进制匹配三成功构建，target已清理；136×3正文/核心、6编译/12受控JVM、真实成功两类4重编实际重跑保持，未执行真实目标。前继Signature 10122冻结载荷核验；既有2.9 generic physical overlay的R2新测试4/5及后续未验收改动保留，不授予2.9通过。本轮止于2.6，2.9–2.10/S1未完成，S2未进入。


### S1 2.9 泛型物理名称检查点（2026-10-23）

后继 [泛型物理名称检查点](results/s1-generic-physical-names-v1/README.md)让同类调用key/库存保留raw三元组，field/method词法及六处AST连接共享严格MUTF标识符出口，constructor规范owner连接保留同次SSA/完整使用/停止守卫。五生产文件，无新增读取/resolver/cache/Session/crate或序列化字段，恢复合同22，schema11/解析3/inputs v8。五项同源码永久反例0/5→5/5；60输入泛型反射40/60→60/60，120重编/240正常运行均保持，收益为元数据恢复。61目标1869 passed / 8既有failed / 10ignored，其中补入三个既有generic目标49项全通过；45evidence、60raw/body/map、108套绑定预期更新及完整报告差异门通过。六负例保留raw/显式别名/泛型拒绝或准确Partial/Failed前缀，未执行变异输入。Signature96编译192JVM、metadata40/80及112/224、78源码/24状态/四预算、136×3与真实两类4重编保持，真实目标未执行；field-write/setter两组R2已绿控制48编译96JVM保持。674来源/资产、62二进制和八成功构建匹配，target清理，10122前继载荷保持；准确同字节CLI复用已实际运行的60组矩阵，无重复计数。R1 check/R2 native及R3检查器误判Partial的失败原流保留，补跑12门后覆盖18门，不称首次全绿。后续Body类型上下文与2.10总门承接，2.9/2.10/S1仍不勾选，S2未进入。

### S1 2.6 当前源码复核（2026-10-23）

[v20 收据](results/s1-environment-recheck-v20/README.md)保留固定旧构建 2+7 个失败和当前新构建 A1.4 全 11 项通过；相邻回归 288 passed / 0 failed / 1 ignored，两个旧调用签名的最小构建修复另有 6 项现有 enum 回归。环境身份、defining loader 后继及 memo 机制未新增修改。136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持。补充 include 输入的采集时点与覆盖限制如实单列，不将编译后补采称为全量预构建冻结，也不将 A1.4 通过当作现有 2.9 overlay 或 S1 整体验收；本次止于 2.6，S2 不进入。


### S1 2.9 Body 类型上下文检查点（2026-10-23）

[Body 类型上下文检查点](results/s1-body-type-names-v1/README.md)将完整 raw InnerClasses rows 沿 MethodDeclaration/Java facts/Declaration/retained AST 共享，commit/replay/fragment 复用同一 SourceClassNames，删除成员名列表双轨；具名边核验后投影，local/anonymous/无行保留物理拼写。两项永久反例0/2→2/2，60开发输入重编48/120→120/120、240隔离JVM精确匹配；65目标1897 passed / 8既有failed / 10ignored，R3两项匿名回归已在R4恢复且原流保留。300绑定/60raw与map/120额外evidence通过，既有泛型、Signature、metadata与136×3/真实两类4重编保持，真实包未执行。2,624源码/资产和66二进制匹配，target清理；schema12/恢复23/解析4/inputs v9。v1原流遗失与v2历史generator pin失配已单列，不冒充全部输入历史可核验。嵌套enum child-context待实际复现，逐方法解析/资源总门归2.10；2.9/2.10/S1不勾选，S2未进入。


### S1 enum上下文假设更正与raw连接承接（2026-10-23）

[嵌套enum上下文复核](results/s1-nested-enum-context-v1/README.md)关闭先前未闭合的可到达假设：terminal body Some在当前Engine唯一生产点必带delegation与constructor field index，nested render在二次emit前拒绝；正式8输入的实际拒绝一致，两个javac23物理家族重编和4次JVM保持。248个pins匹配，历史harness/空stdout核验错误已保留并分列，只有closure-v3作正式结论。不新增行表或放宽委托域；死分支清理归2.10。沿链发现活跃enum名称连接的11类非BMP失败，进入[enum raw名称实施](results/s1-enum-raw-names-v1/README.md)，当前旧源码四测试1通过/3失败，产品未验收；2.9/2.10/S1不勾选，S2未进入。

### S1 2.6 当前构建验收（2026-10-23，v21）

本次指定 2.6 的 [v21 当前构建验收](results/s1-environment-recheck-v21/README.md)引用已实际复现并核验原流的旧 2+7 失败（本次不重复计数）；当前 A1.4 全 11 项通过，六目标 288 passed / 0 failed / 1 既有 ignored，另 6 项既有 enum 回归通过。环境/provider/callee/facade/身份测试五文件与 v20 逐字节相同，无新增 2.6 产品或测试体修改。借用在途 enum R1 构建的 9 条成功命令，2,640 项来源与归档、67 二进制匹配，target 清理；136×3 正文/核心、6 编译/12 受控 JVM、真实两类 4 重编实际重跑保持，未执行真实目标。已有 2.9 enum 草稿未取得产品矩阵/负例/evidence 验收，不借本次结果追认；2.9/2.10/S1 未完成，S2 未进入。


### 当前构建的 A1.4 保持与 enum 名称局部迁移（2026-10-23）

[enum R3 收据](results/s1-enum-raw-names-v1/acceptance-r3.json)将已有raw命名出口接入enum精确消费者，保留descriptor外壳与raw身份连接，不新增缓存/Session/读取模型。2.6的完整有序环境、defining loader后继和memo片段不变；全11项及相邻288通过。名称迁移属2.9局部检查点，不能归为新增身份修复。

原四永久测试1/3→4/0，第五R2失败→R3通过；132矩阵40→60投影的R1收益在R3源码精确保持（120编译/240JVM按同源码引用，不重复计数），十个`_`输入在R3新增20编译/40JVM。66目标1902 passed / 8既有failed / 10ignored，负例/evidence与168旧完整源码+78声明保持；136×3、6编译12受控JVM、真实两类4重编通过，真实目标未执行。evidence比较器漏排除method RuleDetails.fields的失败及v2修正后补验分列。2643来源/67二进制匹配九步构建，target清理。剩余raw候选未复现，资源/actual read/IR及E1/E3由2.10承接；2.9/2.10/S1未完成，S2未进入。

R3负例补充：terminal helper的dash/surrogate名字在正文原样泄漏且缺结构化raw/alias，报告仍Complete；enum候选拒绝并未覆盖此方法名称合法性，不能外推全类正确。该真实消费者缺口仍归2.9，当前用户2.6范围不扩展处理，见[负例逐项分类](results/s1-enum-raw-names-v1/negative-review-r3.md)。


### 方法声明与调用共享拼写（2026-10-23，局部验收）

[调用名称检查点](results/s1-invocation-name-boundary-v1/README.md)把CallTarget诊断视图与Java方法名出口分开：普通声明、普通调用和特殊调用拼写共用raw MUTF决定，Java7/8 `_`保持同名，不可表达调用按既有fallback保留raw/BCI。普通门放在原参数/varargs检查后、AST提交前，完整owner/descriptor及停止/来源守卫保持；未新增事实模型或缓存。恢复合同24→25，其余schema12/解析4/inputs v9不变。

四永久测试1/3→4/0，16同源开发输入的32生成编译30→32、JVM60→64，target7 `_`另4编译8JVM；24份非法名称错误Structured改为Mixed/Fallback，12损坏报告仍Failed，变异不执行。67目标1906 passed / 8既有failed / 10ignored，A1.4全11及相邻288保持；前继完整报告/有效源码与136×3/真实成功重编保持。初harness空源码、基线排序、委托构造器比较器与后处理解析错误分列保存，未把工具错误算产品收益或首次全绿。2669来源/68二进制和9构建匹配，target清理。lambda/MethodHandle及generic/enum独立门与其余raw消费者尚未闭合，2.9/2.10/S1不验收，S2未进入。


### MethodHandle/helper 名称连接与开放提交门（2026-10-23）

[MethodHandle/helper 检查点](results/s1-lambda-handle-names-v1/README.md)、[收据](results/s1-lambda-handle-names-v1/acceptance-r4.json)：六生产文件复用 Java 方法名门与声明别名，具名非lambda$ MethodHandle拒绝不可表达名称，闭合helper的inline/rename按严格MUTF连接；声明和暂存别名冲突不得合并不同raw名，原同raw重载保持。恢复25→26，schema12/解析4/v9不变。相同测试源码旧2/4→6/0；16合法开发输入完整类编译28/32→32/32、生成JVM56→64；68目标1912 passed / 8既有failed / 10ignored，A1.4全11/相邻288保持。45负例、声明/暂存碰撞与重载保留、旧完整报告/源码及136×3、6编译12受控JVM、真实两类4重编通过，真实目标未执行。2700来源及归档匹配，显式选择12个R1下层+57个R4根层/CLI，不称同一次69构建；所有target清理。

**开放门不能省略**：captured-this kind7/ASCII helper原编译1/原JVM2成功，旧生成2编译失败，当前all完整源码逐字相同且无投影，诊断为空。本轮只实际运行一个输入；其它预想输入/证据未运行。13个独立门通过，instance门失败，不能宣告2.9通过。工具javap文本/目录冲突与真实缺口的原流分别保留，成功oracle未重复计数。未闭合helper、独立方法及碰撞caller fallback、其它raw消费者和2.10总门继续承接，2.9/2.10/S1未完成，S2未进入。


### captured-this与static helper共用需求闭包（2026-10-23，局部验收）

[captured-this共享需求门](results/s1-helper-demand-v1/README.md)、[收据](results/s1-helper-demand-v1/acceptance-r1.json)关闭前继kind7需求遗漏：header不再要求static，精确raw身份、kind/flags、完整使用、单site、同次AST及停止/提交守卫保留。两生产文件，恢复26→27；schema12/解析4/inputs v9不变。相同七测试旧6/1→当前7/0；ASCII/supplementary×inline/rename四开发输入完整生成编译0/8→8/8、16生成JVM一致，非holdout；首旧oracle按完整源码精确相同引用并独立核验，不冒充重新运行。固定输入essential/all的actual Body解码4、IR六阶段各4、entry读取2次833字节与红基线相同；scan/AST保留和逻辑成本增加，不外推完整S1资源门。五变异15完整拒绝报告及明确IR预算Partial无提交通过，变异未编译/执行。68目标1913 passed / 8既有failed / 10ignored，A1.4全11/相邻288保持；L110完整报告、旧普通/enum/generic/Signature/Body/声明及136×3、6编译12受控JVM、真实两类4重编保持，真实目标未执行。

初构建被系统中断，39前缀产物仅有五步Cargo成功事件，退出码/峰值/独立rustc记录缺失；接续四步实际exit0生成30产物，69项显式来源选择、2701来源/归档匹配，target清理。不称九条正常退出或完整S1资源通过。11门和finalize通过；未闭合/helper独立方法与碰撞后caller的最终fallback、其余raw消费者及2.10总门仍未完成，OpenSpec17/59，2.9/2.10/S1不勾选，S2未进入。


## S1 2.9 lambda 最终消费检查点（2026-10-23）

[lambda 最终消费检查点](results/s1-lambda-final-consumption-v1/README.md)、[收据](results/s1-lambda-final-consumption-v1/acceptance-r3.json)（2026-10-23）：五生产文件让 raw Member、同次 BCI envelope、公用名称终结与私有 provisional 类候选闭合；root 复用精确 ExplanationOnly marker 及现有原文/span 门。未闭合引用不输出非法调用，整方法 quote 同步撤销丢弃的 lambda/accessor 状态；保留闭合 helper 和 captured-this。正式旧十项7/3，R3原十项加新增合法 sibling 共11/0，另一个root marker/span单元通过；R1有效投影退化和R2合法sibling误报的实际失败原流保留，不能称首次全绿。R3同次九命令成功，2708来源/归档、69二进制匹配，target清理，恢复27→28/schema12/解析4/inputs v9。68目标1918 passed/8既有failed/10ignored；27拒绝、三组合9控制/18变异、合法sibling三evidence和16公共预算stop/8零预算拒绝通过。组合原3编译6JVM、生成6编译12JVM在R2已成功，R3核验完整源码和产物/原流后引用，不重复计数；变异未编译/执行。122 lambda/captured-this（9旧非法文本改quote）、84 invocation、132 enum、adapters、168旧源码/78声明、当前136×3/6编译12受控JVM及真实两类4重编保持，真实目标未执行；固定captured-this实际Body4/IR六阶段各4/read2/833bytes保持。其它raw消费及完整职责/停止/资源门按[next-boundaries](results/s1-lambda-final-consumption-v1/next-boundaries.md)承接，2.9/2.10/S1不勾选，S2未进入。


## S1 2.9 数组 helper raw 交接检查点（2026-10-24）

[数组 helper raw 交接检查点](results/s1-array-helper-raw-v1/README.md)、[收据](results/s1-array-helper-raw-v1/acceptance-r2.json)（2026-10-24）：删除Site三份helper String，以同次IR的BCI/CP连接raw handle，最后统一命名；Java层接管整个声明/返回类型匹配，facade只供选中事实和原事务。四合法开发输入数组引用投影1/4→4/4，旧生成8编译/16JVM原本成功，不算编译成功率收益。R1新native实际失败后补齐遗漏门，R2新测试1/0、旧数组16/0；12报告、6新编译/12受控JVM和六变体36恢复报告通过，变异未编译/执行，损坏编码保留明确失败/可靠前缀。70目标1935 passed/8既有failed/10ignored；A1.4全11及相邻288/0/1、旧能力与当前136×3/6编译12JVM/真实两类4重编保持。2713来源/归档与71产物匹配，九命令同次正常成功、target清理；恢复28→29/schema12/解析4/inputs v9。比较器三类错误和原流保留，成功oracle核验后引用，不称首次全绿。其余raw消费者仅静态清点，按[后续边界](results/s1-array-helper-raw-v1/next-boundaries.md)继续2.9/2.10；OpenSpec17/59，S1未完成，S2未进入。


[write-accessor raw descriptor 检查点](results/s1-write-accessor-raw-v1/README.md)、[收据](results/s1-write-accessor-raw-v1/acceptance-r1.json)（2026-10-24）：复用 reader descriptor components 和现有 source_names.current_class，将已有 key/expected_descriptor 保留为 JvmBytes，字段/header/frame/成对发射共同核对 raw 身份；仅源码 Type 走共享严格命名，无新增读取/缓存/身份实体。两个生产文件，恢复29→30，schema12/解析4/inputs v9保持。四合法开发控制 helper Structured 1/4→4/4，完整生成编译2/8→8/8；三个变化源码新增6编译12受控JVM，ASCII同字节引用旧2编译4JVM。12控制报告、六变体36选定负例通过，变异未编译/执行；损坏字段按Partial可靠前缀/skipped fields[0,1)交付。71目标1936 passed/8既有failed/10ignored，A1.4全11/相邻288/0/1、旧能力、当前136×3/6编译12JVM和真实两类4重编保持，真实目标未执行。2718来源/归档与72二进制匹配，九命令同次正常成功、target清理。Partial/Failed与enum嵌套合同比较器失败原流保留，续验复用成功控制，不称首次全绿；冗余负例重跑不累加。剩余六行/五机制静态消费者及2.10资源/停止总门见[后续边界](results/s1-write-accessor-raw-v1/next-boundaries.md)，ASCII WA三项旧失败未修复；2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。
### 泛型消费者在源码边界统一命名（2026-10-24）

[泛型消费者源码拼写检查点](results/s1-generic-source-consumers-v1/README.md)、[收据](results/s1-generic-source-consumers-v1/acceptance-r2.json)（2026-10-24）：raw receiver/List元素共享严格名称出口，同次AST raw InnerClasses闭合真实成员路径，字面$不猜嵌套；四生产文件，恢复30→31/schema12/解析4/inputs v9保持。六开发控制的泛型反射8/24→24/24，完整生成编译12/12保持；当前新8编译16JVM，两个同字节ASCII复用旧4编译8JVM，非holdout。两个dollar类保留旧Partial/anonymous_typed_nesting_unproved，不能称Complete。R3错误class<T>控制与R1新test2/1原流保留，R2修正测试3/0；补入旧List目标19/0/1。73目标1958 passed/8既有failed/11ignored；18正报告、七变异42报告、15局部保留及旧能力门满足。A1.4全11/相邻288保持；136×3/6编译12JVM/真实两类4重编核验同CLI的v24冻结原流后引用，未重复执行，真实包未执行。2726来源/两归档匹配，74binary显式选R1排除错误oldtest+R2testfix，不称完整R2同次构建，target清理。实际新复现overload raw cast：class-scope T/E的ASCII反射4/4、supplementary0/4，当前CLI原文保持缺口，下一步只迁移AST拼写并保留raw语义binary_name；其它消费者及2.10总门按[next-boundaries](results/s1-generic-source-consumers-v1/next-boundaries.md)继续。2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

本项的成员路径费用仅是 row-visit AnalysisSteps 上界；stated_path 内部逐边取消与实际CPU/read/IR/live/RSS/时间仍需2.10实测。root的generic_void_body_declaration及selected_nested_sibling_bound_spelling仍含纯Java策略，纳入A1.1职责收尾；本项不以命名修复关闭它们。

### 显式bound cast保留raw身份（2026-10-24）

[显式bound cast raw身份检查点](results/s1-generic-cast-source-v1/README.md)、[收据](results/s1-generic-cast-source-v1/acceptance-r1.json)（2026-10-24）：AST Type复用共享严格名称出口，后续SignatureType::Class.binary_name直接保留raw MUTF；候选/explicit bound/most-specific/flags/census守卫不变，两个生产文件，恢复31→32/schema12/解析4/v9。原两输入2编译4JVM、旧生成4编译8JVM核验，relay泛型反射4/8→8/8，生成编译4/4保持；supp新2编译4JVM，ASCII同字节引用旧2编译4JVM，非holdout。六正报告/七变异42报告通过；前六保留泛型拒绝/擦除，截断Failed可靠方法前缀1/4、skip[1,4)，变异未编译执行。74native1959 passed/8既有failed/11ignored，新永久test1/0；U正负39、T/memberpath15及旧能力门保持，A1.4全11/相邻288保持；当前32 CLI实际重跑136×3/6编译12受控JVM/真实两类4重编通过，真实包未执行。2729来源/归档、75binary/74cleanup与九命令同次构建匹配，target清理，U4952冻结payload核验。root generic void/nested bound源码策略按[职责输入](results/s1-generic-cast-source-v1/responsibility-audit-input.md)归既有2.10/A1.1；nested bound/raw span/needle及资源退出门仍待，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

## S1 泛型声明边界第三轮验收（2026-10-24）

[泛型声明职责与raw sibling检查点](results/s1-generic-declaration-boundary-v1/README.md)、[第三轮收据](results/s1-generic-declaration-boundary-v1/acceptance-r3.json)（2026-10-24）：Java统一generic-void的Option<Result>选择/严格生成及共享generic方法声明拼装，root保留事实/原候选顺序/停止/提交；三生产文件，无新DTO/Session/cache/read。source_class_name/source_identifier替代三个标准UTF8边界，raw唯一sibling/同outer/binary重建/physical $拼写门保持，恢复32→33/schema12/解析4/v9。两开发输入selected-leaf泛型反射2/4→4/4，JDK23生成编译2/2保持；supp新1编译2VM、ASCII同字节引用旧1编译2VM。JDK8 leaf编译exit4旧限制保留，不称完整生成族或holdout收益。R3六门正常通过：14native957/0/1，6正报告/六metadata变体36报告、168完整源码/78声明保持，当前136×3/6编译12受控JVM/真实两类4重编通过，真实目标及变异未执行。2732来源/构建前/归档、15二进制与四正常命令同次匹配，target清理；R1编译失败、R2成功但route职责遗留、误选R2归档的审查原稿均保留，R3已闭合本链。其余泛型专门策略及2.10资源门按[next-boundaries](results/s1-generic-declaration-boundary-v1/next-boundaries.md)继续，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

## S1 泛型专门声明策略迁移（2026-10-24）

[泛型专门策略迁移](results/s1-generic-declaration-policies-v1/README.md)、[收据](results/s1-generic-declaration-policies-v1/acceptance-r1.json)（2026-10-24）：Java接管method-local empty-void throws、instance null-return、static local throws三条策略的Option<Result>路由/严格准入/完整声明，root保留事实、原顺序、停止与提交；复用共享header，无新DTO/Session/cache/read。实际以W33 CLI复现只改Signature CP #11的interface-only bound导致exit101/panic，Code不变；Java显式要求class-bound且无附加interface-bound后局部拒绝，保留物理方法/正文/Exceptions，合法控制保留泛型。恢复33→34/schema12/解析4/v9。16选定native961/0/1，新增永久反例1/0；静态throws既有3测试扩入cohort，不称新增恢复收益。当前原/变异6报告、W正6报告/负18报告通过；对W33的18负例源码/正文/完整coverage/前缀严格保持，对W32另18比较仅3个bridge旧拒绝文案不同并显式记录，不称完整JSON一致。168完整源码/78声明及当前136×3/6编译12受控JVM/真实成功两类4重编保持，真实目标及变异未执行。2735来源/构建前/归档、17产物与四成功命令同次匹配，target清理，构建峰值755250698bytes不冒充S1资源验收。W6218冻结payload全部核验保持；剩余generic body/ordinary/no-body/constructor/class-scope策略及bridge/needle/2.10资源停止门按[next-boundaries](results/s1-generic-declaration-policies-v1/next-boundaries.md)承接，八项既有失败未修复或重测，本次非全仓门；2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

## S1 同类泛型body声明职责（2026-10-24）

[同类generic body声明迁移](results/s1-generic-body-declaration-v1/README.md)、[收据](results/s1-generic-body-declaration-v1/acceptance-r1.json)（2026-10-24）：完整bounds/formals/return sources/throws准入与header组装归Java，root只核对certificate↔physical member、供事实/原顺序/停止/提交；复用descriptor/严格命名/共享header，无新DTO/Session/cache/read。两生产文件纯迁移，合同34/schema12/解析4/v9保持。现有预算sweep加入method-local T/U的MethodShadow，不新增test function；扩入既有same_class_generic_binding16项，17选定native977/0/1。四开发context×三evidence的12报告真实进入本route，当前源码/方法/execution/完整coverage/logical usage与旧X一致（仅测量elapsed排除）；旧完整生成8编译16VM及原oracle16VM已绑定，当前同字节复用，非holdout/恢复域收益。旧/当前同130cutoff全匹配，32个双body完整停止点均0或2泛型头。普通UTF8非MUTF name只改CP #8，旧/当前各3报告均Reader Failed，methods[1]拒绝且保留init前缀/完整coverage，未进genericroute，变异未编译执行；任意raw helper域差异不能冒充产品可达域变化。168源码/78声明、当前136×3/6编译12受控JVM/真实两类4重编保持，真实目标未执行。2735来源/归档/构建前与18产物、四正常命令匹配，target清理，峰值770427606bytes仅是构建守卫。target对象审查、elapsed比较、复制stem与tuple序列化四类工具失败原稿/原流保留；续验复用成功native、3个CLI前缀和130预算原流，不称首次七门全绿。剩余ordinary完整路由/no-body/constructor/class-scope、bridge/needle与实际read/IR/资源/停止门见[next-boundaries](results/s1-generic-body-declaration-v1/next-boundaries.md)；八项既有失败不在本轮分母，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

## S1 普通参数化声明策略迁移（2026-10-24）

[普通参数化声明策略迁移](results/s1-ordinary-parameterized-policy-v1/README.md)、[收据](results/s1-ordinary-parameterized-policy-v1/acceptance-r1.json)（2026-10-24）：Java接管完整ordinary准入/返回来源/throws/声明与proof分类及exact-null选择，root保留同次事实、物理身份、有序路由、停止和提交；Signature/descriptor/varargs现有实体迁入Java并统一generic解析，无新DTO/read/cache。三生产文件纯迁移，合同34/schema12/解析4/v9保持。六门有效验收：22native1023/0/2，9开发context的27evidence+2预算逐mode源码/方法/execution/coverage/logical usage保持；实际19方法成功/4旧拒绝，Generic族单列。继承body12/130cutoff/name3保持，65compact error完整JSON核对，32个双body完整partial点无半项泛型头；168源码/78声明及当前136×3、核心6编译12受控JVM/真实两类4重编保持，真实目标与变异未执行。2735来源/归档/构建前、23产物与五成功命令同次匹配，target清理；峰值769858021bytes仅是构建护栏。旧CLI误选和name验证器KeyError原流保留；修正后复用成功门与all前缀，不称首次全绿。NoBody误称AST/SSA的旧文案、未覆盖的List参数primitive结果、临时return拼写成本和其余职责/资源门按[next-boundaries](results/s1-ordinary-parameterized-policy-v1/next-boundaries.md)承接；八个旧失败未重测或修复，2.9/2.10/S1不勾选，OpenSpec17/59，S2未进入。

## S1 NoBody 泛型策略迁移（2026-10-24，AA）

[NoBody 泛型策略迁移](results/s1-no-body-generic-policy-v1/README.md)、[收据](results/s1-no-body-generic-policy-v1/acceptance-r1.json)（2026-10-24）：Java 接管 NoBody 的层级源码准入、method-local 泛型完整声明和 slot/annotation/varargs/header 拼写，root 保留原 erasure 后预检位置、同次物理事实、CP 费用、调用绑定、停止和提交；没有新 DTO/read/cache。三生产文件与两现有测试增强，恢复 34→35，schema12/解析4/inputs v9 保持。纠正 ordinary NoBody 误称 AST/SSA 的说明，新旧均39字节，原 method-local 说明保持。有效六门：22 native 1023 passed/0 failed/2既有ignored；7上下文21报告、2实际预算stop与1零值输入拒绝保持；继承body12/130cutoff/name3，65 compact error完整JSON和32原子partial点通过；168全文/78声明保持。当前136×3对Z完整JSON除物理NoBody说明、实际engine tuple和usage.elapsed外相等；20上下文72次物理marker，其中16上下文48份顶层文本改变，声明/raw/费用/limits/来源/coverage保持。当前核心6编译12受控JVM、真实两类4重编通过，真实目标与变异未执行。2735来源/构建前/归档、23产物及五成功命令同次绑定，target清理；770145514bytes仅为构建磁盘护栏。首core driver全文断言exit1与合同preflight字段错误均保留，R2复用已成功原流离线验收，没有重复CLI/JDK或重跑native；不称首次全绿。后续构造器、family source policy、member descriptor与普通声明拼写及bridge/raw/资源退出门见后续清单；八项旧失败不在本次分母，2.9/2.10/S1仍未验收，OpenSpec17/59，S2未进入。

后继职责按 [next-boundaries](results/s1-no-body-generic-policy-v1/next-boundaries.md) 的真实消费者与事实/策略界面推进；已迁为薄适配的 generic 函数不重复计为 root 策略。

## S1 泛型构造器源码策略迁移（2026-10-24，AB）

[泛型构造器源码策略迁移](results/s1-generic-constructor-policy-v1/README.md)、[收据](results/s1-generic-constructor-policy-v1/acceptance-r1.json)（2026-10-24，AB）：Java接管class-scope构造器选择、完整准入、candidate/body-proof回退、InitRecord/参数槽/作用域/字段写入证明、raw字段唯一匹配及声明/proof分类，root保留同次facts、subject身份、erasure/CP/同类调用与委托物理门、停止和完整提交。两生产文件纯迁移，无新DTO/read/cache/test函数/产品资产，恢复35/schema12/解析4/inputs v9保持。有效六门通过：24选定native1034 passed/0 failed/2既有ignored；80上下文240报告与8非零预算边界全文、marker、声明、物理身份、费用/limits、来源/coverage保持；52上下文有成功构造器投影，28没有，RawNewHold旧Partial保留。NoBody24/body12/130cutoff/name3、65compact error与32原子partial点、168全文/78声明保持；当前136×3全文及408完整JSON除已验证engine与usage耗时外匹配，核心6编译12受控JVM/真实两类4重编通过，真实目标未执行。2735来源/归档/构建前、25产物及五成功命令同次匹配，target清理。六份旧GenericConstructor缓存报告与重新生成的JAR身份错配已保留原流，以现存冻结JAR补采AA旧6份，当前235前缀复用+5续跑，不改snapshot、不重建或重跑成功native/JDK，不称首次全绿。subject纯比较求值提前的实际CPU成本归2.10；剩余family/descriptor/普通声明、bridge/raw与资源门及八旧失败继续承接，2.9/2.10/S1未验收，OpenSpec17/59，S2未进入。

下一步按[后续职责清单](results/s1-generic-constructor-policy-v1/next-boundaries.md)迁移已选family的源码策略，保留root物理关系/候选/停止/事务；已迁移构造器不再计为root策略残留。


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
