## Context

动机见proposal。2026-10-18当前产品红基线、真实canonical/SSA与首拒已核，见instrumentation.md和results索引。三类拒绝不同，不能按相同local1文本修改声明门。既有Region已区分带目标身份的LoopBreak、LoopContinue、SwitchBreak，Builder能生成循环标签，Return和Switch AST已足够。

## Goals / Non-Goals

**Goals:** 在原有有界Region walk中补齐switch来源的返回叶片、闭合非局部完成时的join选择与消费；只认领可证明且恰好一次的块。case内return先行，带标签break随后，并覆盖SL已有mixed锚，不遗漏其循环尾效果。

**Non-Goals:** 不删除或放宽declarations fallback守卫，不改canonical/SSA事实，不扩大异常处理证明，不新增公共AST/Region或pass、通用路径等价solver、依赖。无新代码提取的必要；参考JADX已有测试及出口分类算法，真实错码不照搬。此leaf不关闭CF13或71/612整单元。

## Decisions

1. **复用loop_terminal_returns。** 将已解码switch的直接返回目标纳入既有返回叶证明来源；保持单入口、精确正常边、外层scope、无真实异常边/其它出口、terminal decoded Return等要求，新增switch来源另核clone path及decoded targets与真实successors相符。comparison原两successor入口保持；switch可有多个case，不能只接纳两个successor而遗漏SL四分派。只拓宽来源dispatch种类，不把无后继块一律当return，不把method return当loop break。返回叶集合继续由原Frame scope、coverage与Builder共用，避免第二份返回证书。switch join若需识别已证返回，可在原私有LoopTarget携带同一返回叶集合，经原loop_targets继承，不另建Frame平行证书或重复证明全图；新metadata的所有权、scope和计量由root审查。
2. **在既有switch_loop_join接缝证明局部join与完成。** 保持已通过形的原先路径优先。只在真实loop frame下，为无postdom的非局部完成和postdom恰是精确loop break目标的缺口，分类各arm到共同局部join、精确continue、精确loop break或已证terminal return的路径。沿现有有界poll/charge遍历，不改变visited。无candidate/shared-tail=false本身不能当证明；无局部join也须完整验证每条路径。跨case、嵌套循环/dispatch、外来入口、未配对异常边、未知出口或多重候选不能猜测归属。优先扩充原私有返回结果语义，不建立平行solver；任何确需更大表示的发现先回到root审查。
3. **保留join与continue的不同身份。** mixed的局部join68和continue71分别进入已有switch-arm frame；case2直接goto71是continue，case1/default经68执行尾部后到71是正常流。带标签break形的44→58由原LoopBreak和Builder标签处理，58属于外层续接，不吸收到switch arm；没有局部join时只接纳已经闭合的路径。已证Closed形即使join与postdom同时为None，也须使用已有switch-arm frame传递精确continue，不能退回普通arm而丢失该证书。root的完整BCI差集实际发现SN两返回方法遗漏goto57/60；应由原LoopContinue呈现及保留来源，不另增switch专用latch机制。return44/58由原返回语句呈现。case边界、claimed块集合、共同尾与loop expected coverage须完全一致，不以忽略缺口或重复块完成覆盖。
4. **原子失败和资源计量。** 证明在mutating arm walk前完成，失败保持原fallback；后续失败/停止沿原checkpoint与原因返回，不发布半棵树或正文/map。候选分流复用已读事实，新增真实工作仍按现有预算收费；不得用修改旧P5常量、放松预算或预取消测试消除失败。
5. **完整源码与原JVM是验收依据。** 新CLI对八个原始完整class的default/all输出原样由javac8/23重编，双JVM rebuilt-only classpath执行；新增独立driver覆盖空数组、正常累积、case-return、break/continue混合及多次迭代后break，LD `{9,1,5}`必须为10，SL `{1,2,4,3}`为124。driver不改目标类，核CodeSource与原/重建隔离。SM五控制及其它非目标方法逐字保持原基线，完整物理方法身份/全部语义BCI/UTF-8 source map均须检查。不能用SL旧main未走case3冒充return覆盖。JADX原样编译/运行及失败继续保存。

## Risks / Trade-offs

- [把loop break目标当switch局部join使循环提前或单次结束] → 目标身份分类，LD10及多前缀driver，原/JADX/Jarde三方保存真实差异。
- [case-return排除自然环后被空arm吞掉] → 同一返回叶证书扩大scope和expected coverage，永久真实class测试钉住case body和全部BCI。
- [mixed共享尾重复发射或continue多执行尾] → join68与continue71分开，精确claimed/continuation及外部driver trace；真实错边/重入邻变体必须拒绝。
- [局部恢复绕开完整拒绝或吞Stop] → 不改build.rs门，晚预算/取消/期限及无部分正文/map的公库门验证。
- [为新形重复扫描使无关计费漂移] → 已有路径优先、使用已读loop/operations事实，先核focused/P5，再两fresh workspace；真实新增工作与历史预期冲突单独审查。

## Migration Plan

root先接受同次IR首拒，再派发Luna私有最小补丁和真实输入测试；root审查后串行应用和构建。先case-return，再break/mixed，期间保留整类红项，不能删方法凑绿。最终完整三方矩阵、公库default/all/CLI来源与停止、SM零回退、fmt/CI同款Clippy/MSRV、双fresh seed和精确产品自身CI逐项通过后，才关闭相应任务并提交干净主线。Rust使用5GiB余量、target1GiB/每秒进程组守卫、默认栈及finally exact cargo clean。
