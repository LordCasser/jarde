# Design

## Context

动机见 `proposal.md`。完整阅读了既有 java8-recovery 主规格及 scenarios、analysis-contracts、conservative-output、原 snapshot widening delta 与既有 Stop debt。现行 content 允许解释性产物，错误是实际执行停止没有到达执行平面，不能把所有 explanation_only 当缺陷。

基线是 c814 产品的冻结 CLI（59602720 bytes，SHA256 `cdf06694fafcb4e8cef5a4b9452676489e8cf02495c0254ce38a21e44ee4c7d5`），同一 H1 JAR（4469 bytes，SHA256 `1ee63f3836a5e514cea57b7349dd515ca3f98550154beff187fdca4d019105b0`）。六次普通 CLI 仅变 class_headers：wide limit4096 消费16、Complete/contains_statements；cap1/2 正常 Partial/Refused、exit4；cap8/14 仍 Complete/exit0、main explanation_only；cap15 仍 Complete/exit0，main 与 wide 的文本及去 elapsed 报告相同。cap8-v3 bt30 前四次真实 header 拒绝经 snapshot proof，第五次经 double-brace；cap15 单次拒绝只有 double-brace，无 snapshot 栈。两 trace 全 JSON 去 elapsed 后等于各自普通 CLI 对照。没有改返回、注入假 Error；旧 debugger v1 因 `run` 重置参数而 EOF、零 quota 命中，排除在缺陷证据外。trace 未打印每次 BCI/owner 或 Result，不能把四次命中称四个唯一站点或四个 catch 执行。

真实 c814 源码路径（完整快照与范围见 `results/source-context/manifest.json`）：

| 阶段 | 实际路径与损失点 |
|---|---|
| 前置 | class-source → recover_prepared_member → recovery_from_with_class_candidates:42127–42129 → snapshot_hierarchy_widenings_presented:26643–26654 → prove_snapshot_hierarchy_widenings → selected_reference_header:25424–25456 |
| 共用读取 | resolve_class_source_dependency_read_raw:21597–21631 将 resolution.execution 合到 proof 局部 execution，非 Complete 时返回 Ok(None)，不再携带原报告诊断 |
| 后置 | class_source_with_evidence:2097 → project_class_source_double_brace:4716 → prove_double_brace_allocation:4815；4851–4863 的局部 execution 在 let-else Ok(None) 被丢弃，2097–2115 的既有 Err 聚合因而不可见 |

前置 wrapper 的 Err→空 Vec 是另一静态渠道；本轮 trace 证明上述公共消费路径发生拒绝，不独立证明该 catch 被执行。后置函数 4992–5006 的 child 非 Complete→Ok(None) 是本函数交接中需核定的静态边界，不能冒称另条动态反例。H1.main 的9条 invokespecial 都是 Methodref 构造器，没有 InterfaceMethodref invokespecial、putfield 或 access$；不属于接口 super 或 direct-parent field/accessor 的动态证据。

本变更不改变 classfile parse、方言验证、运行时环境选择或 verifier 结论；不把既有 JVM 可运行性、正文存在、quality 或 source-map 数量当作证明完成。

## Goals / Non-Goals

**Goals:**

- 在两条已证消费路径保留原停止的两种承载渠道：原 Error，及 resolver/child 已发布的非 Complete 中承载预算/取消/期限共享请求停止的 execution 与 diagnostics；先停止工作再发布状态。
- 按实际交付边界保留内容：前置没有恢复产物，后置已有正文仍可靠；方法与类执行平面分别诚实表达。
- 将真实 H1 输入转为永久公共回归，校验来源、第一停止和停止后的收费边界，普通 None 仍可表达拒绝。

**Non-Goals:**

- 不修 reference-overload、direct-parent accessor/field 三个未动态证明 wrapper；它们继续留在既有静态债务，不以共用 helper 调整偷偷扩大本片功能承诺。
- 不新增全局 sticky Budget、预算维度、crate、依赖、通用 pipeline 或层级服务；不变更宿主协议/分层。
- 不重开 emitter、class-view、CLI 最终文档 output_bytes 债务或当前四工程闭包；不把本轮反例当作修复通过。

## Decisions

1. **保留原停止的最窄交接。** 不让依赖读取以一个裸 Option 抹掉非 Complete。只对本片消费者增加足够传递原 execution、diagnostics 与 proof 值的局部结果，或以局部状态输出并在 None/返回 Vec 前显式检查；其他调用者维持原契约。以现有 ends_the_request（facade:41329附近）/error_ends_the_request 的预算或取消分类确定是否结束共享请求，ElapsedMillis 仍是其预算维度；原 Err 保持原 Error，由现有映射生成执行/诊断；resolver 已吸收 Error 时传递它的原报告，而非再造 Error。非 Complete 不是充分条件：value-level Unsupported/decode 停止按既有隔离/汇总处理，不能一律终止兄弟工作。不要从 diagnostic message 解析数值、重 charge/poll 猜原因或编造 limit/consumed/requested。单改外层 `?` 既漏掉 Ok(None) 通道，也可能将合法公共请求变成无分析/恢复报告的裸错误，因此不采用。共享读取自身可作最小辅助改动，但不得引入新的全局运行模型。

2. **前置复用最小 stopped 初始化。** 在 snapshot proof 发现原停止时、构造 Region/AST/emitter 前分流。沿用本次 analysis 所有权、物理方法、callee 与现有返回 tuple，不重新读 Header/Body，不重跑分析。通过 `preserve-interface-super-proof-stops` 拟引入的最小 stopped-before-recovery 构造或等价入口复用既有 report::stopped 初始化，保留 pending evidence/not_produced/unbound artifact；规划时前片尚未实施；本轮真实基线cc305已提供最小入口stopped_before_recovery(request, reason, original_execution, Vec<Diagnostic>)，两渠道与本地813项/完整源已验，最终产品CI仍待组合源码。本片直接复用该真实签名，不再复制RecoveryReport初始化。不得接受 Complete 来伪构造 stopped。counted 原预算维度保留；DependencyDepth、ElapsedMillis 等非 counted 不能塞进 OutputBytes/IrItems，必要时沿用既有 Interrupted 词汇但 execution/diagnostics 必须来自原停止且保留实际维度。Cancelled 不改写为预算。

3. **后置只汇总，不撤销已交付正文。** double-brace 只在本次证明成功时提交它自己的改写；停止前已提交的 main/兄弟方法及其 raw/text/source-map 留在报告里。原停止直接合并到 class execution 并保存原诊断；不把 main 的 Complete 随意涂成 Partial，不调用前置 not_produced factory 删除正文。依赖读取或 child 报告非 Complete 且承载预算/取消/期限共享请求停止时，也在普通 None 分流之前交回该停止。预算停止为 class Partial 原维度，取消为 Cancelled；仅有合法普通拒绝不改变 Complete。

4. **第一停止结束后续工作。** 前置停止后不进入余下正文/optional evidence/类投影；后置停止后不进入 anonymous-interface 等后继投影。c814 的2118–2157 无 Complete gate，不能只依赖2160/2170之后的 gate；在实际停止边界返回可靠报告或明确跳过全部后继收费。保留停止前已发生的收费与 coverage，不退费、不补造范围。报告发布若受 result-item/output 限额约束，按既有可靠前缀契约保留 aggregate/coverage；取消下 stopped member 未必能被 push，不能以强迫产生该 record 换取假成功。测试须看实际阶段/usage或已有观察点，而不仅看末态标签。

5. **完整拒绝与执行停止独立。** snapshot 算法固定深度界、循环终止、缺失/歧义/不可达关系，以及 double-brace 形状不适用，仍是普通保守拒绝；不把所有 None 变 stop。只有原 Error 或非 Complete report 确实承载 request-ending 预算/取消/期限时，才按本片结束共享请求；普通 value-level Unsupported/decode 停止保持原隔离/汇总契约。原始 source/效果/来源控制保持，不扩大 widening 或 double-brace 接受范围。先前已观察到的停止通过停止后不再工作或重 poll 保留；不修改全局 merge_execution 的 strongest-priority 策略（Cancelled 高于 Budget），不增加全局 first-wins。

6. **复用评估与维护边界。** 本仓库已有 Error、ExecutionReport、diagnostics、物理身份与停止报告构造，缺的是局部结果交接。现有 reader/JVM/Java 类型足以完成；第三方 error/层级/管线库无法恢复本请求被丢掉的原诊断，维护与许可成本不合理。保持 Cargo manifests、lock、版本/features 与许可证清单不变，不实现新 parser/graph 服务。生产不启动 GDB/JDK/目标代码、不下载依赖；GDB 只用于已经保存的历史归因材料。

7. **永久验收与最终产品绑定。** 先将 H1 frozen JAR 接入真实 Engine::class_source_with_evidence/相应公共入口与 JSON CLI 测试；cap8 前置与 cap15 后置分别有宽预算/低cap控制，断言非零测试实际执行。数字是当前输入的已观察边界；若最窄接线减少停止后收费，应使用语义边界和原拒绝点断言，而不是强制继续花到历史usage。永久测试不调用 GDB、不注入假 Error。取消使用初始 clear token，在真实 proof Charge 中确定性置 token并在真实检查点观察；期限使用现有受控时钟/observer或最小定位 seam，证明实际期限已过，不能用预取消、sleep碰运气或手工构造 Error 冒公共回归。直接 Err 与原非 Complete report 渠道均需覆盖，且后者核原诊断未遗失。新回归之外，复用原 snapshot/double-brace 合法与拒绝来源控制和必要完整生成源、正确 Runner 的编译执行门，不以单方法片段冒整类。按最终 fix 基线的实际 workflow/项目要求运行适用门禁并取得自身产品 commit CI，不新增无关门、不以 c814 的局部/矩阵证据或其他提交 CI 替代。

## Risks / Trade-offs

- [共享读取有许多消费者] → 限定两个调用链的局部 handoff；审阅所有改动调用点，记录其他三 wrapper 未动态覆盖且不承诺本片修复。
- [原 Error 已被 resolver 吸收] → 保留原 report/diagnostics；仅字段真实存在时声明其值，不为接口一致编造 Error 数字。
- [停止后仍执行或仅修改末态] → 记录第一停止与之后的阶段/费用，重点检查 anonymous-interface 后继，禁止事后涂标签。
- [后置停止误删正文] → cap15 main/class 文本与原 wide 一致的对照以及逐方法结果/source-map保留断言；class aggregate 单独检查停止。
- [取消/期限只发生在入口] → 真实 Charge/checkpoint 证据，初始 token/期限控制；当前历史材料未覆盖这些情况，实现任务不得提前勾选。
- [停止与普通缺证混同] → 宽预算合法、缺失/歧义/不适用/固定界拒绝必须同时通过；不以普遍拒绝实现停止契约。

## Migration Plan

无数据迁移。实施先固定最终源码基线和历史输入、确认前片最小 API 可用状态；添加永久公共回归，局部接线两个消费路径与报告汇总，验证停止/普通拒绝/原始来源；完成最终源码适用门禁与新产品自身 CI。局部接线失败时撤回本片修改并保留失败 raw/source pins，不改变 Budget 生命周期或历史证据。本地实现与适用focus/六档CLI/完整H1源码已执行验收，产品自身workspace/MSRV/CI尚待，任务4.2/4.3保持开放。


## Implementation boundary review

实际交接复用cc305的boxed `DependencyProofFailure::{Error, Report}`，checked reference参数站点只对本片调用链保留真实Err；共享legacy adapter与三个邻居wrapper逐字保持。snapshot在恢复前分流并复用最小工厂；double-brace遇原报告或Error后保留已提交方法而直接结束后继投影。member publication先检查原stops的取消/期限，以及已有local token/ledger stop，再决定能否合法付费发布；不能发布时仅移动原诊断，避免重poll把原预算改报Cancelled或丢诊断。既有全局merge优先级没有改变。

五个真实in-flight控制及六档公共入口已通过；本地无ledger的Elapsed分支仅作静态保留原诊断的修正，不冒称单独动态覆盖。ordinary控制必须破坏读取的中间Mid；终点Greet靠header中直接命名即能证明，删/重复它不构成应拒绝的前提。全部历史错误实验预期与修正raw保存，不将其记为产品缺陷或通过。

本地441/0/1、同CLI六档及H1完整11源编译/原8行双JVM对照的范围和pins见results/root-local-v1；最终CI完成前不将产品集成任务勾选。
