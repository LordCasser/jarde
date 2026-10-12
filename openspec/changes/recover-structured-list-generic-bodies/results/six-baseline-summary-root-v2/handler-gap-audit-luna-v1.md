# CF11 handler/loop gap audit（只读）

## 结论

这是独立于 list 泛型恢复的 handler 控制流债务。最早可确认的事实断裂在 `Walker::loop_region` 的循环测试/边界验收：本例 BCI 7 的循环头含 `Iterator.hasNext()`（BCI 8），它位于唯一命名异常表行 `[0,47) -> 50 RuntimeException` 中，因此 canonical CFG 将该块连到 catch handler。循环结构化路径会把它当作一个可抛异常的 leaving edge，但 `header_tested_loop` 仅接受 dead edge 或已证明的 finally edges，不检查当前保护它的 named catch；latch path 也只接受 dead edge。于是这个循环测试块不能被当前 `while` 证明接收。

另一个同一闭环的缺口是 `Frame::loop_body` 在进入循环体时无条件把 `own_try` 置为 `None`。因此即使允许循环头的 named-catch edge，循环体递归也丢失外层 try 所有权；普通块在 `region_at_inner` 的 `edges_accounted_by_catches` 路径无法把其 throw edge 交给这个 catch。对本样本，循环头 edge 是首先暴露的拒绝点；两处都应成为独立修复验收面。

这不是 raw CFG / 保护区解析漏行：`crates/jarde-jvm/src/cfg.rs:681-723` 按指令 BCI 与半开表范围构造 throw-site edge；`crates/jarde-java/src/normal_flow.rs:151-165` 有意将 exception edges 排除在 normal-flow projection 外。`guard::catches`（`crates/jarde-java/src/guard.rs:13558`）已经能从以当前块开头的命名行证明 catch 形状，`Walker::try_level`（`region.rs:6446-6458`）以 row join 为 protected-body boundary，`edges_accounted_by_catches`（`region.rs:8539-8564`）及 `exception_edge_accounted`（`region.rs:8596+`）已经提供逐块、逐 ordinal 的命名 catch edge 证明。问题是这些证明没有贯通到循环测试和循环体 frame。

## 源码依据 / 可复用机制

- Atlas 已按技能流程 `project(open)` 打开 Jarde，并在 `guard.rs`、`region.rs`、`normal_flow.rs`、`cfg.rs` 做限定 symbol 查询。Atlas 确认 `NormalFlowView::build` 把 `CanonicalEdgeKind::Exception` 计入 excluded 后跳过（`normal_flow.rs:151-165`）；`proved_loop_catch_joins` 是处理 handler rejoin 的独立证明（`region.rs:1902-2023`）。
- `proved_loop_catch_joins` 专门针对 handler normal successor **重接回保护该 handler 的循环内部**；它要求 join 在 natural-loop blocks 中（`region.rs:1924-1962`），并要求每个整条表范围都在 loop body 内且不含 loop header（`1980-1997`）。本例 handler 50 的唯一 normal successor 是 59（loop exit 后的 return），try range 又从 BCI 0 覆盖 foreach setup 与 loop header，故这不是该机制的适用子集。其用途是 reducibility/catch join accounting，不能单独解释这里的失败。
- `try_level` 对一条命名 row 建立 `protected_frame`，其 `own_try = Some(start)`（`region.rs:6451`、`Frame::protected` 的 `2491-2500`）。但 `Frame::loop_body` 重建 frame 时写死 `own_try: None`（`2351-2384`）；普通块的 catch edge 只在 `frame.own_try` 有值时才调用逐 edge 验收（`3359-3365`）。
- Header-tested fallback 在 `region.rs:12748-12760` 对 header leaving edge 检查 dead-edge 与三个 finally proof，然后直接 fallback；这里没有 `edges_accounted_by_catches`。Latch fallback 在 `14262-14267` 也直接拒绝非 dead leaving edge。复合 header proof `proved_header_test` 在 `13387-13390` 同样拒绝全部非 dead leaving edge。这些位置是循环层用自己的 gate 处理异常边，不是 report diagnostic 的反推。
- 不建议把所有 exception edge 泛化当成 loop 结构边，也不建议把 handler -> join 边加入正常循环图。最小架构方向是让已证明的 named-catch owner 随 protected loop 的测试/body frame 继续传递；在循环的测试块和循环体块复用 `edges_accounted_by_catches` 的精确 ordinal/range/handler 校验。循环的正常 CFG、latch/continue 和 handler 控制路径仍各自建模；catch body 由 `try_level` 的 clause walk 拥有。

## JADX 对照

- `jadx-core/src/test/java/jadx/tests/integration/trycatch/TestLoopInTryCatch2.java:16-33,44-49` 用真实 Java source 回归“try 包含 setup + while loop，loop 异常落到 loop 后的 catch”，断言反编译只出现一个 IOException catch。拓扑与此处接近。
- `jadx-core/src/test/java/jadx/tests/integration/trycatch/TestLoopInTryCatch.java:9-55` 和 `jadx-core/src/test/smali/trycatch/TestLoopInTryCatch.smali` 覆盖 loop 内 try/catch 以及 catch/loop branch 几种允许输出；其 handler 会 return，覆盖 catch 与 loop 的不同正常汇合形态。
- 访客事实：`jadx-core/src/main/java/jadx/core/dex/visitors/regions/ProcessTryCatchRegions.java:27-39` 先遍历含异常 handler 的 region；`:69-76` 遇到 `LoopRegion` 就递归进 body；`:81-105` 从 try dominator 按可达路径提取 try region，并通过 handler path 排除已在保护区之后的块。loop 本身由 `regions/maker/LoopRegionMaker.java:80-97` 建立。这是对照资料，不是可直接移植的 Jarde 证明策略。
- Jarde 自有近邻 fixture `tests/p3_loop_try_handler_entry.rs:53-108` 测的是 catch rejoin **循环内部**，与本债务（整个 loop 在 try 内，catch 从 loop 退出后汇合）不同；`tests/p3_nested_try.rs:120-137` 证明已有 named catch edge 可由 `edges_accounted_by_catches` 精确消费，但说明该测试只覆盖 protected block，未覆盖 loop context 传播。这两项应作为邻接回归而不是替代新正例。

## 独立 spec 建议与边界验收

建议新建独立 change，主题限定为“一个命名 catch 完整保护的普通循环可在外层 try/catch 下结构化；handler 在 loop exit 汇合”。正例固定使用 `handled(List<String> input, List<String> output)` 与 javac 8 class bytes、source + `javap` golden，要求 try/loop/catch/after-try 的所有块恰好各有一个 owner，不能靠取消 refusal 或吞 exception edge 达成。先支持一个命名 handler、一条完整保护行、handler 位于 loop natural set 外且有一个 normal successor 到 try join；多 handler、部分覆盖、多共享 handler 先保持拒绝。

边界矩阵至少应固定：

1. **throw / nonthrow 与异常时机**：分别注入 `iterator()`、`hasNext()`、`next()`、checkcast、`output.add()` 抛 RuntimeException；每种情形按执行 oracle 校验 catch 只增一次 `effects`，且 `output.add` 之前/之后的效果不被重排。空输入及无异常输入应证明 catch 不运行，循环 effects 仅按完成的 add 次数增加。
2. **range / loop exit**：正例的 `[0,47)` 从 foreach setup 开始，覆盖 loop header、body 与 latch；保护终点正好在 loop 的正常 exit transfer 前。分别改变 range 使其不含 header、提前越过 body/latch、或延伸到 after-try/catch handler，要求拒绝或按明确语义分区；不能把未保护的块写到 catch 内。
3. **latch**：BCI 44 的 goto 7 是普通 back edge 且 goto 不可抛；它必须保持一次迭代末端，不应被 named catch 当作 handler transfer，也不应被 loop-exit boundary 吞掉。
4. **handler entry / join**：BCI 50 是 catch-entry `astore`，handler normal path到59；catch 不得成为 loop natural predecessor/continue edge。验证 handler entry 的 exception-stack binding 保留，`effects++` catch statement只出现一次，try join 59 的 return 只出现一次。
5. **否定控制**：至少加一条 catch-all/finally、handler 有多个 normal successors、handler 在 loop 内 rejoin（现有 `p3_loop_try_handler_entry` 已测）、loop body 存在未被该 named row 覆盖的 throw site、两条 row 共享物理 handler、range crossing 等 refusal controls，避免把 named catch owner扩展成无条件异常放行。

## 执行状态

本次只读；没有改项目文件、构建、执行 fixture 或跑目标。root 给出的合法 fixture 原类、javap 与 all-report 用作目标描述和最终交叉核对；上述根因依据来自实际 bytecode 控制流事实与当前源码 gate/frame 传播，不从 `jre_region_uncovered_blocks` diagnostic 推断。最终是否仍重现，应由 root 在同一目标诊断中验证。
