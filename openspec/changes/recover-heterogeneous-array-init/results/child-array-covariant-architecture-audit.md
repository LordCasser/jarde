# 子数组协变初始化器架构审计

## 判断

`ownGridDirect` 的准确 child-store reader 与 parent `aastore` 身份可由现有 reader/identity 事实闭合；当前首先截断 parent chain 的是 `prove_array_initializer` 中要求父 component 与 child 实际数组类型完全相等的 gate。`Base[][]` 的 component 是 `Base[]`，子候选是 `DerivedA[]` / `DerivedB[]`，所以该候选在结构事实提交前被拒绝；这不是 Builder 已对一个已呈现元素作出的不可赋值判决。随后普通 new census 仍只能看到构造实例的 reader 是未认领的子数组 store，因而拒绝 Site。这里要分别记录 reader/ownership 闭合与 Java assignability 呈现，不能把前者等同于后者。

最小方向是保持数组结构 proof 与 Java 元素赋值呈现分层：child 及 parent store 的唯一 reader、身份、顺序和闭区间事实使用现有结构 proof；`Builder` 呈现元素时继续复用 `initializer_reference_widens`，以准确 parent `aastore` BCI 为锚。`Number[][]` 中的 `Integer[]` / `Long[]` 可由现有平台关系闭表证明；`Base[][]` 中的用户类数组需要该 store 的精确 snapshot hierarchy widening proof。类型事实缺失或不匹配时，Builder 必须拒绝 initializer 表达式并让完整正文 fallback；结构/ownership facts 可以继续作为真实 Site 来源保留，但不得据此发布部分正文或宣称 Java 类型已证明。不要另造 array subtype 表，也不要把 snapshot type table 透传进结构层作为唯一修法。

## 证据与拒绝边界

direct-v3 六类输入的源文件是 `evidence/heterogeneous-array-initializers-v3-pathfix/direct/{Base,DerivedA,DerivedB,LocalInterface,Mid,Main}.java`；`Main.java` 同时给出 `numberGridDirect()` 与 `ownGridDirect()`。对应的 javac 8 原始 listing 为 `evidence/heterogeneous-array-initializers-v3-pathfix/direct/javac8/logs/direct_javac8_javap-main.stdout`，`ownGridDirect` 的物理序列位于该 listing 行 482–512 附近：外层 `Base[][]` 在 BCI 1 分配；第一个 `DerivedA[]` 在 BCI 7 分配、内部 `DerivedA` 在 BCI 12 构造、内部 store 为 BCI 23、外层 store 为 BCI 24；第二个 `DerivedB[]` 在 BCI 28 分配、内部 `DerivedB` 在 BCI 33 构造、内部 store 为 BCI 44、外层 store 为 BCI 45，最终在 BCI 46 返回。两子数组都在父数组仍留于栈上的同一连续块内直接构造，没有逃逸到局部变量，也没有乱序元素生产。

保存的 Jarde JSON 观察记录为 `results/candidate-v1-fixture-v3/logs/fixture_direct_javac8_render-Main.stdout`（javac 23 对应同目录 `fixture_direct_javac23_render-Main.stdout`）。这不是当前代码新运行的结果，而是已冻结的候选/基线报告。`ownGridDirect()[[LBase;` 的方法报告将 `new DerivedA` BCI 12 和 `new DerivedB` BCI 33 列为未呈现 Site，理由分别是它们的实例只被指令 23、44 读取，而这些 store 当时被 Builder 引用为 quoted。文本同时说 BCI 7、28 的子数组值没有已认领 reader。此记录证明的是 child-array 与 construction Site 的 ownership 链未提交；它没有给出 `DerivedA[]` 对 `Base[]` 的 Builder 赋值判决。

当前结构代码的关键点：

- `ArrayInitializers::prove_with_composition` 在每个 block 上倒序收集候选，再从可以独立提交的 consumer 向 child 链提交。consumer 是写入本轮新建数组的 `aastore` 时，child 明确不能独立提交；只有 parent chain 完整后，所有 arrays 与其 `pending_sites` 才一起认领。见 `crates/jarde-java/src/build.rs:12735-12805` 及 `crates/jarde-java/src/build.rs:12558-12567`。
- `array_initializer_reader` 允许最后一个 child store 的 retained array value 被紧邻的唯一 parent store 读取；它也允许单一后续 reader，但要求 reader 的其它操作数构成该 interval 的完整表达式。ownGrid 两个 child 的 outer `aastore` 紧接在内层 `aastore` 后，因此这里的 reader 结构可闭合。见 `crates/jarde-java/src/build.rs:13700-13756`，以及 `prove_array_initializer` 调用处 `crates/jarde-java/src/build.rs:13444-13462`。
- 候选 child 通过 final ValueId 与消费 store 身份关联；随后 `child_type` 由子数组的 `NewArray` element/rank 得出。但当前 gate 是 `component != child_type` 即拒绝，见 `crates/jarde-java/src/build.rs:13215-13251`。这把“元素类型是否可赋值”误当成“child 是否闭合”的等值条件；它早于 `ExprKind::NewArray` 的实际呈现。
- parent 失败后，child 候选的 consumer 仍是父 `aastore`，`store_writes_into_a_constructed_array` 会将它留在待 parent 提交状态；没有 parent chain 时 child 与随附 Sites 不会被认领，见 `crates/jarde-java/src/build.rs:12775-12805`。这一点维持了失败路径的原子性，修正不得改变。
- Builder 对真正呈现出来的 initializer 元素调用 `array_initializer_element`，引用赋值检查复用 `initializer_reference_widens` 并使用精确 `store_bci`，见 `crates/jarde-java/src/build.rs:25353-25372`、`crates/jarde-java/src/build.rs:26685-26748`、`crates/jarde-java/src/build.rs:29446-29471`。此 helper 接受闭合数组形状、平台数组关系、既有 scalar widening 行及 BCI/source/target 完整匹配的 `ProvedSnapshotHierarchyWidening`。等秩数组会剥掉 `[]` 后，只用相同的已知 leaf relation 检查；不会推断任意用户类层级。
- 类型 fact 当前不在 `ChildArrayFacts` 内。该视图只读取已提交/当前 block 的 candidate array maps（`crates/jarde-java/src/build.rs:12569-12613`）；这是结构/ownership 视图，不应被当成 Java assignability 结论。Array initializer proof 使用的 `ArrayCompositionContext` 也不需要仅为 child 的类型表判定而扩充（`crates/jarde-java/src/init.rs:181-188`）。同一请求的 Builder 已能取得 `snapshot_hierarchy_widenings`（`crates/jarde-java/src/report.rs:130-132`、`crates/jarde-java/src/report.rs:9653`），可由它在精确 store BCI 上调用现有 `initializer_reference_widens`。因此最小方向是复用准确结构 reader/identity facts，再由 Builder 决定完整 initializer 是否可呈现；未知/拒绝的类型关系必须触发完整正文 fallback。无需将 snapshot type facts 透传到结构层作为唯一方案，也不创建新 subtype walker。

## 与历史 nested-number-array 报告的区别

`results/em18-baseline-failure-analysis.md` 记录的 `nested-number-array` 是另一种旧输入：先将 `Integer[]`、`Long[]` 各存进局部变量，再创建 `Number[][]`，在旧 report marker BCI 38（最后一个外层 `aastore` 为 BCI 37 后）拒绝 `Integer[]` 对 `Number[]`。那条报告是在旧产品快照中观察到的类型呈现拒绝，不是本审计描述的 inline child ownership 失败。当前 `initializer_reference_widens` 已有等秩数组 + 六个 boxed Number leaf 的闭表事实，单独测试明确覆盖 `Integer[] → Number[]`、`Integer[][] → Number[][]`、反向和 rank mismatch（`crates/jarde-java/src/build.rs:33564-33611`）。所以旧 BCI 38 不能用来证明当前 local-child 形态仍是 type gap，也不能替代 direct-v3 `ownGridDirect` BCI 24/45 的结构链分析。

同一份旧失败分析还说明 fresh `new` 元素由于唯一 reader 是 `aastore` 而在 assignability 判断之前被拒绝。这与 ownGrid 候选链的 ownership 问题一致，但它不是“组件赋值非法”的证据。日志中的 Jarde compile/run exit 0 也不能算恢复成功：相关 direct 输出是 explanation-only/fallback，且原始 trace/输出不匹配。

此前的 `results/em18-next-architecture-audit.md` 只分析 scalar boxed-number rows 与单层直接 initializer 路径，没有审计 child-array gate；它对此范围不构成正向结论。此报告在该范围补充架构边界，不修改历史记录。

## 最小复用方案与原子性

建议下一片单独处理 child-array covariance：

1. 不改 `array_initializer_reader` 的唯一 reader、闭 interval、消费位置和 effect 顺序约束；解除当前 exact `component == child_type` 对结构 chain 的截断，保留准确 reader/identity/ownership facts。initializer 元素由 Builder 复用 `initializer_reference_widens` 并锚定父 `store.bci()`：`Number[][]` 中的 `Integer[]`/`Long[]` 应由现有表通过；`Base[][]` 中的 `DerivedA[]`/`DerivedB[]` 仅在该外层 store 有匹配的 exact snapshot widening proof 时可呈现。
2. 类型 proof 未能证明时，Builder 必须拒绝完整 initializer/body projection 并 fallback；结构候选和 Site 来源仍可作为已闭合事实留存。这不是允许未知类型的 initializer 成功，也不允许只渲染 child、吞掉 parent `aastore` 或发表部分正文。
3. 保持结构 facts 与 Sites 按 complete parent chain 批量认领、每个 Site 只迁移一次。独立负例应证明缺失/错误类型关系即使结构 reader 已闭合也不会产出成功正文，同时 parent 与 child 的实际 store/reader 结构未被误报为不闭合。

## 独立验收范围

正例需要运行冻结的 direct-v3 完整六源文件家族（`Main`、`Base`、`DerivedA`、`DerivedB`、`LocalInterface`、`Mid`），分别以 javac 8 和 javac 23 输入。至少完整恢复并重编/运行：

- `numberGridDirect()`：`Number[][]` 直接包含 `Integer[]` 与 `Long[]` 子数组。断言外层运行时类型、两个 child 值和 trace 顺序与原始输出一致；这是现有闭合 platform relation 可接受的 child-array widening。
- `ownGridDirect()`：`Base[][]` 直接包含 `DerivedA[]` 与 `DerivedB[]`，外层 store BCI 对应准确 hierarchy facts。断言 `Base[][]`/子数组运行时类型和 trace `12` 一致，且 `mark(1)` 先于 `mark(2)` 执行。若测试走 method-only 请求而无 snapshot proof，应拒绝；完整 family/class-source 请求则显式提供精确两 store 的 proof。

负控制应区分结构与类型：

- 结构负例：改坏 child 的 sole-reader/顺序闭区间（例如额外 reader 或不匹配 index/store），parent 与 child 均不得提交，不能留下局部 construction Site。
- 类型负例：在 verifier-valid classfile 中使 `Number[][]` 的 child store 写入 `String[]`，保留真实 `aastore` 与其 `ArrayStoreException`；证明该不兼容关系没有被 initializer 或 child Site folding 吞掉。普通 Java 源码无法编译 `new Number[][] { new String[] { ... } }`，不得把非法源码当成此负例。
- hierarchy proof 负例：`Base[][]` 使用缺失、错误 BCI 或错误 target 的 snapshot proof 时保持拒绝，确保没有从名字相似或数组等秩本身推导 `DerivedA[] → Base[]`。
- rank 负例：错误维数的 child 保持拒绝；现有 helper 已覆盖 rank mismatch，array composition 应继续沿用该结果。

这片只报告设计与已有证据，未运行 Cargo/Java/Git，也未改产品、测试或 fixture。
