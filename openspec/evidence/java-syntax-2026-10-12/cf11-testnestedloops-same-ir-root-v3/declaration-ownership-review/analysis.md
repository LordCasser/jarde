# TestNestedLoops foreach 声明所有权审阅（静态推断）

## 结论

内层 `local5` 的拒绝原因已经由 root 的同 IR 诊断实际观测到：`iterator_declaration_before_init`。真实 generated source 表明它不是 `Declare(Some(init))`，而是提升到方法体的 `Declare(None)`，再在外层循环体中执行 `Assign(local5, l2.iterator())`。当前 candidate 在当前层的 `self.stmts` 中找到了初始化赋值，随后又要求同一列表中存在它之前的空声明；声明实际归属祖先层，因此这个 `rposition` 返回 `None`。这不是 generic-signature 拒绝，也不是 `iterator()`/`hasNext()`/`next()` 的证明失败。

统一处理应以现有 `LocalVariable` 和 `DeclarationPlacement` 为所有权身份，避免把 AST `Vec` 下标当成局部变量所有权。最小的安全路径是：candidate 继续证明 iterator 局部及其唯一初始化/消费者，返回“变量身份 + 初始化写入 BCI + 声明 owner/path”的编辑意图；只在当前 loop body 删除经证明的初始化写入。对于声明清理，若它位于当前层之外，不得把祖先下标塞进当前 `remove` 列表。第一轮可保留那个无读者的空 iterator 声明（只是未使用局部，语义不变）；若要清除它，再由完整 AST/region owner 上的稳定身份编辑统一删除。不要把任意 ancestor `Declare` 找到后当成当前 `Vec` 的 index。

外层循环是另一个入口门槛：其 body 首条语句为 `s1 = (String) local3.next()` 的 `Assign`，而 `s1` 声明已提升到方法体。现在只接受首条 `Declare(Some(...))`，因此 outer 在进入 iterator proof 前就被拒绝。可让 binding proof 识别“首条声明初始化”或“首条赋值”，但赋值路径须继续证明写目标是 `next()` 结果经正确 cast 的那个局部。最小安全投影是把增强 for 的合成元素 cast/赋值留在 body，继续写原 `s1`；只有当完整访问/liveness证明 `s1` 可安全收窄为 header 变量时才可省掉该赋值或改其声明位置。此项不是内层 iterator 声明问题的第二个特判，而是对同一个局部写入所有权模型增加一种已证明的 body binding 形态。

## 已观测事实

- 固定 class 为 `cf11/UpstreamNestedLoops`，SHA-256 `8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231`。baseline `execution.json` SHA-256 `5635e60b5565a19ba4fab116d0b6ae73f72e62568e0761edeff2365291c95435`。baseline 原始 javap 与完整 Jarde 输出分别由 pins 固定。
- 原方法 `test(Ljava/util/List;Ljava/util/List;)V`：外层 iterator store BCI 6、while test BCI 13；外层元素 store BCI 25；内层 `l2.iterator()` invoke BCI 28/store BCI 33；内层 test BCI 42；inner next/cast/store 在 BCI 47/52/55。`s1` 对应 slot 4，`local5` 对应 slot 5，`s2` 对应 slot 6。原始 javap 的 LVT 将 `s1` 记为 BCI 27..100、`s2` 记为 BCI 57..97；iterator locals 没有 LVT 名称。
- 完整 Jarde source 在方法体开头输出 `Iterator local3; local3 = l1.iterator(); String s1; Iterator local5;`，然后进入 outer while；outer body 先写 `s1 = ...`，再写 `local5 = l2.iterator()`；inner body 先声明并初始化 `s2`。source 与 report 同时标记 `ordinary_generic_source_unproved`，内容是当前 Program/SSA 无法证明参数化类型下的方法体。
- 同 IR diagnostic raw stderr（SHA-256 `dd853ee46eff134645137cf80380d27418e9d64701c9ae9f53b7195785b1c850`）明确记录：BCI 42 inner attempt 的 `body_first=Declare`，随后 `stage=iterator_declaration_before_init`；BCI 13 outer attempt 的 `body_first=Assign`，随后 `body_first_not_initialized_declaration`。该诊断只揭示 candidate 的阶段；它没有输出每个局部的完整 `SlotUse`、owner path 或 liveness 集合。

## 代码路径与拒绝条件

候选快照 `/private/tmp/jarde-testnestedloops-same-ir-diagnostic-candidate-luna-v4/build.rs.candidate` 的 `Builder::iterable_for_each_candidate`（约 21301 起）先要求 loop/test 形状，随后 entry gate 限定 `body.first()` 为 `StmtKind::Declare { value: Some(_) }`。通过后它在当前 `self.stmts` 反向找同名 iterator 的 `Assign` 或 `Declare(Some)` 作为 `iterator()` 初始化；再在同一 `self.stmts[..init_index]` 里找同名、类型为 Iterator、`value: None` 的声明。最后返回两个 `usize`，调用端在约 19072 起对当前 `self.stmts` 排序并删除。对当前 fixture，初始化项在 outer-loop body 的独立 `self.stmts` 中，空声明则由 method-body owner 输出；因此现有 `Vec<usize>` 删除协议没有表达跨层 owner 的能力。

`loop_region`（candidate 约 18816 起）先 `take(&mut self.stmts)` 暂存 parent list，递归构建 body 时使用新的当前 list，再将 `loop_body` 与 parent list 分离。`declare_at`（约 22810 起）从 `Declarations::at_region` 读 owner path，并在该 region 开始时发出空 `Declare`。所以“先在祖先输出声明，后在子 region 建立初始化赋值”的分层来自既有 lexical declaration planner，而非异常 AST。

`Declarations` / `DeclarationPlacement`（基础源码 `crates/jarde-java/src/build.rs:441-500`）已经按 `LocalVariable` 建立 `Local`、`Elevated`、`Incomplete` 三类放置；planner 根据使用路径的 common owner、首写路径和 reaching-definition 证据确定 `at_region`。`HoistedDeclaration` 保存稳定局部身份、类型、写入锚点 BCI。`reuse::Plan::variable_at(slot, at)`（`crates/jarde-java/src/reuse.rs:89`）能用 slot/BCI 返回 split-slot-aware `LocalVariable`。这些是可复用的统一身份和 owner 证据。

当前 `StmtKind::Declare`/`Assign` AST（`crates/jarde-java/src/ast.rs:753` 附近）只保存名字、类型/值与 origin，不保留 `LocalVariable` 身份。若进行跨 region AST 删除，应以内部身份/owner/写入锚点表示意图；不能只靠名字做全树搜索，也不能将祖先语句列表的位置解释成子列表位置。另一方面，保留已无读者的空 iterator 声明不改变执行语义，是避免新增跨树编辑机制的最小落点。

## 外层 binding 与 liveness 边界

当前 Jarde 生成文本中 `s1` 声明在方法 body，首个写入发生在 outer loop body，后续被 inner equality、length、add/remove 分支读取。javap 的 slot 4 LVT 范围为 BCI 27..100，outer 元素存储 BCI 25；本次诊断没有完整访问图，所以不把 source/LVT 印象升级为 liveness 证明。若把 `s1` 直接移成增强 for header，必须由 `LocalVariable` 的全部写、读、scope/owner 证据证明没有循环外使用或冲突写。无需为了回收该变量而阻止 foreach：可以先把 `next()` 元素绑定保留为 body 中对 `s1` 的赋值。

对 `local5` 则已有 candidate 的 SSA 路径负责证明其初始化结果、header phi 与 hasNext/next 读取关系，且 raw 诊断表明拒绝发生在声明定位前后这个 stage，而非这些检查。未在本审阅中重新运行证明，不能声称所有剩余 proof 已通过。

## JADX 对照与范围

本地 JADX 源码 `LoopRegionVisitor.checkIterableForEach`（`jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java:246-340`）检查 iterator SSA var、phi、恰好两个 iterator 变量消费者、`iterator/hasNext/next` 调用，并检查 wrapped cast/元素变量及 loop region。通过后隐藏 iterator 初始化、相关 use，再替换成 `ForEachLoop`；`RegionGen.java:199-211` 写出 `for (var : iterable)`。测试 `TestIterableForEach.java` 只提供 `Iterable<String>` 的集成输出样例。

这是架构对照，不是可直接移植的证明。JADX 操作 DEX SSA 与 `RegisterArg`/region ownership；Jarde 操作 JVM classfile SSA、`LocalVariable` 和自己的 lexical declaration planner。工作区 JADX 源码版本来自 checkout，其 `build.gradle.kts:14-17` 默认 `JADX_VERSION` 为 `dev`；本次 baseline runtime 使用的 `/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx` 与该 checkout 源码版本必须分别记账。

## 尚未证实

没有读取/重算此方法完整 `SlotUse`、`DeclarationPlacement`、所有 reaching definitions 和 liveness；没有证明 `s1` 可收窄到 enhanced-for header；没有执行 candidate 或任何工具链；没有证明 generic Signature projection 会因 foreach 修复而通过。generic refusal 是报告中的独立现状，必须在后续实际运行中单独观察，不能当作此处 inner/outer candidate 的拒绝原因，也不能假设修复 foreach 可解决它。
