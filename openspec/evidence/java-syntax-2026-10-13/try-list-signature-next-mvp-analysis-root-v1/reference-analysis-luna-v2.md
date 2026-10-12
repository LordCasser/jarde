# CF11 / EM20 下一 MVP：Try 前缀 + 泛型 List foreach 的交集范围

**性质：只读架构研究。** 未运行 Jarde/JADX 构建、测试、JDK 或 CLI；未修改 Jarde/JADX 共享源码、测试、账本或 OpenSpec。以下“源码可复用”是代码静态阅读结论；“既有观测”只引用已经留存的原始取证/复核记录。

## 结论与边界

下一项应作为当前“完整 List 用途证明”和“protected prefix-loop Region 证明”的窄交集：只接纳同一完整方法中**一条普通 typed catch row 精确保护 try 前缀和已证明的 iterable foreach loop**，并且对 `List<String>` 参数沿用现有闭合用法集合。具体的控制流 envelope 保持 [recover-protected-prefix-loop-sequences/proposal.md:7-10, 21-27]：一个普通单 range try；body 为单个前缀序列后接 loop；range-end 的纯 continuation 在 try 外；named row 由当前 Catches 绑定；每个 block/异常边仅一个 owner。当前保护前缀候选测试把 `handled` 固定为 `Straight + Loop`、出口 47、handler 50、loop blocks `[7,16]`、try body `[0,7,16]`；有尾部 sibling、fallback 和 nested Try 的拒绝样例（[protected_prefix_loop_tests.rs:495-615]）。这份 Region 证书只证明此结构。

投影的语法边界不能缩成“只让 handled 一个方法过”，也不能扩成“任何带 try 的 foreach”。最小有意义的测试组是把上述单一 Try envelope 与**当前已接纳的 List 用途种类**交叉：直接把精确 `List<String>` formal 当 foreach iterable；直接用另一个精确 List formal 作 `add(E)` receiver；保留物理 `Iterator.next()` 后的 String checkcast；至少一个 loop body 含已证明的条件/嵌套 loop，另一个是 straight body。参数名、方法名与debug变量名不能作为身份门槛。要是某个嵌套形状不落在当前完整 `Region`/`Program` 范围内，该形状仍拒绝，不为了测试数量放宽所有权。

现有 generic-list 设计列出的有限用途是 `iterator()`, `add(E)`, `remove(Object)`, `size()`, `clear()`，以及嵌套 foreach、条件；只在真实 receiver generic 闭合后删 `add(E)` 的合成 Object wrapper，`remove(Object)` wrapper、重载选择与物理 checkcast 要保留（[recover-structured-list-generic-bodies/proposal.md:7-12, 24-27]；[design.md:15-20, 22-28]）。跨到 Try 后仍应逐调用复用这项闭集与 exact descriptor/receiver证书。不要把源码写成 typed `List<String>` 就视为调用实参可赋值。

本轮原 generic-list change 的 design 明确把 `handlers` 列为 no-go（[recover-structured-list-generic-bodies/proposal.md:9-12, 24-27]；[design.md:11,15-19]）；Try-prefix-loop 是独立结构变更，其 proposal 也将“泛型 handler 投影”列为非目标（[recover-protected-prefix-loop-sequences/proposal.md:25-27]）。所以这项交集需要在两 change 完成后单独立项和审阅，不能借当前 Try Region 变更顺手打开，也不能把旧 List ledger 的 `handled` refusal 预期静默改成成功。71/612、CF11 完成计数均不变。

## 测试账本的交叉覆盖

`tests/structured_list_generic_bodies.rs` 是当前 List 投影的主要永久回归：

- `fixed_nested_string_body_projects_source_and_preserves_physical_origins` 测 `List<String>` nested foreach / 条件 / add 与 remove overload，以及 default/all/source-map 一致（[lines 235-331]）。
- `differently_named_string_and_integer_classes_follow_their_list_element_type` 证明同类逻辑的名称变化和 `List<Integer>` 不是按固定类名识别（[lines 333-396]）。这可继续作元素类型差异控制，但下一项目标是 List<String> Try 交集，不因此要求放开 Integer try。
- `nearby_list_controls_refuse_only_unproved_generic_projections` 保留 `rawAliasEscape`、参数重赋值、wildcard 写入、`indexOverload` 拒绝及 raw-signature control（[lines 132-180]）。
- `raw_alias_escape_reference_lifetimes_match_default_all_and_source_origins` 固定两个 foreach 复用局部槽时的 Integer/String 局部声明和 checkcast、raw alias `escaped = target` 与 `escaped.add((Object)value)`，及 BCI 来源（[lines 625-699]）。这是硬边界：泛型 List receiver证书不能吞掉别名或局部生命期差异。

现有 try / loop ledger 是正交拓扑：`tests/p3_loop_try_handler_entry.rs::catch_rejoins_inside_its_enclosing_loop` 及同行 handler-range/共享 row/跨 region write controls 覆盖“loop 内 catch 并回到 loop”这一方向（[lines 53-108, 124-236]），不是 Try 包住 prefix+foreach 的替代正例。`tests/p3_typed_catch.rs` 有 named row/多 row/共享 handler 等 catch 语法；`tests/p3_nested_try.rs` 有 nested catch 形状；`tests/p3_twr_enclosing_catch.rs` 覆盖 TWR 与 catch。它们保护各自入口，下一项只复用单 ordinary typed catch 证书，不扩展 multi-catch/nested try/TWR/finally。

CF11/EM20 近邻还有一个需要保持独立的实际债务：`openspec/completion-review.md:5-10` 记录 `rawAliasEscape` 的局部槽 5 两段生命期、BCI 27 Integer 与 69 String checkcast，并记录旧 Jarde 把二者合成单 Integer 声明会导致完整生成类编译失败；同处记录旧 JADX raw alias 反例会把 Integer 直接写入 `List<String>` 并双 JDK compile fail。它属于“局部生命期/别名及 call-boundary”错误，不是 Try/Loop 证书放行理由。

## 本地 JADX 测试对照

以下来自 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java` 的静态测试源码；本报告没有执行这些 JUnit 测试：

| JADX test | 明确覆盖 | 对下一项的用处与限制 |
| --- | --- | --- |
| `loops/TestNestedLoops.java:12-41` | `List<String>` 双 foreach、if、`add`、`remove`，并断言泛型循环头 | 泛型 List 使用语法与局部循环，**无 try**。作为当前 accepted List-use 参照。 |
| `loops/TestIterableForEach4.java:11-28` | `List<Object>` foreach + 分支/break，断言循环不是 while | 证明 foreach body 转换有条件/transfer 限制；没有 try 前缀或具体 E。 |
| `trycatch/TestLoopInTryCatch2.java:14-49` | Java try 初始化 `BufferedReader`，再 while，close/suffix 和 IOException catch | ordinary try 前缀+loop 的近拓扑，**不是 foreach/List，也含 try 后 suffix**；只能证明 JADX 测试有该结构，不证明 Jarde 可接纳。 |
| `trycatch/TestLoopInTryCatch.java:9-60` | Smali 中 loop 包 try/catch，给出数种 break/return 汇合输出 | 反向嵌套；应用作负向 topology control，不能与 outer try-prefix-loop 合并。 |
| `loops/TestLoopRestore.java:9-17` | Smali 测试断言 `try` 与数组 foreach 同时保留 | 覆盖 try + foreach 共存，但 iterator/array 类型，不是 List<E> signature。 |
| `trycatch/TestTryCatchFinally3.java:15-43` | Java try + List 的 foreach + catch + finally，断言四种正文节点 | 最邻近组合正例，但带 finally；下一 MVP 仅普通 catch，不能顺手泛化到 finally。 |
| `generics/TestGenericsInArgs.java:11-45` | 方法 Signature 的 `<T>`, `List<? super T>`/`Set<T>`，含 debug/no-debug 形式 | 方法泛型参数打印及 debug profile；不是局部 Code Var 使用证明，也不是 Try/foreach。 |

这些测试分别锁住维度，并没有一个现成测试同时保证 List<E> 用途闭合、try 前缀/handler ownership 和 raw-alias 安全。JADX 适合作为算法对照和额外完整源输出参考，不是正确性 oracle。历史 CF11 完整类观察应继续把 original 作为行为 oracle；JADX rawAliasEscape 的错误源码和 compile failure 要原样保留，不要为了和 JADX 对齐而接受它（[completion-review.md:5-10]；既有完整类矩阵也保存在 `recover-structured-list-generic-bodies/results/six-baseline-summary-root-v2/README.md:1-5`）。

## JADX 可借鉴的算法，不移植其结论

1. **Signature / erasure 对齐。** `SignatureProcessor.parseMethodSignature` 从物理 Signature 读类型参数、参数和返回类型，先 expansion，再逐个与方法 descriptor 比较，匹配才更新方法参数类型（[SignatureProcessor.java:180-255]）。这适合作为“声明签名结构与擦除一致”的源；它不分析该方法体所有写入是否满足 E。
2. **foreach 识别 / receiver 元素类型。** `LoopRegionVisitor.checkIterableForEach` 检查 loop test 只有 iterator 一个输入、iterator SSA 无 phi、恰有两次使用、生产为 `iterator()`、消费者为 `hasNext()`/`next():Object`，并把 cast/next 与 loop 内局部所有权对齐（[LoopRegionVisitor.java:246-339]）。`fixIterableType` 用 iterable 的 E 和 next 变量类型比对，只有一致/可证明子型或受限 `? extends` 才修正变量类型（[same:342-384]）。可借鉴 exact iterator-use 关系和类型一致性条件；它返回成功并改写 IR，本身不是 source-closure/catch-row 证书。
3. **普通 Try 区域提取。** `ProcessTryCatchRegions` 遍历包含异常 handler 的 regions（[ProcessTryCatchRegions.java:27-39]），遇到 LoopRegion 会递归 loop body（[same:69-79]），再按 dominator 可达性抽取 Try，排除 handler 可达的块（[same:81-117,120-130]）。该可变 CFG 区域算法是拓扑参照。Jarde 当前 `named_catch_rows_for_shape`/Frame/Region 证书更精确，要求异常表行和当前 Catches 精确绑定、逐块归属及预算/Stop；不能替换为“路径从 dominator 可达”。
4. **Debug 局部类型/名字与 SSA CodeVar。** `DebugInfoAttachVisitor` 把 LVT/LVTT 区间和 register关联，若存在 local Signature 就解析、展开并校验后附在指令上（[DebugInfoAttachVisitor.java:100-183]）；`DebugInfoApplyVisitor` 尝试从 definition/use 的 debug attrs，再按 SSA use offsets 与 LVT 范围套用 type/name（[DebugInfoApplyVisitor.java:64-147]）。`InitCodeVariables` 把 connected SSA phi vars 连接成 CodeVar，只有唯一 immutable type 才给组合变量类型（[InitCodeVariables.java:39-110]）；ForEachLoop 还用 fake region insns 保存 loop变量与 iterable供后续 codegen（[ForEachLoop.java:9-39]）。参考点是“名字/类型选择应有局部区间及 SSA 关联，不能按 slot/name 字面猜”。Jarde 禁用 debug metadata 的 `-g:none` 也必须证明同一结果；debug 名称不是 generic candidate 的充分条件。
5. **调用形参与类型替换。** `MethodInvokeVisitor.processOverloaded` 先取调用目标的 MethodDetails，再对每个候选做 type-variable substitution，然后依据实际 compiler types/casts解决 overload（[MethodInvokeVisitor.java:80-126,216-299]）；`TypeUtils.replaceClassGenerics` 根据 receiver 实例的 class vars 建映射（[TypeUtils.java:154-185]）；`MethodUtils.getMethodGenericReturnType` 暴露被替换后的 generic return type（[MethodUtils.java:86-95]）。可参考“先绑定调用声明 receiver，再做形参实际类型和 overload 检查”。Jarde 当前有限 List contract + exact call key 更窄，也更适合其 proof API；不要引入 Jadx 全 classpath mutable TypeInference，也不要照它的 cast minimization 得出删除真实 Object/checkcast 的结论。

**raw alias 的关键反例：** `LoopRegionVisitor.fixIterableType` 只看 iterableArg 的当前 ArgType 和 loop var type，并在 generic iterable 上为 iterator loop variable 设置/接受 E；它没有本函数内的全局参数 alias closure。CF11 `rawAliasEscape` 中 `escaped = target` 把已参数化的 `List<String>` 放进 raw local，再把 `Integer` 加入；若 decompiler 将别名擦掉并按 target 的 Signature 恢复 `add(Integer)`，源码与 heap-pollution/checkcast 语义不一致。JADX 的历史 actual CF11 capture确实在这一方法省掉 `escaped`，给 `List<String>` 调 `add(Integer)`；这属于已有实测错误（[completion-review.md:5-10] 和已固定 raw source/map 测试 [structured_list_generic_bodies.rs:625-699]），不能因其他简单 List foreach 测试通过就接受。`CodeVar`/LVT 关联提供变量/生命周期提示，但不证明 alias 已闭合；缺任何一个赋值/读取/call consumer 就保留 raw fallback。

## Jarde 的连接方式及是否需要新机制

已有的连接件足够，不需要新的 SSA pass、generic solver 或公共 API：

- **Try 结构面：** 同次 `guard::catches`/named row、protected Frame、Region owner 及既有 bridge/Stop 认证，精确只认领 try 前缀、foreach 循环和 catch handler；不把异常边并入普通 CFG。
- **物理 census 面：** 延用当前 `GenericReturnCandidate`、`generic_void_unread_parameter_slots` 对 Code/operand/SSA/effect/operation 一一对应、参数 slot 读取/写入/直接 invoke receiver 的现有证明。当前 generic candidate 对任意异常行及 `effect.handlers()` 均早拒（[report.rs:9128-9136, 9298-9306, 9566-9593]）；将来应增加一个被精确 named Try/Region 证书限定的窄 candidate 路由，不删除现行 no-handler 路由的完整性门。handler-owned指令也要在 same-run census 对齐，且用现有 budget、poll、Stop 原样传播。
- **Source/type 面：** 继续通过 `ordinary_parameterized_declaration` 与 `prove_generic_list_parameter_body` 证明每个 physical parameter read 对应 source local / foreach 或 exact receiver call；`add(E)`、`remove(Object)` 和其他平台方法分开认证，projected declaration、wrapper移除、source map 一次事务发布（[class_source.rs:7183-7243,7407-7445]；[facade.rs:12666-12760,12822-12900,16047-16077]）。Region 不替代这面证明。

因此架构上只需要**一个与已认证 Try 证书绑定的受限 VoidBody/census 分支**；不是新的通用类型或控制流机制。正例失败/stop 时继续原 raw source+marker；未知 handler、语法超出现存 Region/Program 完整性或参数 alias 未闭合就拒绝。下一 change 应给双证书交叉绑定一条永久 full-class positive 和最小变异反例，而不是依赖 JADX 结构测试补齐 Jarde 的证明。

## 实际观测 vs 静态推断

- **已留存的事实：** `completion-review.md:5-10` 记录 CF11 原类双 JDK 可编译运行，早期完整 Jarde 对 `handled`/`rawAliasEscape` 的真实失败与 JADX raw-alias 双 JDK 编译失败；它明确区分 no-runtime。当前 71/612 ledger 不因其中一方法拒绝或候选测试通过而增加。父任务本轮说明也称 protected-prefix Loop 证书和当前 List 正例分别已经接受，但合成交集尚未运行。
- **静态推断：** 本报告指出的 JADX 内部算法可复用边界、由原 `handled` 结构与现行候选 handler gate 推出的后续连接方式，以及“最小 syntax group”都来自源码/既有设计审阅。没有对本地 JADX 测试运行，也没有试跑下一项 Jarde source projection。
