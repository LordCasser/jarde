# `preserve-deferred-value-order` 接入地图

本地图只记录当前实现的事实和接入边界，基线是本 change 的 `proposal.md`、`design.md`、`specs/java8-recovery/spec.md`、`tasks.md` 及当前源码。这里的“延期”指生产指令不在自己的 `instruction` 分支产生 Java 语句，而把值交给最终 reader 的 `render_value(value, at, depth)`；现状对 call 有显式记录，对其它值有隐式延期。地图不引入新的 pass、全图缓存或 AST 形状。

## 1. producer、构造入口和现有延期路径

构建入口在 [`crates/jarde-java/src/build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:980)。`build` 从 `SsaTable::blocks()` 建立 `instructions: BTreeMap<u32, &SsaInstruction>` 和 `block_of: BTreeMap<u32, CanonicalBlockId>`（988–994），然后建立一个共享的 `Builder`（1007–1055）。`Builder` 的 `deferred: Vec<(ValueId, u32)>`（1167–1170）目前只记明确被延期的 invocation 值及 producer BCI；`quoted_bcis`/`deferred_producers`（4934–5077）再以该记录和现有 operation 递归补回 fallback 来源。array/field/cast/new 目前没有同等的显式绑定记录，它们靠值链在 reader 处重建。

| producer / 语义 | 生产指令的 `instruction` 分支 | 当前最终表达式入口 | 现有独立语句或延期事实 |
| --- | --- | --- | --- |
| 普通 `invoke*` | `Operation::Invoke`，[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2231) 起；无 local 且 `produced_value_reaches_a_reader` 为真时 2281–2298 直接返回，并将每个 stack `ValueId` 放入 `deferred`。有 local 或不能延期时走 2300 的 `call_expr`。 | `render_value` 的 `Operation::Invoke`（3707–3721）调用 `invoke_expr(bci, instruction, target, at, depth)`；receiver/args 递归都以最终 `at` 为评价位置，`call_expr` 本体在 3943–4057，descriptor 绑定在 `arguments`/`invocation_argument`（4201–4345）。 | 有 local 的 invocation 是 `write_statement`（2325–2328）；无 local 且未延期的是 `StmtKind::Expr`（2312–2317）。延期调用最终 reader 失败时由 `quoted_bcis` 取回。 |
| 已验证 `new` 构造 | `init.rs` 中 `Site`（[`init.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/init.rs:65)–82）把 `head/dup/constructor` 和 constructor 参数收成一个 site；`Sites::sites`/`verify`（222–430）验证 immediate `dup`、同块 `<init>`、receiver 身份、参数区间、site 内操作和唯一外部 reader。`instruction` 在初始 skip（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2140)–2145）跳过 `sites.owns(at)`，所以 head/dup/constructor 不各自产生语句。 | `render_value` 先按任一 owned BCI 调 `sites.site_of`，再调用 `new_expr`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:3605)–3612）；`new_expr`（4534–4582）固定使用 constructor 的参数值和 `site.owned` 来源。 | site 是值表达式的唯一现有承载点；若 site 没有可呈现的外部 reader，普通 allocation 会在 2569–2576 fallback，避免丢掉分配或构造效果。 |
| 已 claim 的 field read | `Operation::Field`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2497)–2517）：`fields.claim(at)` 且 `shape.writes()` 才是 field write 语句，read 返回 `Ok(())`。 | `render_value` 的 field read（3772–3813）按 `field::Plan` 的 `shape.receiver` 递归渲染 receiver；static read 使用 pool owner path。 | read 的文本落到消费它的 store/call/return/condition；未 claim 或下游 reader 被拒绝时，`deferred_producers`（5049–5070）把 read 和 receiver 生产者加入 quote 来源。field write 仍在指令位置，不能被此 change 误归为 value。 |
| array element / `arraylength` | `ArrayLoad`、`ArrayElementLoad`、`ArrayLength`、`NewArray` 在 2523–2551：若 enum 规则拥有或 `produced_value_reaches_a_reader` 成立则没有自己的 statement，否则以 quoted source fallback。`ArrayStore` 单独在 2520–2522 写 statement。 | `render_value` 的 `ArrayLoad`/`ArrayElementLoad`（3815–3875）、`ArrayLength`（3877–3896）、`NewArray`（3898–3930）递归数组、index、length，评价位置保持最终 reader 的 `at`。 | 这些 producer 的 null/index/length/负长度等 JVM 检查属于 producer 自身的效果；当前全图 reader 判断只证明“有文本位置”，没有证明该位置不跨独立 statement，因此是本 change 的主要接入面。quote 侧同样在 5055–5064 保留它们的来源。 |
| 除法/取余等 `Arithmetic` | `Operation::Arithmetic` 是值类，在 2430–2438 无 statement。 | `render_value` 3654–3680 要求恰好两个 stack operand，递归两值后由 `arithmetic_op` 生成 `ExprKind::Binary`。 | `idiv/irem/ldiv/lrem` 的零除异常由同一个 arithmetic producer 产生，不能因 reader 是后续语句而搬到后面；此处没有另一个 builder 分支可补救。 |
| ordinary `checkcast` | bridge cast 直接在 2481–2485 消去；普通 cast 2485–2495 只有 reader 能写值时返回，否则 quote。 | `render_value` 3722–3756 读取一个 stack value，bridge 递归转发；普通 cast 生成 `ExprKind::Cast`，operand 仍在最终 `at` 渲染。 | ordinary cast 的运行时检查需跟随原 producer 的顺序；`deferred_producers` 5039–5047 会在 quote 中保留非 bridge cast。bridge 的现有认领规则保持不变。 |
| `Push`/`Load`/`Negate`/嵌套 arithmetic | `instruction` 2430–2438 全部是值，无 statement。 | `render_value` 3619–3652、3654–3705；`Load` 另用 `slot_name_denotes_the_same_value` 检查 load 处读到的值是否仍可用同一 local 名字表示。 | 这些是允许保留在同一表达式链中的透明节点。若链要跨独立语句，必须有可命名的已保存值；不能把 stale local 当成等价 alias。 |
| `Store`/`Return`/`Throw`/条件/selector | 它们是 reader：store 从 2164 起，return 2330–2351，throw 2353–2372，branch/switch 由 region 层消费。 | reader 在自己的 `at` 调 `render_value`；`return_expr` 在 2624–2651，条件由 `test_expr`/`condition` 在 1941–1962。 | 这是最终写表达式的位置。producer 的 statement 只能在已证明 reader 不能安全承载时保留 fallback 或保存绑定，不能同时再写同一个 effect。 |

`render_value` 的关键不变量是“origin 保留 producer BCI，`at` 表示实际评价位置”（3560–3568）。当前实现把所有嵌套 operand 递归到最终 `at`；这正是 deferred effect 被重排的来源。修复接入点应集中于 producer/reader 证明和一次性的绑定呈现，不改 `ExprKind`/emitter。

## 2. 足以判定依赖和独立语句的现有事实

### 2.1 精确依赖事实

`crates/jarde-jvm/src/ssa.rs` 的 `SsaInstruction`（约 292–346）为每条指令保存有序 `reads: Vec<(Slot, ValueId)>` 和 `writes: Vec<(Slot, ValueId)>`；`SsaTable::value(ValueId)`、`SsaValue::def()`、`SsaValue::uses()`、`SsaTable::blocks()`（约 218–258、462–510）给出 value 的单一 SSA 定义、所有精确 reader 和 block。这里的 `ValueId` 是唯一依赖键：不能用“相同 slot”“相同 opcode”或裸 BCI 代替。

`build` 已把每条 instruction 映射到 `SsaInstruction` 和 `CanonicalBlockId`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:988)–994）；`stack_operands`、`single_stack_read` 和现有 `local_read` 只从这份 read/write 事实取值。一个有限的 deferred-order 证明可以因此沿 `ValueId -> SsaInstruction.reads()` 的实际链走，并在每一跳记录 producer 的 instruction identity、canonical block、BCI 顺序和 operation。`Definition::Entry/Phi/Caught` 的差别也必须保留：当前 `render_value` 对 stack Entry/Phi 直接拒绝（3575–3603），`value_bci` 对 Phi 只给 block BCI（5605–5611），所以 merge 值不能被假装成某个普通 producer。

`SsaInstruction` 的 reads 顺序是指令实际读值的顺序（ssa.rs 的字段注释）；`CanonicalInstructionEffect` 还保存 `block`、`bci`、`may_throw` 和该 BCI 的 handler 列表，`SsaTable::effects()` 暴露同一 instruction 粒度的异常事实。该事实与 `CanonicalCfg::throw_sites()` 对齐：异常边属于具体 throwing instruction，不属于粗粒度 block 尾部。涉及除零、空 receiver、数组边界、cast、allocation 或 constructor failure 的保存判断，必须以这个 instruction/handler 事实为边界，不得以“表达式看起来纯”替换。

### 2.2 block/path 和 producer 顺序

`CanonicalBlockId`（[`canonical.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-jvm/src/canonical.rs:70)–103）由原始 BCI 和 `jsr` call path 组成；`is_clone()` 说明同一原始 BCI 在不同 canonical clone 中不是同一执行位置。`CanonicalCfg::blocks/edges/throw_sites/unreachable`（315–387）保留正常/异常/clone 边界。故任何保存绑定或 reader 结论都必须至少保留 `CanonicalBlockId`，或在遇到 clone/跨 block 无法唯一定位时拒绝；裸 `u32` 只能作为文本 anchor，不能作为执行位置身份。

目前的 `produced_value_reaches_a_reader`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:4843)–4875）对 `ssa.blocks()` 全图扫描，只检查某个 `ValueId` 是否被某个 `renders_the_value_it_reads` reader 读取。`renders_the_value_it_reads`（4877–4931）只回答“该 BCI 的文本是否会写入它读取的值”：它不是执行顺序、同一 block、唯一 reader 或独立语句证明。尤其 array/cast/field 等会被列为 reader，但其自身可能会抛异常或是后来重建的 expression。change 的 bounded proof 必须把这两个现有 predicate 限定在实际 value chain 和合法 region scope 内，不能把全图“存在 reader”直接当成可延期。

现有 `numeric_comparison_reader`（约 1964–2057）给出可复用的证明形状：一个 producer 只有一个 stack write、SSA `uses()` 恰好一个、reader 是同一 canonical block 中紧邻的 branch、branch 读取同一个 `ValueId`，producer 自身 reads 数量也满足操作定义。deferred-order 的 proof 需要同样使用 `ValueId`、reader 唯一性、block/path/BCI 相邻或明确的线性区间；不能扩大为新的全图索引。若 chain 中遇到 phi/entry、branch merge、loop back edge、guard/try 的异常边、已有 fallback、未知 operation、另一个独立 statement，证明应失败并交回 `quoted_bcis`/region fallback。

### 2.3 如何区分“同一表达式”与“独立语句”

可保留 inline 的最低事实是：producer 与最终 reader 的 SSA chain 在允许的 producer 集合内；每个中间值只有该 chain 的实际 reader；producer 和 reader 位于同一可见 region/linear block 范围；中间没有已发布 statement、branch/loop/handler 边或不能证明不抛的 operation；各嵌套 producer 的最终评价位置仍是同一个 reader `at`。这使 `call -> cast -> field -> index -> store` 之类表达式保持一次呈现，但不会把 producer 的 throw/side effect 越过已执行的 Java statement。

“独立语句”是 builder 已在 producer 与 reader 之间写入的 statement，或由 region 结构明确分隔的 instruction 区间，而不是仅仅“reader BCI 大于 producer BCI”。现有 `instruction` 逐 block 按 `names.instructions().to_vec()` 顺序处理（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:2105)–2120）；因此 proof 至少要面对同一 block 中的字节码顺序和 region owner。不能通过重新调用 `render_value` 让一个已经执行过的 producer 在 reader 位置重新执行。

允许的保存点是 producer 处的一次 Java local declaration/assignment，之后 reader 只读该名字。保存只对 proof 能证明“producer value 的唯一合法 reader 在该保存可见范围内”的链成立；如果 producer 的值还会被多个 reader 看到，或 reader 处需要不同评价次数/不同分支路径，继续沿 fallback。测试计划中的 18 项 call/field/array/cast、24 项 arithmetic、12 项 allocation 和 construction-order audit 都应归入这个同一判定，而不是为每种 opcode 再造规则。

## 3. region、block、if prefix 与 arm 状态

### 3.1 region 的实际范围

`crates/jarde-java/src/region.rs` 的 `Region`（约 372–459）明确保存 `Straight`、`If { prefix, branch, branch_bci, then_arm, else_arm, join }`、`Switch { prefix, branch, groups, join }`、`Loop { header, test, test_bci, body, ... }`、`Guard`、`Try` 和 `Fallback`。`Region::blocks()`（499–560）是递归覆盖；build 内的 `own_blocks`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:933)–975）区分本 region 自己写的 block 和嵌套 child 写的 block。

`region_paths`/`collect_paths`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:864)–931）为每个 `CanonicalBlockId` 计算 region path，并把 fallback path 单独放入 `RegionPaths::fallbacks`。`declaration_region`（823–846）已经使用 common prefix 决定 local 声明放在哪个共同可见 region；这是保存值的可见性事实来源。当前 `Builder` 只把 path 传给 `region`/`declare_at`/`arm`，`block`/`instruction`/`render_value` 没有 path 参数，故后续接入必须复用现有 path/CanonicalBlock 事实并在不能取得唯一范围时拒绝，不能从裸 BCI 猜 Java scope。

`RegionPath` 是声明计划使用的结构位置，不等同于 Java 词法作用域。多个顶层 `Straight` region 虽然 path index 不同，仍然位于同一个方法 body；`If` 的 prefix、test block 和最终 `StmtKind::If` 也共享该 if 所在的外层 body，不能仅因 path 不同就认定 local 越界。真正新增 Java 块的是 `then_arm`/`else_arm`（以及相应 switch/loop/try body）；共同可见范围应由已有 region 的 ancestor、`declaration_region` 的 common prefix 和实际输出容器共同判断。这个边界允许 prefix producer 被 condition 或两个 arm 读取时在外层保存，也避免把兄弟 arm 的局部绑定提升后互相可见。

### 3.2 if/switch prefix 和 branch

`Builder::region` 对 `If`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:1230)–1268）的顺序是：`declare_at(path)` → prefix blocks → `test_effects(branch, branch_bci)` → `test_expr(branch_bci, false)` → 分别以 `child(path, 0/1)` 构造 arms → push `StmtKind::If`。prefix 和 test block 中 branch 前的 instruction 是真实先执行的 statement/effect，不能因它们的值后来成为 condition 或 arm operand 就挪到 branch 文本内。`test_effects`（约 1920–1940）会跳过 branch BCI，按原 block 顺序写其余 instruction；`test_expr`（1941–1962）才只构造 condition。

`Switch` 同样先写 prefix/test effects，再在 branch BCI render selector；switch expression 的 join return 由 `switch_join` 写入各 arm，并把 join BCI 放进 `settled`，避免 join 再写一次（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:1309)–1362）。deferred 保存不能让 settled instruction 重放，也不能让 selector 的 producer 越过 prefix。

### 3.3 loop/guard/try 和 arm 的状态

Loop condition 在 loop statement 内部通过 `test_expr(test_bci, taken)` 构造（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:1364)–1396）；注释明确说明 test effects/calls 应每次迭代执行，不能 hoist 到 loop 外。因而 loop header/back edge、条件与 body 的执行次数不一致时，deferred proof 应失败；不能把一次保存声明当作每次迭代的等价执行。

Guard 在 1398–1415 起先写 prefix、`range(plan.lead())`，再按资源/monitor shape 写 statement；Try 的 lead 也先通过 `range` 写出。`settled` 记录 try lead 已写的 BCI 及 switch join return，`instruction` 开始处会跳过它们（约 2140–2145、1190–1203）。deferred 不能跨越这些已写/已结算范围重新呈现 producer。

`Builder::arm` 的实现边界还要单独保留：它只临时替换 `self.stmts`，并不保存/恢复 `declared`、`undeclared`、`deferred`、`settled`、`budget` 或其它 Builder 状态；这些状态对两个 arm 共享。因此 arm 内 producer 只能写入该 arm 的 `Vec<Stmt>`，不能因共享集合自动出现在 sibling；prefix producer 若两个 arm 都读，绑定要写在 if 外层 body 的真实容器且 producer 只按 prefix 顺序评价一次。若 producer 只被一个 arm 读取，不能把 arm-local 绑定提升到 if 外层来制造未执行路径上的名字。跨 arm/merge 的 Phi 仍按上节拒绝或保留 fallback。

`NameTable`（[`names.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/names.rs:332)–345）只为 class file 已有 local slot 的 `LocalVariable` 提供确定名称，并保存 reserved/aliased/invented 事实；`free_name`（482–503）会避开 reserved 和所有 local 名称。Builder 已有的 synthetic lambda 名还通过 `param_name`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:5614)–5626）额外避开 `lambda_params`。deferred saved value 若采用现有 Declare/Local 表示，命名必须沿这套 namespace 检查并确定性生成，不能伪造 JVM local slot、复用某个不相同的 `LocalVariable`，也不能让名称只存在于共享 arm 状态而没有对应声明。

## 4. `push`、fallback 与返回契约

`push`（[`build.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:5652)–5664）和 `push_into`（5666–5681）会先 `poll`，再按 `IrItems` charge，然后调用 `undeclared_statement`，最后才把 statement 放入对应 vector。二者的 `Result<(), StopReason>` 只表示 budget/cancellation 是否继续，不能被解释为“原 statement 一定按原样发布”。`undeclared_statement` 可将待写 statement 换成 `StmtKind::Fallback`（5684–5710），设置 `ragged` 并保留该 statement 原本会引用的 BCIs。

`fallback`（5629–5650）同样设置 `ragged`、把 primary/derived BCI 组织成 `OriginSet`，再经过 `push`。`write_statement`（3234–3295）返回 `Result<bool, StopReason>`：`bool` 只表示声明/赋值是否成功；`write_target`/`declaration_refused`（3420–3468）会把拒绝的 local name 记录到 `undeclared`，后续任何读这个名字的 statement 都必须转 quote。

因此 push 可作为保存绑定的唯一发布入口，但调用者必须保留以下契约：

1. expression 只有在 producer-order proof、type/visibility/name proof 全部通过后才可创建和 push；调用 `push` 成功后，不能再让 reader 重新 render 同一 producer。
2. `push`/`push_into` 失败或 statement 被 `undeclared_statement` 替换时，保存绑定不能留在共享状态中；应回到原 producer/reader 的 `quoted_bcis` 或 enclosing region fallback，确保 effect/source 仍被记录且没有半个 local 被引用。
3. quote 的来源必须继续由 `quoted_bcis`/`deferred_producers` 从同一 ValueId chain 收集，不能仅引用 synthetic local 名；否则 fallback 会丢掉 producer 的异常或调用来源。
4. 每次新声明和后续 assignment 都必须走现有 `poll`/`IrItems` 预算；不要用无界的全 SSA 扫描来替代已有 bounded traversal。`MAX_VALUE_DEPTH`（`build.rs` 的 value renderer）和 region 的 `MAX_REGION_DEPTH`（[`region.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs:1061)–1084）仍是拒绝边界。

## 5. `new` 的 site/value alias 唯一映射

`init::Site` 的逻辑身份不是单个 BCI，而是 `head` allocation、紧邻 `dup`、同块 constructor invoke 及 constructor 参数 BCIs 的集合（[`init.rs`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/init.rs:65)–82）。`Sites::site_of(bci)`（156–164）按 owned set 将这三个 BCI 都映射回同一个 `Site`；`Sites::owns(bci)` 用于 instruction 的早期 skip。`verify`（272–430）还保证：constructor receiver 是该 set 内的唯一 instance；参数 producer 严格位于 dup 与 constructor 之间；site 内只允许已声明的无 statement producer；site 外对 produced values 的 `renders_its_reads` reader 必须恰好一个。`outside_readers`/`is_the_instance`/`produced_at`（433–518）都基于精确 `ValueId` definition，不接受 Entry/Phi 假装是 new instance。

`render_value` 对 head/dup/constructor 任一 value BCI 都先 `site_of` 后进入 `new_expr`（3605–3612）；`new_expr` 从 constructor instruction 取实际参数、以 `site.owned` 构建 `ExprKind::New` 及 origins（4534–4582）。所以一个 deferred binding 的 key 必须是逻辑 `Site`（或该 site 唯一的 constructed `ValueId` 加 `CanonicalBlockId`），不能是 `head`、`dup`、constructor 中任一裸 BCI；否则同一 object 会被声明/表达两次，或把 `dup` 暴露成未初始化值。

site 的 argument producer 仍必须使用既有 `arguments`/`invocation_argument` descriptor 规则（4201–4345），并且 constructor failure、argument call、receiver evaluation 的 origin/评价位置要整体归入同一次 `new_expr`。若 site 的外部 reader 不满足 deferred-order proof，保持 init 已有 refusal/quoted source；不要在 init verifier 之外另造“看起来像 constructor”的 site 或把 `Allocate` 直接 render 成 Java expression。

## 6. 有界接入结论

现有最小接入闭环是：以 `SsaInstruction.reads/writes` 和 `SsaValue.def/uses` 取得一条真实 value chain；以 `CanonicalBlockId`、instruction 顺序、`CanonicalInstructionEffect` 的 may-throw/handler 和 `RegionPaths` 判断 chain 是否在同一可见、未被独立 statement/控制流边界切断的范围；对允许的 call/new/field/array/arithmetic/cast producer 只在 proof 成功时于 producer 位置建立一次可命名保存值，reader 复用该 local；proof 任一点失败则维持现有 `render_value`/`quoted_bcis`/region fallback。

这个闭环不改变 `StmtKind`、`ExprKind`、emitter、AST 或 init/region 的职责边界：region 继续只给结构，init 继续只认 verified `Site`，Builder 继续负责一次 statement/表达式的发布和 budget；同一 proof 应被 producer 延期判断、reader rendering 判断和 quote/source replay 共用，避免“有 reader 就延期、render 失败再另算”的分裂结论。
