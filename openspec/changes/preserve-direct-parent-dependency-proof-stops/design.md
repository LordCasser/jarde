# Design

## Context

动机与行为见 proposal 和 delta。本片基线为 c8718aeff6cc08b686c7b3c3495bfd0bb9efa2b1，facade SHA256 `ee8146c1907e1a24e3a631f1106dfad261046acd037fab45c2cde6ce04d16bac`；最终CLI2669…/59674128B绑定完整4902源，不能只凭单文件pin冒认产品。建项时该提交自身CI仍待，现已由CI38692921405和独立审计接受；该处记录规划基线；本片后续实施状态见文末，修复自身CI独立验收。

DT-29原JAR1253B、SHA256 `31ec2e5d6b4fdac4fc4c910904c3b84dc92f64d54e83a1a81b41e57b41dad5c3`，目标 `dt29/SamePackageParentFamily$B.set(ZZ)V`。旧958a和final2669各执行wide4096、cap1至8，共18命令，全部exit0。wide实际Header用量3，保留三条父字段cast和物理access$002；cap1/2用量1/2，却把父字段写入替成解释，仍Complete。

只观察的真实GDB证明：final2669 cap2一次Budget::exceeded经field proof；cap1两次依次经accessor、field proof，inferior均exit0。严格审计仅归一六个usage elapsed叶，保留全limits后，完整JSON与普通运行一致。栈未打印BCI/owner或原Error数字，不将三条拒绝说明算作三次预算拒绝；原report→None丢失为源码支持的归因，外层Error catch是否动态执行未由这些栈证明。DT-29的source是当前IR的B，关系可由IR提供，本轮只动态证明owner读取停止，外部source读取仍须独立回归。

宽预算完整root恢复文本已含nested A/B，原样javac23 --release8加未改Runner编译；Java8与Java23各运行原JAR/重编结果，四次输出均为原oracle `true:true:false:false`。standalone A/B报告仅用于物理核对，不重复作为编译单元。v2误将三份报告合编造成binary-name冲突，原始失败保留，四次runtime仅属于修正组装的v3；未改Java文本。此为正例基线，不是低预算修复通过。详细原始材料在本地cache，任务1.1须形成可移交清单。

## Goals / Non-Goals

**Goals:**

- 复用已有原Error/原报告两渠道，保留实际原因、全部诊断、analysis、物理方法和真实已获准用量。
- 在恢复前分流，沿用可靠前缀与付费发布边界，首个accessor停止阻断后续field工作。
- 保持正常字段和helper算法；补足实际外部source读取的公共控制，不把当前IR的快捷路径当该覆盖。

**Non-Goals:**

- 不修reference-overload、不扩大Java字段访问性或helper内联范围，不宣称method-only已有本片反例。
- 不改变Budget生命周期、全局merge优先级或新增预算维度、crate、依赖、pipeline/层级框架。
- 不改变parse、方言验证、runtime selection或verifier结论；生产引擎不执行目标代码，不增加GDB/JDK依赖。受控fixture的源码恢复、编译与运行是不同证据。

## Decisions

1. **原地接入最窄typed读取。** 既有 `DependencyProofFailure::{Error, Report}`、`selected_dependency_proof_header` 和 `stopped_before_recovery` 已真实存在。两个proof返回DependencyProofResult；其owner读取以及仅被两者使用的 `selected_direct_parent` 改用同一typed读取，保留唯一原predicate。caller通过现有proof closure与14项tuple交回停止，不复制report默认初始化。只删外层catch会漏report→None渠道，因此不足；不新增adapter或复制predicate。
2. **保持原承载数据。** 直接Error沿原映射保留数字；Report携带原execution和完整diagnostics，不解析文本、再poll/charge或根据当前usage猜数字。只传递真正结束共享请求的Budget/Cancelled/Elapsed；普通None与值级Unsupported/decode保持原隔离。所有必要source/owner读点须审查，reference-overload的legacy路径逐字保持。
3. **复用恢复前与发布边界。** 已完成analysis/callee/facts沿原所有权传递；停止不创建Region/AST/emitter、不重分析。沿用现有publication gate：sticky ledger或取消/期限阻止当前ResultItems准入时，仅移动原诊断到aggregate，保留较早成员，不为一个unpaid stopped record重试收费。token后续已置位不能取代原预算原因，全局strongest merge不改。
4. **验证到真实路径。** DT-29公共wide/cap1/cap2验证owner与两个消费者；另一个真实外部source fixture验证关系读取中断。复用现有最窄observer定位，token初始clear，在真实Charge中取消，或让实际checkpoint观察已过期ledger；不注入Error、预取消或sleep。记录停点账目与后继tail，只断言首次观察停止后无新增工作，不要求撤销已准许charge。两渠道分别核对实际覆盖；缺失/歧义/不适用控制不得误判Stop。
5. **复用已有能力和门禁。** 问题是仓库自有身份/预算/报告的局部交接，现有reader/JVM/Java类型足够；第三方error或层级库不能找回原诊断，且增加维护许可面，故无新依赖。永久测试不依赖GDB。适用focused/workspace/MSRV/Clippy/严格OpenSpec与当前workflow保持原门；源码对照使用原完整root、原Runner和受控JDK，不把片段或verifier成功当整族语义成功。

## Risks / Trade-offs

- [仅保护owner读点] → 外部source公共回归与全部typed调用点审阅；当前DT-29的IR快捷路径不冒覆盖。
- [后续poll改写预算为取消] → 原停止直接交接，复用publication gate，无停止后的收费/轮询。
- [普遍拒绝让低cap过关] → wide源码与行为对照、普通拒绝和来源控制同时保持。
- [报告有正文即被当Complete] → 分开核analysis/recovery/class执行平面，停止前prefix与未提交body分别断言。

## Migration Plan

无数据迁移。先固化可移交反例和正例，再接入永久回归、最小生产交接和真实in-flight控制；最终源码完成适用门禁与自身commit CI后同步归档。失败raw与所有脚本组装错误保留，不改原fixture、Runner、oracle或生成Java。当前计划不等于实施完成；当前c871 CI也不能作为未来修复自身CI。


## Implemented local boundary (2026-10-15)

实际生产修改仅六处：两个direct-parent proof返回既有DependencyProofResult；两个owner读取改用selected_dependency_proof_header；仅供二者调用的selected_direct_parent复用同一typed读取及原唯一predicate；原proof closure用?交接并删除吞Budget/Cancelled的两处catch。未改reference-overload、全局Budget/merge或publication gate，未新增依赖。

scoped observer复用既有cfg(test)观测：DT-29 accessor/field各覆盖真实收费取消、已过期ledger实际checkpoint和ClassHeaders原Report；外部C receiver B的关系证明实际尝试source B，原Report停止后不读owner A或进入field。实际成功fixed测试才支持这些后继断言；旧生产7个新增失败不支持未执行断言。取消初始token clear，观察真实hand-off全部15个counted维度、原Error/Report及全部诊断、无后继工作/收费tail；sticky ledger发布控制只保留已付费constructor，不强迫当前setter准入。公共plain Budget控制则明确要求停止setter record已合法准入。

外部C公共Partial原本已绿，只作兼容控制；source B首次读取顺序依据scoped callback观测，未推广为每种外部source/owner组合均有动态覆盖。缺失/歧义仍Complete的普通拒绝并保留物理BCI来源。局部最终457/0/1及真实完整root/原Runner双JVM实验分别记录；自身commit CI、同步归档与最终main绑定仍待，详见tasks。
