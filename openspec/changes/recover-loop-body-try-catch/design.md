## Context

同次事实见 [instrumentation.md](instrumentation.md)。有限循环的自然环遗漏正常流终止的 throw arm，现有 catch join 证明因此拒绝；foreach 的 catch join 已通过，另卡在 try 外 `aload; astore` 的引用呈现；无限 retry 的普通图无环，通用 catch 行走又进入自己的 try，最终所有权检查正确拒绝重复块。三个落点都在现有 `jarde-java` 区域和正文恢复中。

当前基线是产品 ca45c0e。固定类基线56命令/112raw，固定同次 IR 四 lane/16 physical methods；retry 两变体基线88命令/176raw、同次 IR 八 lane/18raw 都由 root 实际独立接受。它们仅证明缺口，不证明实现通过。JADX 有限 loopCatch 实测会让异常逃出；retry 实测正确，采用其 try/body/handler 的归属思路，但不搬入 synthetic splitter 图改写。现有 Endless/Try/LoopBreak/LoopContinue 已可表达目标语法；不需要依赖、公共类型或第二 CFG。

## Goals / Non-Goals

**Goals:**

在完整物理表行、原 CFG、SSA、来源和停止约束下恢复 spec 的三种有限循环及 JADX 无限重试/break 代表。区域只能拥有一次物理块；保护范围外的效果不能进入 try；原来可逃逸的异常仍可逃逸。恢复、编译和运行是三个分别验证的结论。

**Non-Goals:**

本片不扩大 fragmented 认证、catch-all/finally/multi-catch 的支持边界，不重写自然循环或删除 handler 普通边。v2 观察文本导致的 nested StringBuilder 构造参数拒绝独立记录。DF.loopFinally 当前另有 handler51 uncovered，但生成方法已有 return，遗漏内容为注释；它不等于 javac 错误。原始 debug javac8/23 错误均只报 loopCatch 缺返回（生成 DF.java:46）。finally 差距独立记录，完整类仍必须实际编译运行；不能删除方法或修补生成源码。

## Decisions

### 1. 有限 catch 的证明同时提供 join 与 terminal body 所有权

扩展 `region::proved_loop_catch_joins` 的现有返回事实：原 pair 集继续用于 `irreducible_blocks`；新增的有界 owner 集按 header/handler/row 关联，供同一个 Walker 使用。私有记录只存关联和新增块，不复制 CFG、自然环、完整表或 SSA。原来可接受的普通 catch 及 fragmented pair 路径保留。

新增许可仅在一个具名物理行、完整 canonical 映射、exception-only handler、单普通后继 join、join 属于该环且不是 header 时产生。声明保护跨度和实际 throw sites 都逐项核对。每个与表行相交的块属于原自然循环体或下面证明的 terminal body，且受 header 支配；循环测试不能进入保护范围，已证明的 update BCI 也不能进入。对跨块范围按真实 instruction BCI 判断，不能把整个 fused block 都当作 protected。

terminal body 从行内、非 header 的普通 body comparison 分支进入：每个新增入口与路径节点的普通前驱封闭在该 source/链中，不能借另一个入口、join 或 handler 进入。沿普通边取得有限、无循环的 terminal arm，末条真实 decode/SSA 指令必须是 Throw，无普通后继。允许多个分别闭合且互不重叠的 arm；本片不把 Return、未解码死端或任意环外块视为同样证据。每个 arm 完整落在该保护行内，物理 jsr path 与 canonical 映射一致。

普通路径检查的终点为 join 或这份证据中的 throw leaf；任何绕开二者的循环、死端、额外出口都拒绝。这里证明 Java 循环体的所有权，不证明每个可能异常一定被 catch 类型匹配。new/constructor 仍可能抛出未匹配的 Error，完整 exception dispatch、row 顺序和逃逸路径原样保留；不能把保守 Exception edge 当作运行时类型证明。

Walker 只把匹配本 header 的 terminal owners 纳入 body frame scope 与 expected coverage。`loop_body_sequence` 的普通循环推进仍以原 natural blocks 为依据，throw arm 从既有 try/branch 递归进入。原 `loop_terminal_returns` 保持 Return 的契约；throw 不混入其集合。header-tested/test-chain/latch-tested 的 scope、coverage 必须消费同一事实，不能只解除不可归约诊断而漏掉 throw。

`header_tested_loop` 当前调用 `prove_for_header`，不能假设 for_header 为 None。归纳变量扫描将已认证 terminal owners 视为 body：读像普通 body 读一样处理，写 induction slot 或 invariant 仍拒绝；preheader/update 真正环外的规则不变。显式检查证明的 update 不落入表行。DV.plainFor 的 row `[10,32)`、update39 是当前锚点，BCI 不硬编码进产品。

备选是只给 handler→join 一个豁免，或全局扩张 NaturalLoop。前者留下 scope/coverage 缺口，后者让所有循环分析共享未证明的成员；采用现有 catch 证明中的局部 owner 扩展。

### 2. foreach 复用已有引用 load 的呈现规则

在 `build::presented_reference_store_value` 给同 block 的引用 Load 增加明确分支。优先复用 `presented_array_operand` 已有的 Ref 类型、实际 local read、单次消费、同 block、producer 在 consumer 前、非 quoted region 检查；store 前区间只能包含这次 load 的指令，不能跨效果、改写或分支。

Load 是在原 store 位置呈现的局部读取，可以位于 try 外。仅此分支不要求 protected try path；constructor/static Invoke 的现有 protected-arm 限制保持。不要把整个 reference gate 的 try 条件删除。

`all_reads_reach_presented_writes` 的全读 SSA/phi 检查、`cross_exception_store_type_is_proven` 和声明 common-owner/catch-parameter 作用域检查继续执行。每个 read 必须到达这个 reuse variable 的实际已呈现写；Entry/Caught/缺失定义/PhiInput::Itself/另一未呈现写继续拒绝。声明可提升，但赋值、load、效果和来源仍在原位置；不能因“引用类型可命名”就猜测初始化。

这补的是已有表达式证明，没有新的 SSA pass 或特殊 foreach holder。

### 3. 无限 retry 是既有 Walker 的局部异常循环形

在普通 named-try 分派前，以同次 `guard::catches` 的结果核验 retry；它仍遵守 guarded ownership，不绕过 finally/resource reader。候选不命中时沿用原行走。目标使用现有 `Region::Loop(Endless)` 包含 `Region::Try`，try 正常退出为 LoopBreak，handler 末端回跳为 LoopContinue，方法尾部在 Loop 后继续。

候选必须满足：单具名行/单 handler，无 inner/transparent/竞争行；try 从 header 开始（非空 lead 暂不获许可）；handler 无普通前驱，参数来自现有 CatchSite；protected 子图完整，正常完成只有一个物理出口；handler 的有限普通路径最终只有一条显式 Transfer 回同 header，不能进入尾部、共享 body 或有第二出口。第一实现覆盖现有直线 handler 代表，分支 handler 需另有完整证据，不能借当前许可猜测。

正常 exit 必须由表 end BCI 的真实 Transfer、该 SSA/canonical block 的唯一普通 successor 与 `Catches.join` 三者互证。`Region::Try.normal_exit_bci=None` 只是尚未附加出口来源，不等于 `Catches.join=None`。当前 v3 普通边 `0→55,29→0,55→63/73,63→73`，表 end26 是 goto55；`join_after` 按现有代码应得55，这是代码与物理事实导出的判断，候选测试必须显式断言实际 shape 值。join 缺失不能自动当作重试入口。

完成边/范围证明后才创建局部 loop frame：scope 为 protected+handler 精确 owner 集，break target 为 exit，continue target/header 为 try 起点。protected walk 以 exit 为边界；catch walk 单独以 header 为边界，禁止再读 header。在唯一 handler goto、完整 catch coverage 和无 sibling tails 都确认后，用既有 LoopContinue 保留真实回跳 BCI。正常 goto 对应 LoopBreak。可以在现有 try seam 传入私有已证明的 retry 上下文，避免复制 try_level；不能挪用 switch/fragmented flags 表示 retry。

现有 Endless 的 `Region::blocks` 在 body 已拥有 header 时不再重复计 header。完整树的 `overlapping_owner`、未覆盖指令、效果及异常边检查仍执行。不能关闭检查来让样例通过。catch body 自己的异常不受该表行保护，必须继续逃逸；原五个 oracle 的 uncaught AssertionError 是必测项。

所有识别先证明再认领，child build 不完整就恢复 visited 等实际修改的局部状态。StopReason 原样向上传播，不能转为 None/false。候选循环来源 derived(header)，break/continue 和 try 出口关联各用真实 transfer BCI，保留当前 source map 的物理方法身份。

### 4. 计费与验收遵循现有闭环

表、块、边、SSA uses/phi 的新增遍历逐项计 AnalysisSteps/已有对应维度并 poll；集合大小受本次 view 与 Budget 约束，递归保持现有深度界。没有默认 stack 扩大。停止测试必须实际落在新增证明内部，并证明没有发布部分 owner/正文。

Luna 分别准备 region 所有权与 build 引用 Load 的私有候选，root 审阅后顺序应用、构建和验收。完整 DV/DF 与 retry v2/v3 都保留；v3 是观察干扰控制，不替换 v2。产品每次改变后重建 CLI，再做双 JDK/双 metadata profile 的完整类对照。DF 完整类 gate 只由未改写源码的实际编译与行为证明，不能用仅 loopCatch 的方法片段当作完成，也不能从 finally 的质量诊断推断编译失败。

验收保持原值、异常、调用次数和事件顺序；另测 crossing-header/update、ordinary handler predecessor、shared/crossed row、额外出口、共享 terminal owner、reference 多消费/间隔效果/错误 phi、handler 非 header 回跳等反例。无 try、已完整支持的 finally 与 fragmented fixtures 保留既有文本；当前未完整支持的 DF.loopFinally 单独保留失败与后续修复证据。

## Risks / Trade-offs

- 自然环许可与 Walker scope 分离导致遗漏 → 同一次证据供 join/scope/coverage/induction 使用，最终 exactly-once 检查保留。
- terminal may-throw 被误当作必匹配 catch → 保留完整派发与逃逸，不靠静态异常类型推断改边；整类 runtime 验证错误输入。
- 引用 load 放宽牵连 effectful producer → 只在 Load 分支复用现有窄证明，constructor/call 门不变。
- retry catch 重入或吞掉方法尾部 → 独立 header/exit 边界、唯一 Transfer、完整 owner 与原五 oracle 核验。
- DF finally 的质量诊断被误当作编译错误 → 保留原方法和诊断，检查实际 javac stderr 与 runtime；质量差距独立登记，不扩大 catch proof。

## Migration Plan

源码候选在 main 上由 root 串行应用，无新分支/worktree。先 focused fixture 和停止/来源门，再 fresh 双 seed workspace、fmt、当前 CI Clippy、global strict、精确 CLI 重建；每次构建守5GiB可用、target1GiB、1秒进程组监控、default Rust stack、finally exact cargo clean。推送确切产品，接受自身 CI，最后以真实 HEAD 审计干净主线。观察和候选均不增加 CF-18/71/612 完成数。
