## Context

动机见proposal。真实typed首次两轮2/1失败保存在recover-proved-local-source-types/results/focused-root-v1/v2，缺口[82,92,105]为append调用后pop。root核同次字节码，读现有discarded_evaluations/call_result_is_discarded/quoted_bcis/call_statement；Atlas仅在build.rs范围确认pops_of当前两处调用都是拒绝路径，不能据此宣称全仓无其他消费者。

既有discarded_evaluations只准同SSA block相邻producer、准确写入ValueId、唯一use为pop、无local write、opcode0x57，并优先独立调用丢弃而非后继静态qualifier。accounted跳过pop；fallback quote通过pops_of携带pop，成功普通call_statement却只用Origin::direct(call)。

## Goals / Non-Goals

**Goals:** 在成功普通调用语句消费现有discards对应，将真实pop挂为derived到语句完整跨度；可独立关闭，不依赖char局部恢复。

**Non-Goals:** 不改discard计划、SSA或JVM类型，不泛化pop2、任意copy/phi、跨block或有别的reader。不扩大到short-circuit/accessor/任意Stmt类别的统一来源框架；若另有同类消费者遗漏另记债务。局部类型与条件switch单独推进。

## Decisions

1. **消费现成一对一证据。** 两条普通调用语句创建路径均在call_expr成功后，只读取discards[at]，存在则在原direct(at)上plus_derived(pop)。不使用包含qualifier的pops_of无差别扩散；qualifier已有expression来源保持。两处用同一个小型现有Builder内部来源消费helper避免漂移；不新增证书或扫描。
2. **完整语句承担丢弃。** 将derived挂Stmt而非参数/receiver/调用名子表达式，因为Java调用语句的分号表明结果丢弃。原primary和表达式来源不变，打印正文恒同。
3. **预算沿原输出入口。** 有pop时对该证据读/追加计一步并poll准确pop BCI，Stop在push前传播；push/emitter仍对来源输出计费，永久真实IR内部测试核准确Stop以及公开原子发布。
4. **独立实际验收。** Luna私有最小draft，root审读真实输入及边界、先接受基线，再应用。使用不需要类型片的冻结完整class；原/JADX/Jarde双JDK8/23原样重编运行，default/all全物理BCI、准确owner/name/descriptor/完整语句span；只对应pop新增来源，正文及旧来源恒同。
5. **已有能力足够。** 标准库BTreeMap、OriginSet和Budget提供全部信息，不增依赖/许可/维护实体。parse/verify成功不等于恢复或运行等价，分别记录。

## Risks / Trade-offs

- [把已跳过pop误附到无关call] → 仅复用准确discards key，不猜相邻opcode或callee需求，真实额外reader/pop2反例。
- [新来源导致局部跨度或改变原输出] → 完整Stmt字节范围含缩进/分号/末尾换行，原primary/所有旧来源精确比较。
- [忽略预算或在Stop前部分发布] → 精确pop收费/poll，真实同次IR内部budget与公开Stop/cancel，无部分text/map。
- [扩大类型片范围] → typed生产已完整保存并恢复HEAD，先完成本片精确自身CI和clean交付再重新应用typed。

## Migration Plan

真实失败/架构证据→独立最小spec→私有patch/真实class测试→root守卫验证及冻结CLI→完整对照→精确产品CI→账本/handoff/clean交付。历史raw和typed失败不覆盖，不考虑错误行为兼容。
