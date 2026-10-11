# CF11 NestedLoops2：外层 indexed-for 首个候选门槛（只读）

## 已观察事实

- Root 的独立收集器/验收记录 `/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/checker-root-v1.json` 为 `verified_observation_capture`（29 条命令）。JDK 8/23 原始类、JADX default/rename-none 源码和 Jarde default/all 都完成编译与运行；Jarde 输出在语义上与原始 9 组输入一致。这个结果只说明本基线观察到 Jarde 将外层输出成 `while`，不说明 `for` 证明具体在哪个分支失败。
- 原类两个 JDK 的 `test(List<String>)Z` 都有相同物理 CFG：初始化 `i` 在 BCI 2–3；条件在 BCI 4–11，包含 `invokeinterface List.size()I`@6；正文从 BCI 14 开始；`iinc 3,1`@41 与回跳 `goto 4`@44 位于尾块；循环出口 BCI 47。证据见原始 `javap` 行（JDK8 `raw/3.stdout.raw`、JDK23 `raw/6.stdout.raw`）及其 `execution.json` 命令记录。
- JADX 两 profile 都保留 `for (int i = 0; i < list.size(); i++)`，内层仍是 `while (j < s.length())`。Jarde default 文件 `/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/jarde/javac23-jarde-default/UpstreamNestedLoops2.java` 将外层写为 `i=0; while (i < list.size()) { ...; i=i+1; }`。

## 当前源码能推出的第一候选门槛

- `Walker::loop_region` 会在通用循环路径尝试 `header_tested_loop`（`crates/jarde-java/src/region.rs:11541-11549`）。单条件循环不满足 `header_test_chain` 的至少两个测试条件（`:13313-13315`），于是 `header_tested_loop` 的单测试分支会调用 `prove_for_header`（`:12783`）。这是按当前形状推导的调用路径；缺少同次 trace，不能声称基线确实走到了这行。
- 若到达 `for_header_candidate`，最早可从静态证据指出的拒绝候选是条件指令扫描：`:13668-13695` 允许 induction/invariant `Load`、`Push`、算术和比较，其他 operation 立即拒绝。原始条件 BCI 6 是 `Invoke(List.size)`，所以按指令顺序扫描会在该处拒绝候选。`test_is_pure` 自己允许调用作为条件表达式生产者（`:9586-9594`），形成两条规则的直接不一致。这里是源代码和真实 BCI 能共同推出的候选首拒；仍应由实际 IR trace 确认此版本中 `prove_for_header` 被调用、该块的 `SsaInstruction` 列表含 BCI 6，以及未在之前的门槛退出。
- 其他 `for_header_candidate` 门槛包括单回边、更新指令尾形、回边入边、唯一 preheader/初始化 store、SSA phi 两输入/use 数、循环内 induction/invariant 写检查（`:13442-13665`, `:13713-13755`）。单凭 javap/源码无法确定这些在 canonical/SSA 表中的真实事实；若 BCI 6 前存在更早失败，需以 trace 为准。

## 架构对照与建议

- JADX 1.5.6 的 `LoopRegionMaker` 先根据 exit/header 构建一般 `LoopRegion`；indexed-for 是之后 `LoopRegionVisitor.checkForIndexedLoop` 的窄识别（`jadx-core/.../LoopRegionVisitor.java:84-143`）。它要求尾部增量结果进入唯一两输入 phi、条件使用 phi、init 有唯一赋值、loop 内使用/赋值约束成立，然后只隐藏 init/increment 并标成 `ForLoop`。`LoopRegionMaker` 本身不证明 indexed-for。
- Jarde 已有更明确的 `ForHeader` SSA/CFG 证明和 `build.rs` 投影（`build.rs:18845-18925`）；无需新增识别器。最小方向是先保留当前初始化、phi、更新和循环不变性证明，再让条件表达式可包含 `test_is_pure` 已证明、且仍留在循环头部求值的 value-producing operation（如 BCI 6 的调用）。不把调用移出条件，也不降低 induction/update 的所有权证明。先用同次 IR trace 证实首拒后，再改该窄白名单。
- 外层输入 signature 的 `ordinary_generic_source_unproved` 是另一个 source projection/fallback 观察。它不能解释上述 `ForHeader` 条件白名单，也不能据此把 generic projection 和 loop-form 缺口合并。

## Atlas 限制

Atlas 项目已打开，但 full structural cache 对当前 checkout 陈旧（symbol 行号/签名与 live source 不符，且提示跳过 lazy extraction）。因此本审查没有把 Atlas 结果作为 gate 或调用路径证据；以上行号均以当前文件和实际 class/Javap/raw 为准，也没有启动 full index/sync。

## 未验证项

缺少本次方法的 canonical/SSA `ForHeader` 逐门槛 trace。尤其不能把 BCI 6 的静态候选拒绝写成已经运行观测到的 gate；本报告不声称外层 `for` 已被接受或拒绝于某个实际分支。
