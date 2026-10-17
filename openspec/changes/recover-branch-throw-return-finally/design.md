## Context

动机见proposal。已封EX.class为1632B，SHA256 e6f0eb1e029a2c883d3ee5df5e089eda17fea27833a9244297cede139cd61287；finThrow Code38B、唯一catch-all[0,17)→27。0/1分支至4的new/dup/RuntimeException构造/athrow13，或14的String常量/astore16；正常cleanup17/20/22后load25/areturn26；异常store27、cleanup28/31/33、load36/athrow37。memo-v3仅物理事实，当前fallback不证明真实首拒；必须实际probe既有副本proof、资源准入、Region和Builder。

root已读prove_finally_copy的正常后缀/异常链、cleanup_sequence及finally_body的精确visited检查，既有代码允许保护体throw并支持If。故不预设需要新证书，更不先放宽所有catch-all；先定位第一次丢失完整证明的消费边界。

## Goals / Non-Goals

**Goals:** 独立完整类与原EX目标方法的单分支throw/单saved String return恢复，真实异常身份、清理次数与顺序、完整物理来源及停止可证。参数、成员名或String字面值不是准入标识。

**Non-Goals:** 不恢复finally自身return/throw、不吸纳nestedFin/exception-only拓扑，不重写CFG/SSA事实或忽略有throw-site的保护体，不新增公共AST/Region/enum、依赖或通用图等价器。父4/7和71/612不因叶片局部通过关闭。

## Decisions

1. **先证实缺口。** 用同次analyze_method_ir的Code、canonical、SSA、effects、Operations和真实ExceptionHandlerFact，实际记录既有prove_finally_copy返回及资源/Region首拒。只有证明缺失时才添加私有有界检查；若证明已存在，就仅补消费接缝，不另建平行机制。候选分流在付费扫描前排除无关形，维持原P5计费断言。
2. **正常与异常闭合分别核。** 正常String producer→save→load→return唯一链、唯一保护行、每个throw-site的row归属、全部正常边与owned块、两份cleanup的Operation/本copy SSA producer/consumer对应都必须成立。new RuntimeException及athrow是真实受保护抛出；handler入口caught对象→store→load→athrow须保持同一身份。cleanup在半开区间外且不递归进入本handler；清理抛错替换pending return/异常，不能用不可达handler证明替代。
3. **沿既有Finally/SavedReturn和受限walk消费。** 原If、Throw和保存点留在try内，cleanup只输出一份；normal copy、handler和return machinery由原完整证书认领。body/handler/continuation无缺失或重复owner；子失败、未声明local、fallback或Stop不发布部分Plan/Region/正文。新增遍历与分配沿已有poll/charge，停止原样传播且不退款。只在实际证据要求时补最小私有metadata，不扩大对外表示。
4. **完整对照而非局部片段。** 双真实javac8和23编译独立完整类并核实际Code/row对应；公库/default-all及CLI报告要求Java/Structured、完整方法和物理BCI来源。report.text原样重编，以相同driver在仅含rebuilt目标的classpath下用Java8/23运行；保留正常/负数、cleanup正常/抛RuntimeException或Error/空receiver的值、异常type/message、顺序/次数及可观测身份。内部新建primary无法在调用前动态拿到引用，其身份由SSA证明，不能虚报driver覆盖。JADX固定版本原样比较，真实失败记no-runtime。原EX完整产物保留所有其它红方法，不删方法或修文本凑绿。
5. **边界与维护。** 复用仓库已有Rust/Runtime和本地JDK/JADX，不引入许可或维护负担。真实class变体覆盖cleanup参数/receiver/row中界、错保存值/重抛对象、外来前驱和其它出口；收费后的晚预算/取消/期限必须未提交而保留原原因。仅匹配诊断、opcode列表或字节数不能代替上述证明。

## Risks / Trade-offs

- [资源头把异常构造误作资源、或body分支重新触发同一finally] → probe同次调用入口与Frame继承/精确scope，修具体接缝，不跳过通用守卫。
- [finally抛错被本handler再捕获执行两次] → physical半开范围、effects与handler集合静态核验，driver注入同一个cleanup异常并核identity/一次调用。
- [独立小类通过而EX被宣称全绿] → 两者分别记录；原EX finReturn/nestedFin继续保留真实红项，父最终全量门继续待办。

## Migration Plan

先落红基线/真实首拒，再派发Luna私有实现；root审读后应用并独占工具链。fmt/当前CI Clippy/必要focused/P5/reader/双seedworkspace、完整原/JADX/Jarde矩阵及确切产品自身CI依次验收，提交推送并核clean main/target absent后才关闭本片任务。每次Rust遵守5GiB余量、target1GiB、1秒进程组守卫、默认stack及finally exact cargo clean；旧临时脚本丢失可从durable证据恢复，不重跑已接受基线冒作新结果。

## Root actual first-refusal finding (2026-10-17)

root 在 e575 上实际运行两轮受守卫保护的探针，保存在 `results/baseline-root-v1/capture.tar.gz`。既有 `prove_finally_copy` 对双编译器独立类和封存 EX 均返回完整证书：normal cleanup `[17,25)`、handler cleanup `[28,36)`、saved return `16→26`、primary binding/reload/rethrow `27/36/37`，覆盖全部20个物理 BCI。直接调用 `resources(entry0)` 已返回 structured Finally，完整 owned 集为 `0,4,14,27`。

实际 Region 跟踪未进入 `finally_body`：block0 仅有条件测试，没有 leaving exception edge；提前的 shared candidate 只处理原有 Load-slot0 cleanup，未认领本形。进入 throw arm4 后才调用 generic guard，此时其正确的 `start4 > protected.start0` 检查拒绝。normal cleanup/return 又融合在 block14，不能新增位于 owned body 内部的 cleanup block boundary；该私有候选已在产品测试前撤回。

最小修复仅让单 catch-all、row 起点正是当前 block、terminal 是范围内 comparison 且恰有两个 normal successor 的入口，在遍历子分支前调用原 `guard::examine`。原 profile/frame/unvisited、copy/value/range/ownership 和 bounded child walk 守卫仍为准入依据。不新增 proof、公开表示、cleanup boundary 或 Builder 机制。仍须实际验证无关 P5 费用及失败/停止时不发布部分形状。

## Existing boundary promoted by the same proof

第一轮 fresh workspace 在 p3_precise_rethrow 的旧负例处停止：`PreciseRethrowBoundaryProbe.anyFinally(I)I` 与本形结构相同，既有 SavedReturn 已证明 primitive 返回，但旧测试仍要求 unsupported。原源码为 try 内条件 throw 或 return17、finally调用log。新输出保留同一 try/finally 与完整类其它方法，未将 catch-all 伪造成用户 catch。root只把该旧预期提升为 Structured/no-fallback 的 finally 正例，保留 named catch、multiCatch未证明参数和 changedValue 的原有断言；不按类名或返回类型封堵正确恢复。失败 raw 单独保留，修订后重新执行完整双 seed，不拼接失败运行。
