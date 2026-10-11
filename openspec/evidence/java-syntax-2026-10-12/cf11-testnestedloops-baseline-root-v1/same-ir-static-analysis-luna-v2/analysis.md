# TestNestedLoops foreach 首拒诊断准备（静态分析 v2）

## 冻结 baseline 已证明什么

`execution.json` 的 SHA-256 与 root 提供的 `5635e60b…95435` 一致。JDK 23 原始目标 class 为 `cf11.UpstreamNestedLoops.class`，SHA-256 `8ece89e4…d8231`。Jarde 四个 profile 都生成了该方法的 structured source；现有源文本仍是两个 `while`，保留 `l2.add`、`l1.remove` 和循环后的 `size`/`clear`。同一文本还带有 `ordinary_generic_source_unproved` 的 method Signature refusal marker。该 marker 是 class-source Signature 投影观察，不能证明 foreach 候选因泛型而拒绝；现有 JSON 没有保存 builder 收到的每个 loop AST，也没有候选 refusal reason。

Javap 将外层 loop branch 定位在 BCI 13（`hasNext` invoke BCI 8），内层 branch 在 BCI 42（invoke BCI 37）。外层 `next`/checkcast/store 在 17/22/25，内层对应位置在 47/52/55；循环体调用位于 `equals` 61、`length` 69、`add` 79、`remove` 91，循环后 `size` 104、`clear` 113。源码输出在外层 loop 前单独声明 `s1`，loop body 第一项写成 `s1 = (String) local3.next()`；内层 `s2` 则是 loop body 内带初始化声明。由此可以提出一个**待验证假设**：外层可能在 iterable candidate 的首道 body-shape gate 被拒，因为该 gate 期待首条是带值声明；内层表面上满足该 gate，但它可能在任意后续候选条件失败，也可能根本未到该候选调用点。输出文本不是 builder AST trace，故不能据此宣告真实首拒。

## 候选调用顺序和可观察范围

`build.rs:18879-18890` 在处理一个 loop 时先暂存当前 statements，再递归构造其 body，最后恢复 statements。因此嵌套 loop 的 body 会先完成；`build.rs:18982-18991` 在构造当前 `kind` 后才调用 `iterable_for_each_candidate`，且只有 `for_header.is_none()` 且 `tests.len() == 1` 才进入该分支。若两层都满足该分支，递归构造决定 inner 尝试早于 outer 尝试。准确诊断应记录 eligibility gate 和每次尝试，不应只在函数内打印失败：这样既能区分“未调用”与“调用后拒绝”，也不会把调用顺序当作仅由文本推断的事实。

当前 `iterable_for_each_candidate` 的返回型是 `Result<Option<...>, StopReason>`（21224-21229）。成功返回 `Some`，所有正常形状不符都收敛成 `None`；budget/cancel 才是 `Err`。内部包含一个函数外层的快速 gate（21230-21248）以及随后大量 `Option` 闭包内的 `return None`（21274-21642），所以现有接口不能报告首个失败谓词。

## 建议的最小临时 same-IR 观测

在 `build.rs` 内加入仅测试编译、显式环境变量启用的局部诊断，不增加公开 API 或持久数据模型：

1. 在 `loop_region` 的候选调用分支（18982-18991）记录一次 `candidate_attempt` 事件：物理 loop branch/test BCI、`path`、`loop_headers`、`form`、`tests.len()`、`for_header.is_some()`、当前 loop `kind` 和 body 首 statement 的紧凑结构（variant/name/value variant/origin BCI）。在 `iterable_for_each_candidate` 顶部再记录给定 `test_bci` 和首个 gate 的输入。对跳过候选分支也记录 `candidate_skipped` 及 `for_header/tests` 条件。
2. 保持全部候选谓词和执行顺序不变，为候选函数的正常拒绝出口赋稳定短标签。外层快速 gate 拆成依次检查的命名 gate；Option 闭包中的每个候选级 `return None` 改由局部 `refuse!("gate-name")` 记录首个拒绝后立即返回 `None`。不要替换 `find_map` 等内部搜索闭包的局部 `return None`；那表示“本项不是 iterator init 候选”，不是整个 foreach candidate 的拒绝。需要区分 `Ok(None, first_gate)`、`Ok(Some)`、`Err(StopReason)`，不能把 Stop 当 shape refusal。
3. 记录每次结束事件与对应 BCI，包含 `projected`/`refused`/`stopped` 和首拒 gate。建议只打印首 statement 与小型字段，避免整棵 `StmtKind`/SSA 大 dump；如需类型门槛诊断，仅在对应该 gate 时记录 iterable local 的 `presented` type、physical iterator owner/descriptor、iterator slot/phi 概况。所有额外信息应只解释现有谓词，不改变判断结果。

对这份 class，静态上预期容易辨认的首个事件位置是 outer branch BCI 13 与 inner branch BCI 42；最终 trace 应以 `test_bci` 原始值记录，不用假设 AST header BCI 等同这些值。outer 首 statement 是 assignment 的解释应由诊断记录的 `StmtKind` 确认。inner 是否调用、首拒是什么、generic presented type 是否影响 owner matcher，均待 trace。

## 驱动真实 Program 的最小路径

同 IR 的合适驱动不是只在 `region.rs` 调 `recover`：该函数只生成 `Recovered.regions`（`region.rs:1438-1453`），旧的 nested-loop same-IR 示例也止于 CFG/SSA 与 region recovery（私有候选 `region.rs.candidate:15912-15997`），没有构造 AST `Program`。

可在 `crates/jarde-java/src/report.rs` 的测试模块增加临时测试：以冻结 JDK23 `.class` bytes 开 `ArtifactSnapshot`，照 `region.rs:22093-22168` 构造 `analyze_method_ir(..., AnalysisStage::ALL)`，选择物理方法 `test(Ljava/util/List;Ljava/util/List;)V`；用 `RecoveryFacts::new(MethodFacts::new("test", "(Ljava/util/List;Ljava/util/List;)V", 2).with_access_flags(0x0001).with_declaring_class(...))`，按需补 debug locals，然后构造 `RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)` 并调用 `report::recover`。Facts/Request 的既有用法见 `report.rs:15381-15418`，字段构造 API 在 `facts.rs:82-120,1096-1120`，request API 在 `report.rs:8447-8472`。

这一路径会经过真实 recovery pipeline：`report.rs:11156-11172` 用 request 的同一 `MethodIr` 恢复 regions；`11379-11427` 把相同 `canonical`, `ssa`, `operations`, `code` 和这些 regions 交给 `build::build`。候选调用就在这次 Program construction 之中。因此将 test 绑定至 frozen class、方法身份和 Java 8 baseline profile，再启用上述局部事件，能观测实际 Program builder 内 outer/inner 的候选拒绝，而不从渲染文本二次重建 AST。

如要最大程度对应 class-source baseline，优先让测试保留 classfile 的 debug-local facts（`RecoveryFacts::with_debug_locals`，facts.rs:1110-1120）；但核心前提是确实用原始同一个 `MethodIr` 走 `build::build`。直接 `report::recover` 不会重现外层 generic Signature 投影 sidecar/marker；这是可接受的，因为本诊断只问 foreach candidate 的执行路径与首拒。不得据此把 loop refusal 与 `ordinary_generic_source_unproved` 建立因果。

## 候选门槛与范围边界

函数先要求循环条件是 `Iterator.hasNext()Z`、循环体第一条是带值声明（21230-21248）；之后要求首声明由同一 iterator 的 `next()` 或单层 cast 产生（21274-21323）。它再找前置 `iterator()` 初始化和 iterator 声明，限制初始化到 loop 间的 statements（21324-21376），核对 interface owner/source type 与 descriptor（21391-21433），再验证 init/store 唯一消费、header phi、loads、next-result 单一消费及 exception-handler 区间（21435-21642）。最终将 body 原样交给 `ForEach`，只替换 next 值和删除 iterator locals（21644-21715）。所以诊断从方法首拒足以定位当前拒绝，不要顺手扩展为整帧/全方法 value trace。

循环体的 `add` 和 `remove` 是普通 body effects；候选实现保留 `body.to_vec()`（21655-21662），没有搬移它们。`size`/`clear` 位于两个 loop 结构之外，也不参与 candidate 的 loop body 首条 gate。现有 source marker `ordinary_generic_source_unproved` 来自 Signature source projection，baseline 已在四个 profile 都记录；它和 iterator owner、SSA phi 或 body-head shape 是不同证据域。本准备只定位 loop gate，不提出产品修复。

## 待实测项

尚未看到 same-IR candidate trace；outer 的“首条 assignment”目前是从输出顺序与构造代码推断，inner 的失败位置完全未定。还没有证明两层都到达 candidate、哪条谓词先拒绝、source `presented` type 的精确值、或拒绝与 generic Signature 投影有无因果关系。root 后续如运行诊断，应将原始 stderr、runner 输入 bytes 和临时 candidate source hash 一并冻结；本文件不是内部 trace，也不是新验证结果。
