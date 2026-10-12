# CF11 nested `List<String>`：架构建议（修订，静态审查）

## 结论

实现应优先挂在已有的 class-source retained-AST 与 generic-call projection 路径上，别先给整个 `RecoveryRequest` 增一类调用证明再跑第二次恢复。这个任务的成功体已经有完整 AST/物理指令锚点可供检查；需要增加的是局部、可拒绝的“已声明 formal generic type → 具体调用参数”的证明，随后用现成 AST edit/elision API 修改 source-presentation wrapper，并经现有 overlay/transaction 发射。

CF11 当前物理事实是 `Iterator.next()` 擦除返回 `Object` 后跟真实 `checkcast String`（bci 22）。必须保留这条 checkcast；`List.get` 只是一个有用的负向/对抗控制，不能写成当前成功例。调用 `add` 的 source-presentation `(Object)` cast 在 bci 79，由 outer derived anchor 证明为合成包装。

## 应优先复用的现成路径

- [`ClassSourceMethodAstSource`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:6206) 已保留完整 `Program`、物理 method、`parameter_slots`、`complete_code`、exception-handler 状态、完整 instruction BCI 集和 `(BCI, opcode, CallTarget)`。公开 sidecar `ClassSourceMethodAst` 是对这个源的 opaque Arc，见同文件 1342 行。它满足沿同次恢复读结构化方法体、把名字/槽与物理调用身份相连的基本条件。
- [`class_source_invoke_ast_sites`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:2019) 校验完整 Code 标记、唯一 BCI/opcode/target、唯一 AST call node，并返回 receiver 与各参数的 source shape。调用 census 仍由调用方闭合；不能把空的 requested-sites 当作“无其他调用”。
- [`ClassSourceInvokeArgumentPresentationCast`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:1451) 和 [`project_class_source_invoke_argument_edits`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:2955) 已区分 source-only wrapper 与物理 checkcast。它按精确 `(方法内 BCI, opcode, CallTarget, argument_index)` 校验唯一站点，在单次 AST clone/traversal 中移除 wrapper；`generic_call_ast_projection_tests` 已有 presentation wrapper 可去、physical checkcast 不可去的控制。当前 `class_source_expression_reference_presentation_wrapper`（同文件 3589 行）仅接纳 child 为 `Call | Null`，而 CF11 `add` 参数 child 是 `Local s1`；所需改动应是把这一助手有限扩到“能唯一映射到本方法某个物理 formal slot 的 direct Local”，由后续 `Signature`/类型证明把关，不能把任意 Local 当泛型来源。
- [`GenericCallProjectionState::project_generic_call_arguments`](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:729) 是再发射/overlay 接口：它比对原始 artifact 和原 AST emission，再存入 projected emission 与 markers；不是 proof finder。`facade.rs` 15776 行附近已有“AST site → project edits → emit → overlay → stage/commit or rollback”模式可参照，避免把新投影绕过 class transaction。
- 现有 [`GenericReturnCandidate`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:6363) 和 [`generic_return_candidate`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:8975) 仍只覆盖若干窄 body shape；本例不应把“不是 VoidBody”当作必须恢复第二次的理由。应从 retained AST/physical census 走对该循环形态的局部识别与正式参数来源绑定。

仓库当前没有名为 `ast_calls.rs` 的文件（全树文件名查询无结果）；同类现成 API 实际位于 `crates/jarde-java/src/report.rs` 的 `class_source_invoke_ast_sites`、`project_class_source_invoke_argument_edits` 及 `generic_call_ast_projection_tests`。

独立架构债务（从本次 CF11 范围拆出）：AST edit 的 mutable visitor（`for_each_statement_expression_mut` / `for_each_expression_mut`，report.rs 8221、8332 行）递归遍历；调用前有 `program_node_count` IR budget charge，但节点计数不限制递归深度。当前只记录，不混进 CF11 泛型绑定改动。

## 最小完整实现方向

1. 从原已保留 AST 取这个 body 的调用站点，并用完整 Code census 关闭指令/调用集合；以 `parameter_slots` 将 receiver 的 direct local 映射到物理 formal slot，再读取该 formal 已验证的 `Signature`（`List<String>`）。不要靠 AST `presented` 或变量名猜泛型。
2. 真正需要的有限源类型 owner 层次是：参数 formal `List<String>` 绑定 `java.util.List<E>` 的 `E=String`；`List<E>.add(E):boolean` 绑定 add 参数；增强 for 的迭代器来源沿 `List<E> -> Collection<E>.iterator(): Iterator<E> -> Iterator<E>.next(): E` 传递同一个 E。按 release、symbolic owner、声明 owner、name、擦除 descriptor 和 generic Signature 逐一匹配（包含精确继承边）；未知继承关系拒绝。这里只建这几条直接支持 CF11 的契约，不扩成通用 JDK generic resolver。
3. 对 add 只在证明了 source formal String 后，确认 bci 79 的 `(Object)` 是 descriptor-erasure presentation wrapper，产生精确 `ClassSourceInvokeArgumentPresentationCast` removal。`class_source_expression_reference_presentation_wrapper` 需从当前 Call/Null child 有限扩大到 exact direct formal Local；其映射必须用本 AST 的方法身份和 parameter slot。不能改变量的物理类型事实，也不能对其它 `Object` 参数全局放宽。
4. `Iterator.next()` 的返回 source type 由 `Iterator<E>.next(): E` 和该迭代链绑定为 String；bci 22 的物理 `checkcast String` 必须留在 AST/source。用已有 `project_class_source_invoke_argument_edits` 单次编辑，再重新发射并经 `project_generic_call_arguments` overlay；失败则整个投影回滚或拒绝。

已有 `SameClassGenericMethodContract`/`SameClassGenericArgumentSource`/`SameClassGenericSubstitution`（[`class_source.rs`](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:2769)）提供了“物理 method identity + `SignatureType` + 参数来源 + binder 顺序”的可复用形状和窄匹配思路，但实现明确仅用于 same-class；不要伪装成 JDK `List` contract，也不要因此把另一个证明对象塞进所有 recovery requests。

## JDK 契约及拒绝边界

没有发现维护中的有限 JDK **generic method Signature** 契约。[`release_reference_argument_widens`](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:31650) 是 Java 8 `java.util` raw extends/implements 表，不含 `List<E>` 的 `add`、`Iterator<E>.next` 签名；它的负例还明确拒绝字符串化的 `List<String>` 类型名。此里程碑应增加最小、版本约束且可审阅的 List/Iterator 契约条目，并按 owner/name/descriptor/signature 精确匹配；契约不符、版本不符或继承来源不确定就拒绝。

当前 slice 只支持精确 `List<String>` direct formal、已枚举的循环 body 与闭合的站点集合。接收者别名/字段/自定义子类、raw/wildcard capture、参数被重赋值、调用缺站点、descriptor/BCI 不配对都拒绝。不得用层级猜测、变量名、擦除 `Object` 或 AST `presented` 恢复 E。

## 算法参考与对抗控制

JADX [`LoopRegionVisitor.fixIterableType`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/regions/LoopRegionVisitor.java:342) 从 receiver 唯一 generic argument 约束迭代变量，不兼容就返回 false；[`MethodInvokeVisitor.processOverloaded`/`resolveTypeVars`](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/MethodInvokeVisitor.java:102) 先替换 receiver/class 和 method type variables，再按有效 formal 判重载及 cast；`TypeInferenceVisitor.makeInvokeUseBound` 也为 generic invocation argument 单列 use bound。可借鉴数据流方向，不能照搬其依赖全局 root hierarchy 的宽能力。

审查控制应覆盖：正例移除 bci 79 的 `add(String)` `(Object)` presentation cast 且保留 bci 22 `Iterator.next()` 的 String checkcast；`List<Integer>` 实参与 String 不匹配应拒绝；raw List 保持原路径；`List<?>.add` 不得误接受；`List.get` 可作为非当前事实的独立控制；自定义未知子类/receiver alias、formal 重赋值、调用 BCI/descriptor/owner 不配对、重载错绑、method binder 遮蔽 class binder 均拒绝。Root 已报告三 request 诊断来自实际运行，计数不同不影响结构事实；本报告静态审查不把失败 parser 解释成“未运行”，也不宣称自己运行过验证。

## 静态证据身份

以下为只读审查时文件内容 SHA-256（不是 Git commit）：

- `report.rs` `9dde045eb410c797f840a85df5b9dccf711bea140ccce30651fd34243bcd2497`
- `class_source.rs` `bd797db1b47a0258c5031260f867fcd766ae9d5036eb4e8373f7be6fb827a71f`
- `build.rs` `866cc87d55f881886d217b47f5f54fcea789e2fdc925f494a866a386d3bdde0c`
- `facade.rs` `9c55f3a53b0e435ea5c3e2bedac0fe66f32d474aecc9d1237029518f3703f22b`
