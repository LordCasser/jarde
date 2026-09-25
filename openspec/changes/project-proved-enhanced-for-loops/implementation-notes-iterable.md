# Iterable 4.2 实现边界审计

本说明按当前 `crates/jarde-java/src/build.rs` 与 `ast.rs` 的结构提出最小实施切片；不代表该切片已经实现或通过行为验收。证据边界见 [`remaining-4-1/analysis.md`](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/analysis.md)。

## 入口与必要证明

候选只在 `build.rs` 的 `Region::Loop` 已完成普通 `While` 后检查（约 4422–4578 行），不扩大 `ForHeader` 数组候选。第一版限定 `java/lang/Iterable.iterator()Ljava/util/Iterator;`：`CallTarget` 已直接提供 invoke kind、owner、name、descriptor 与 interface-reference（[`facts.rs`](../../../crates/jarde-java/src/facts.rs) 365–404 行）。必须验证 interface 调用、精确 owner/name/descriptor；`Iterator.hasNext()Z` 与 `Iterator.next()Ljava/lang/Object;` 也逐项验证。不要把这个白名单扩成任意 `iterator()` owner，也不要把 `List`/`Collection` 当成 `Iterable` 的隐式子类型：本仓库没有通用接口继承判定。

Jarde 当前输出会把 iterator 槽写为单独的 `Iterator localN;`，再在循环前写 `localN = expr.iterator();`。候选要识别这对声明/赋值；完成 SSA 排他性证明后同时移除赋值和现在无读者的声明。若头部 phi 透传使 `uses()` 比预期多，不能简单放宽总数；照数组长度约 5525–5580 行的做法，只接受入口定义加自身回边的已证明 phi，并确认真实消费者仅有 test/next 路径。保留 `expr` 原 expression tree/source，例如 `((Iterable) supplier.get())`，foreach 中只能求值一次。

对已构造的普通循环 AST，要求循环条件根节点是该 iterator 的 `hasNext()`；循环体首个可观察语句必须是该 iterator 的一次 `next()` 结果绑定。若体内以 `Object raw = it.next(); String value = (String) raw;` 表示转换，保留这一绑定及原 cast 声明。禁止有第二个 `next()` 消费者、其他 iterator 逃逸/消费者、额外前置效果、不同执行路径汇入绑定、非直接首动作或无法确定顺序的表达式。`ValueFacts::uses()` 的现有精确 use 记录可做独占消费核对；局部槽名/文本相同本身不是证明。next 结果的 cast 与副作用顺序必须仍由原 body AST 表达，cast declaration 的来源 BCI 应维持原绑定/cast BCI。

异常集合必须取自每个真实 invoke BCI 的 canonical instruction effect（当前数组候选以 `self.ssa.effects().instructions()` 按 BCI/block 查 effect，并比较 `handlers()`，约 5685–5694 行）。首片要求 `iterator()` 初始化、每轮 `hasNext()`、`next()` 与移动后的元素读取/转换相关可抛指令处理器集合一致；至少 `hasNext` 与 `next` 必须严格相同。JVM `next()` 进入 foreach 头部后位于原 body 的 try 之外，4.1 JADX 反例已证明不同处理器集会把捕获异常改为逸出。无法取得 effect、不能以 BCI 对齐或集合不一致一律拒绝；不应借局部变量声明规划去猜 try 语义。

## 最小 AST 变换

现有节点 `StmtKind::ForEach { ty, name, array, body }` 已足够承载 `Iterable`，但 `array` 字段名不准确（[`ast.rs`](../../../crates/jarde-java/src/ast.rs) 722–728 行，emit 在 `build.rs` 5717 行附近）。可将字段改名为 `iterable`，由同一 emitter 输出 `for (Object item : iterable)`；不需要第二种 foreach 节点。

对 raw `Iterable`，增强 for 变量必须声明为 `Object`。Jarde 当前可呈现的 `while` body 已含原元素绑定/cast AST：新建无冲突的 synthetic `Object` 名称，把绑定右值中的 `next()` 结果叶替换为该 foreach local，再保留原有 `String value = (String) raw` 语句。不要将元素变量改型为 `String`、删除原 cast 或重建 cast；这会改变 raw 类型擦除下的 CCE 位置。已有 `NameTable::free_name_with` 可检查保留名并计入 IR budget（例如 binding plan 约 6101 行），应复用其预算/碰撞策略，而不是自行拼一个可能冲突的局部名。

iterator 初始化语句在 loop 外，现有 AST 已先把它放进 `self.stmts`；`Region::Loop` 暂存 `outer`、独立遍历 body，再恢复 `self.stmts`（约 4467–4488 行）。因此 candidate 应返回完整、尚未提交的计划：foreach 节点、来源集合、iterator initializer 在 `outer` 中的索引/BCI，以及将被改写的首个 body 语句。全部证明成功后，在同一个提交分支中移除 initializer 与旧 loop 表达、递减 statement 计数并 push 新节点；失败时保持 outer initializer + 普通 while 原样。不可先删除 initializer 再检查 cast/handlers，也不可只把 loop 改成 foreach 留下不可用的 iterator 声明。

来源需包含 iterator 初始化、`hasNext` 条件调用、`next` 调用、原 cast/element binding、原 body 其余语句所携带的真实 BCI；以直接/派生 `OriginSet` 合并，不能只挂 loop test BCI。候选每次扫描 SSA block、phi、调用 use、effect 与命名都按既有方式 `poll`/`charge`（数组候选从约 5320 行起示范）；把新候选检查计入 `AnalysisSteps`/`IrItems`，并确保预算停止返回前未触碰 `self.stmts`、`settled` 或 `synthetic_names`。提交后检查 essential/all 正文和来源一致性的既有契约。

## 可复用结构与真实缺口

没有必须新增的全局 IR、SSA 机制或 handler 模型：调用符号身份来自 `CallTarget`，消费者来自 SSA use-list，循环区域由既有 `Region::Loop` 表达，try 内外归属来自 region path，逐指令 handler 集合来自 effect 表；AST 有 `Call`/`Cast`/`Local` 和 `ForEach`。唯一明确的类型准入限制是当前没有一般 subtype solver：本片严格匹配 direct `Iterable.iterator` owner 可作为显式边界，但 receiver 的 Java 源表达式仍须具备可编译的 `Iterable` 类型证明（例如 parameter/field 的已呈现类型）；若只能从栈 descriptor 推断或需要 `List`/`Collection` 到 `Iterable` 的继承关系，则拒绝并保留 while。

现有名称层是 slot/reuse 规划而非可随意增添元素局部的 API，foreach synthetic local 必须沿用其保留名集合并有明确 scope；AST 变换也不应依赖 `SSA value.replaced_by` 去改静态类型。除此之外未发现阻止 direct-Iterable 最小子集的架构阻塞。4.1 的多重消费、cast 次序、首动作与 handler 反例都是应明确拒绝的形状，不是扩大分析器接受面来解决的理由。
