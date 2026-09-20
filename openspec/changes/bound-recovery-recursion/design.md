## Context

动机与复现见 [proposal](proposal.md)。基线事实：同一输入在默认与放大预算下都以栈溢出 signal 结束，说明承载递归的调用深度与预算无关；恢复层的失败词汇里没有任何分支表示「递归无法完成」，因此这次失败连一个 `Partial` 都没机会发布。

恢复层已经有界的地方和没有界的地方必须分开看：

- `crates/jarde-java/src/build.rs:73` 的 `const MAX_VALUE_DEPTH: usize = 24;` 在 `render_value`（`build.rs:1543`）与 `deferred_producers`（`build.rs:2203`）入口检查；`region.rs` 的主体 walk 用 `visited` 集合与迭代循环推进（`region.rs:892` 起），单一区域不会无限前行。
- 但递归边由多族组成：`region_at` 按结构嵌套递归（`region.rs:1134`、`1135`、`1464`、`1601`、`1728`）；`collect_guards`/`collect_paths` 递归整棵 region 树（`build.rs:306`、`405`）；`Builder::region` 按 region 递归（`build.rs:600`）；`emit::expr` 按已提交的 `Expr` 树递归（`emit.rs:393`）。`render_value` 内部还有若干递归边传递 `depth`、另有若干边直接传 `0`（例如调用接收者与参数，`build.rs:1802`、`1807`），所以现有 `MAX_VALUE_DEPTH` 是「表达式嵌套」的界，**不是**整条 native 栈的界。
- `crates/jarde-java/src/stop.rs` 的 `StopReason::{IrTableMissing, Budget, Cancelled, Interrupted}` 是既有停止词汇；`Interrupted { code, at }` 在 `report.rs` 中发布为 `RecoveryOutcome::Stopped`、`ExecutionReport::Partial` 与一条带该 code 的 Error 诊断。既有停止语义规定：停止不提交文本或段表，`content = not_produced`。

上面的清单是候选与观察，不是结论：哪一个族真正吃掉 native 栈必须由本次复现的 native 栈证据决定。

## Goals / Non-Goals

**Goals:**

- 公开恢复入口对任何输入都返回报告；进程 MUST NOT 因恢复请求以 signal 结束。
- 界的检查发生在进入递归之前；界外结束为已发布停止，位置与原因可定位，复用既有词汇。
- 以受控、内存生成的 fixture 把「修正前 signal、修正后报告」变成可重放回归，并用变异证明这条回归真的在保护界。

**Non-Goals:**

- 不预设根因（候选族见 Context）；不把「加一个更大的常量」当修正。
- 不做 region/SSA/AST 的通用迭代化重构；不新增 crate、依赖、`Stacker` 式增长栈库、更大栈的工作线程或第二个执行上下文。
- 不新增 `StopReason` 变体、预算维度、报告平面；不改变既有停止/取消/预算语义与文本契约。
- 不重开 R8/R9 或已归档的停止传播修正；不修 body 解码重新解析类的债务；不做性能工作。

## Decisions

### 1. 先在 native 栈上定位，再决定界加在哪一族

实施 MUST 在固定基线上对受控输入取得 native 栈（`lldb --batch -o run -o bt` 或等价方式），记录重复帧、承载递归的函数与所在 crate，作为本 change 的证据；并逐一核对 Context 列出的候选族中每一处的现有界与递归边传递。不得只按方法名、输入大小或「哪个 crate 看起来可疑」选择落点：`computeMaxStack` 与 `stripFront`/`stripBack` 是不同的方法形态，能同时解释两者的只有证据。

定位结果决定受影响文件与 crate。若重复帧落在 `jarde-jvm`/`jarde-reader`，界加在该处并在验证中记录跨层影响；不把「恢复层之外的溢出」静默改写成恢复层内部的修补。

### 2. 界的表达：进入前的显式深度检查，不是捕获溢出

Rust 进程的栈溢出由运行时在 guard page 上终止（本复现的 `fatal runtime error: stack overflow, aborting`），既不能作为可捕获错误，也没有可查询的「即将溢出」信号；因此唯一诚实的做法是在**进入**下一次递归前检查一个显式界，并把超界变成普通结果路径。

两个候选机制的取舍：

- **显式深度界（选定）**：每族一个 crate-private 常量（与 `MAX_VALUE_DEPTH` 同风格），或由定位结果决定的一个共享界检查 helper；在递归入口检查，超界立即返回已发布停止。代价是「深但合法」的输入会被拒绝；收益是改动局部、可在进入前判定、与既有 budget/停止语义直接组合，不改变任何已发布结构。
- **显式工作栈（不选）**：把递归改写成堆分配帧的迭代循环，能继续处理深输入，但要求重写被定位的 walk、其计费点与取消检查，并把「深度」换成「堆用量」；这与 non-goal 里的「不做通用迭代化重构」冲突，而且本缺陷的证据是 abort 而非「界内输入被错误拒绝」，没有支撑它的反例。

界的数值 MUST 由证据给出：受控复现达到的深度或循环重入点、既有已提交语料实测的最大深度、以及所选常量的余量，全部记录在 verification。MUST NOT 把 `MAX_VALUE_DEPTH` 直接推广成全局界；若定位证明 `render_value` 的部分递归边重置了 `depth`（`build.rs:1802`、`1807` 是观察到的形态），修正 MUST 是让这些边也受限，而不是放大常量。

### 3. 超界发布既有停止，停止契约不变

超界在递归入口返回 `Err(StopReason::Interrupted { code, at })`（`at` 为当前 BCI，`code` 是本 change 新增的稳定字符串，例如 `jre_recursion_bound`），其余由既有路径处理：报告 `outcome = Stopped`、`execution = Partial`、一条 Error 诊断（code 与位置）、`content = not_produced`、text 与段表为空、usage 为本次真实值。CLI 以既有「执行未完整」状态（exit 4）退出，JSON 与库报告同源。

明确解释一处措辞：本 change 说的「可靠前缀」是既有停止语义里的真实 usage、诊断与停止位置，**不是**半构建的文本。既有停止契约规定停止不提交产物；要发布部分文本需要改写 `content/text/source_map` 的停止契约，本 change 不做，也不以「有可靠前缀」为名把它偷偷加上。

对「可以局部化的递归是否应在该 region 上降级而不是停止整个运行」：本 change 选择停止。理由是「引擎无法完成该递归」时，停止是唯一不需要额外证明的诚实答案；把降级限制在一个 region 需要先证明该递归的错误只影响该区域的产物，那是更强的证据要求，留给后续独立变更。该取舍记录在 Risks。

### 4. 受控 fixture：内存生成、形状由定位证据决定

fixture MUST 由测试内的具名生成器构造（不提交第三方字节、不调用编译器），形状 MUST 镜像定位到的递归驱动：

- 若定位落在 region 树/嵌套结构族，生成器构造同一深度的嵌套结构（例如同形的 if/loop 链）；
- 若落在 `render_value` 的某条重置 `depth` 的递归边，生成器构造同形的嵌套调用/算术链；
- 若落在某个循环重入，生成器构造同一重入形态。

忠实性判据（写进生成器注释与测试断言）：生成器必须点名它镜像的复现事实（同一递归族与同一驱动形态），并且修正前的同一字节在该测试进程之外以 signal 结束；不得为了撞到界而堆叠与驱动无关的指令——那样的 fixture 只能证明「界存在」，不能证明修的是本次缺陷。

回归分两层：库入口（`Engine::recover_method` 得到 `Stopped`、非 Complete、诊断点名界）与进程入口（`crates/jarde-cli/tests/task_cli.rs` 的既有进程模式运行 `jarde-cli recover`，断言 exit 4 与 JSON 报告）。signal 是进程级事实，进程层断言不可省略。

### 5. 复用与依赖

库层面没有需要新引入的能力：`Budget`、`StopReason`、serde 与既有 CLI 渲染足以表达该停止。平台机制评估（crate 依赖、`std::thread::Builder::stack_size`、信号处理）在本问题上都只把 abort 推迟或转移到别处，不满足「不能完成的递归必须发布停止」，因此不采用；不新增依赖，也不改变离线/不执行目标代码的边界。`javac`/JDK 只出现在既有测试侧的受控对照流程里，本 change 不需要它。

## Risks / Trade-offs

- **界过小，拒绝合法输入** → 以既有 P3 语料与受控正对照（界内输入逐字段不变）证伪；界的数值由实测深度给出，不用一个猜的常量。
- **界的位置选错，abort 只是换了入口** → 以 native 栈重复帧与变异证伪；变异移除界后必须复现 signal。
- **停止的代价** → 超界时整个方法不发布产物，即使其余区域本可恢复。这是本 change 的明确取舍（见决策 3）；本地化降级需要更强证据，另行独立立项。
- **fixture 只撞界不撞缺陷** → 忠实性判据与「修正前 signal」的前置证据防止这种空洞回归；生成器注释必须点名镜像的事实。
- **停止被误读成新质量平面** → 复用既有 `Interrupted`/`Partial`/诊断，不新增字段；验证中核对停止报告与既有预算停止逐字段同形（除 code/位置）。

## Migration Plan

1. 在固定基线上复现并定位（native 栈证据 + 候选族核对 + 界依据）。
2. 实现界与停止发布，补齐被证明绕过界的递归边；随后加入受控 fixture、两层回归与变异。
3. 跑固定提交的门禁（fmt/clippy/workspace test、既有 P3 与 CLI 目标测试、OpenSpec strict），在 verification 记录复现、栈、界、变异与门禁结果，再同步本 change 的 delta 与状态引用。

回退按本 change 的独立提交进行；回退后必须恢复「该输入使进程 abort、停止语义未覆盖」的公开事实，不能保留完成声明。

与 [`fix-nested-arithmetic-value`](../fix-nested-arithmetic-value/proposal.md) 都改 `crates/jarde-java/src/build.rs`：两者串行实施，先后顺序不改变任一方的验收，但不允许在同一个工作树里并行改动同一文件。

## Open Questions

无。定位结果可能把受影响文件从候选清单上移开（包括移到 `jarde-jvm`/`jarde-reader`），这属于决策 1 的执行范围，不构成需要上游裁决的架构歧义；只有「必须重写跨层递归结构才能满足停止契约」才升级处理。
