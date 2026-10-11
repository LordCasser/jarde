# Hoisted iterator foreach：对抗性静态审阅

## 结论

新 change 的范围适合继续围绕一个内层 `Iterator` loop：保留既有 `Object` foreach header 与元素 cast，只在已有词法 owner 可见性证据成立时接纳祖先声明，初始化赋值删除仅限当前 parent statement list。proposal/design/spec/tasks 对外层 `Assign`、typed `String` header、generic Signature 和跨 frame AST 删除均有明确边界。

仍有两点必须由 task 1.2 真实诊断和实现控制落实，不能从当前证据推断正路径已成立：

1. 当前 helper 在声明查找处先拒绝，后续 SSA/CFG/phi/use/effect 门对目标 IR 是否通过完全未知。日志必须给每一门实际 verdict 和物理输入；“声明门可绕过”不是完整循环 proof。
2. 当前 helper 允许 `iterator()` 初始化与 loop 之间存在 literal-valued 局部写入，而改成 foreach 会把 `iterator()` 求值延到 loop header。这些写入自身没有调用，但可能被 `iterator()` 异常处理路径观察，因而不一定是语义无关。窄实现最好只接受无执行语句间隔（可留空声明）；若要保留 literal assignments，须另有路径证明该时序变化不可观察。

## 已由源码/旧证据确认

- OpenSpec 四项实际内容为 proposal/design/spec/tasks；另读 `.openspec.yaml` 与 `verification-root.md`。strict通过只说明文档形式，不证明代码门已通过。
- 已接受的 same-IR v3 raw 只记录 inner `test_bci=42, header_bci=35, path=[1,1]` 的首拒 `iterator_declaration_before_init`，outer `test_bci=13, path=[1]` 的首拒仍是 `body_first_not_initialized_declaration`。README 明说没有完整 slot use/owner/liveness dump。Frozen Jarde 源文本显示 `Iterator local5;` 提升到方法体，`local5 = l2.iterator()` 在 outer body 内，支持“列表不同”的已有事实，但不暴露运行时 `RegionPath` placement。
- 原 class 的内层关键物理位置是 `l2.iterator()` invoke 28、iterator Store 33、test Load 35、hasNext 37、branch/test 42、next Load 45、next 47、CheckCast 52、element Store 55。原版只有这一条已固定 whole-class 控制；LVT 对 iterator 没名字，slot 5 是它，slot 6 是 `s2`。不要将 BCI 推测当成当前 candidate/SSA 输出；诊断要实测 `LocalVariable` 和读写 ValueId。
- `DeclarationPlacement` 明确区分 `Local`、`Elevated`、`Incomplete`；`Elevated` 通过 `at_region[owner]` 发出无值声明。planner 通过 common owner 和全部 access path 检查 `owner` 覆盖；`declare_at(path)` 在 `region()` 入口发射，并将真实 `LocalVariable` 记入 `self.declared`。因此候选可消费这套现成身份/owner，但它必须连接到候选当前 region path 和已经发出的声明，而不是只看有一个父级同名 `Iterator`。
- `reuse::Plan::variable_at(slot, bci)` 返回 split-aware `LocalVariable`；`None` 可表示某个 split use 无法映射。当前 iterable candidate 内没有调用 `variable_at`：init Store、test Load、next Load 只核对相同原始 slot；因此 change 中要求的“同一 LocalVariable”是需要实际加进候选检查的新联系，不是已由现有 helper 完成的证明。
- `loop_region(region, path)` 在递归构建 body 时 `take`/恢复 parent `self.stmts` 与 `self.declared`，之后才调用 iterable candidate；返回的 `Vec<usize>` 由 caller 对恢复后的当前 `self.stmts` 逆序删除。祖先声明不属于该 Vec。这个时序支持当前 assignment 索引删除，但不支持把祖先 list 的下标塞进来。
- candidate 已硬编码 `ForEach.ty = java.lang.Object`，并将原 `next()` AST 节点替换为 synthetic Object local。`Cast(next())` 分支留住原 cast 和元素声明；“next already Object, then separate Cast(Local)”分支留住 body 的下一条 cast/声明。该代码不是 `for (String s2 : rawList)`。当前 fixture 实际走哪一分支仍需在诊断中报告。

## 需要落实的 owner / index 条件

**Owner identity and visibility。** 在最终 `Builder` 状态（`declarations()` 加 `prove_local_assignments()` 之后）针对 candidate 收集：raw slot；`variable_at(slot, init_store_bci)`、`variable_at(slot, test_load_bci)`、`variable_at(slot, next_load_bci)` 的完整结果；`placements[variable]` 的 variant/owner；对应 `at_region[owner]` 是否含同 identity 及 hoist anchor；candidate 的 `path` 和 `path.starts_with(owner)`；调用时 `self.declared.contains(variable)` 是否为真；owner 声明在生成 AST 中确已发射的语句路径/位置。缺 placement、`Incomplete`、None 或 identities 不同都应拒绝。若支持同一列表中的旧路径，也要把实际找到的 blank declaration 通过 reuse identity 与 placement 对上；名字/slot/类型独自不够。

这里要么把 `path` 传入 candidate（当前签名没有 path），要么使用已有且明确包含该 path 的证明值；不能只凭“owner 是 ancestor”口头判断。owner 应是 candidate loop path 的前缀，且实际 emitter 在进入 loop 前已声明该变量。不能接受 sibling branch owner、深于当前 loop 的 owner、fallback/catch resource owner，或仅在 `at_region` 里存在但本次 walk 未发射的 entry。

**Current-list edit intent。** `init_index` 必须来自当前已恢复的 `self.stmts`；通过 `Stmt.origin` 的 init Store BCI、`Operation::Store` slot、`reuse.variable_at` identity 和 `iterator()` call BCI 对上该确切赋值。新 ancestor path 仅返回 `init_index`。若有同一 current list 的空声明，只有通过同一 `LocalVariable`/owner/anchor 身份证明后才返回它的当前 index。不得递归搜索父 AST，不得跨列表复用 usize。caller 当下逆序删除目前是正确的应用位置；ancestor declaration 作为无用空声明留下即可。

**Stop 与预算。** 当前 caller 先删 `self.stmts` 条目、扣 statement 计数、注册 synthetic name，然后 `self.push(foreach)` 才 poll/charge IR item。`push` 被预算停止时 `build()` 以 `Err(StopReason)` 结束、不会返回部分 `Program`，但要在 task 2.2 用真实预算停止证明上层 report 没有把当前方法的删改 AST 发布成 partial artifact，并记下真实 dimension/usage/location。owner 身份查询和候选新增 work 也要按项目习惯计费；诊断 patch 不得把提前 `return None` 改成最后真的返回 candidate 或越过原子边界。

## 尚未实测的完整 proof

当前唯一 inner 诊断停在声明查找。应用 owner 读法后，必须在同一 frozen class / Program / Builder 路径记录下列阶段，每项给实际 BCI、ValueId/LocalVariable/Block/handler 数据与 verdict；不得以目前源码中有检查推断目标 IR 会通过。

1. sequence/init：从当前 statement list 找到的确切 `iterator()` 赋值，`init_bci`、Store BCI、表达式/receiver origin、init result 唯一 use，以及 init 到 loop 间所有语句和顺序。
2. invoke/type：`iterator` Interface owner + descriptor，`hasNext` 与 `next` exact owner/descriptor，`iterable.presented` 与 Java source owner。固定 class 是 raw List 参数；generic refusal 继续独立，别在这项改参数类型。
3. iterator local identity：init Store 和 test/next Loads 的 slot 与 `LocalVariable` 逐点值；SSA 被替换链、init value use、header Phi slot/value/input/use、header/next block 的未预期指令、所有 iterator slot writes。raw slot 一致不代表 reuse 的 split identity 一致。
4. data consumers：hasNext 与 next receiver Load value 是否都解析到初始 iterator value/同一 header Phi；next result 唯一到原 cast 或 Store；cast 输出唯一到元素 Store；元素写值只到原 cast-load。分别说明当前 fixture 的 inline-cast 与 separate-cast AST 分支。
5. exceptional behavior：init/hasNext/next/test/适用的 cast 的 Effects handler 集合和对应 CFG block。当前代码对 inline cast 才显式比较 cast handler；separate-cast 分支没有同一条 handler 检查，不能在 review 中把“cast handlers 全部比较”写成事实。对实际目标，至少记录 BCI 52 的 `CheckCast`、Store 55 和 handler，并保留 body cast 所在语义位置。
6. final AST/output：outer `s1 = ...`、两层 loops、所有 `continue`/labeled control、inner foreach Object header、原 cast 与 `s2` declaration 的实际 AST/render、删除列表（path + indexes + BCI/identity）、保留 hoisted declaration、`self.declared`、`synthetic_names`、source-map 每个关键 BCI 的归属。`next` 替换 local 克隆原 `next.origin`，foreach origin 又 fold `next` BCI；须观察 normalize 后 BCI 47 是否无漏映射且没有违反来源约束的歧义/重复。检查 init BCI28/Store33、test42、cast52、Store55 同理。当前静态代码无法确定 source-map 校验的最终结果。

## 可对抗的异常时序案例

现有 candidate 在 init 与 loop 之间允许空声明以及常量 RHS 的 `Assign` / initialized `Declare`。例如已有外部 `seen=0`，在 try 中先 `it = list.iterator()`、然后 `seen=1`、再进入 while；iterator 实现若在 `iterator()` 抛异常且 catch 读取 `seen`，原代码 catch 看到 0。foreach 把 iterator 初始化变成 loop header 后，`seen=1` 先执行，catch 会看到 1。literal local assignment 没有调用副作用，却可观测地改变异常路径上的局部状态；当前相等的 handler set 不封闭该读写顺序。

固定 CF11 输入的初始化后紧接 while，没有这类中间写入，因此它不妨碍该固定目标的最小修复；它会影响把这段 helper 作为更一般准入的声明。窄实现应在候选条件中拒绝所有有执行语义的中间语句，只容许空声明，或另证中间 Store 在 init throw 的 handler 路径不可观测。若保留当前常量赋值放行条件，change 对“初始化异常/顺序保持”的要求需要一个有 throwing iterator、可观察 catch local 的实际负控制；不要用标准 List 的非抛 iterator 代替此边界。

## `Object` header / cast 的具体边界

应将已存在的 `Object` header视作这一片的既定 AST 输出，不增 typed header 或 generic helper。对于原 class 的 `Iterator.next()Object` → `CheckCast String` → slot 6 Store 55，改写必须把仅 `next()` producer 移到 Object header，body 仍消费同一次结果并执行 cast，再声明 `s2`；不能让新 header用String隐式 cast、删掉Bci52、重调next或令原Store再读另一元素。

两个候选 AST 分支检查深度不同：inline-cast分支直接用 cast 表达式 BCI 检查 `Operation::CheckCast` 与其结果 use；separate-cast分支从 body 第二条找 `Cast(Local(binding_name))`，核 stored element 唯一进入 cast 输入 load，但没有在本 helper 中显式比较该 cast 表达式自身 origin 是否为 `Operation::CheckCast`，也没有对它单独取得 effect/handlers。后者可能由上游 IR/AST 保真和规则保证；这份静态材料没有证明。因此目标诊断要报出 actual branch/cast origin。若固定目标是 inline branch，可对窄目标实现；不要无证据声称两个编码具有相同 cast gate。

JLS enhanced-for 的 Object目标可兼容 raw `List`，已有 `String` cast留在body；JADX `fixIterableType` 不参与此 proof。本片独立保持 generic refusal 和 outer assignment拒绝。

## 必要的对照矩阵

- 固定 positive：原 current CLI + frozen `UpstreamNestedLoops.class`。诊断版在最终候选投影前强制拒绝/停止，保存完整 raw；对每个 gate给 verdict；不得交付 diagnostic build 为永久实现。
- Existing outer negative：test BCI 13 首条 Assign 保持 `body_first_not_initialized_declaration`，不触碰 outer `s1`。
- Local-variable negative：init/test/next 的 `variable_at` 不同或 None；same raw slot / same name 不得通过。保存最小可复现 class 或现有真实 split reuse fixture，不仅直接构造假的名字判断。
- Placement negative：Incomplete、owner不是候选path前缀、owner条目身份或anchor不匹配、或本次未发射/未 declared；继续保留 while。
- Iteration/exceptions negative：extra read/write、shared init result、wrong phi/input、next 多consumer、cast/store非唯一、关键handlers不同；分别证明真正拒绝的第一门。
- Evaluation-order negative：如果中间 literal assignment 被保留，使用会从iterator()抛出的自定义 Iterable 和 catch-observed outer local，原 class vs 未编辑 generated class 对照。若把候选收紧为无执行语句，该fixture应准确拒绝。最小目标可先选择“只有空声明”的实现边界。
- Projected output：default/all/noRuleDetails同一结构决定；物理BCI 28/33/37/42/47/52/55无来源缺口；完整原始运行仍覆盖原有列表修改与异常行为。

这些是审阅建议，不是新增的 OpenSpec artifact或实现批准。当前 source snapshot在 pins.json 中固定；本审阅没有运行 Cargo、JDK、JADX、CLI、Git 或 network，也没有执行诊断。
