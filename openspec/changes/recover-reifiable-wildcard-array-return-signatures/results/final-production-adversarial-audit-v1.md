# 泛型数组返回组合对抗审计

## 审计范围与结论

本报告只核对当前 `report.rs` → facade → `class_source.rs` 与 Builder 的静态证据链。没有运行 Cargo、CLI、Java 或变义 class。Root 报告 Java-lib 325 项测试和 facade integration 5/5 通过。本报告不把前片 CLI、旧 CI 或四源回放用作当前实现的证明，也不把它们写成新候选验收。

当前候选的结构边界较窄：只有完整单方法体 `return new T[][] { ... }` 可以提交 `ArrayCreation`；同次 Builder 已成功呈现和证明其 initializer，report 再将同次 Program、Code、SSA、effect、decode Operations 与物理返回闭合；`class_source` 只对无参数、无方法类型参数、无 throws 且返回 Signature 为同 rank、同擦除 leaf 的全 `?` 类型参数数组投影原 Signature 拼写。数组元素的 Java 可赋值性仍由 Builder 在实际 store BCI 上证明。未见这几层之间把结构闭合误当成可赋值成功的路径。

## 同次身份、物理闭集与停止传播

`generic_return_candidate` 在 [report.rs:7796](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:7796) 仅把根返回表达式为 `NewArray` 的情形交给 `generic_array_creation_return_candidate`。候选在 [report.rs:8263](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:8263) 要求无物理形参、无 typed-functional target、非 ragged、唯一 Return/statement、Code 完整且无 handler、单 SSA block、无 phi，并要求 Code/SSA/effect 指令数闭合。返回表达式通过 `ptr::eq` 与唯一 Return 的实际表达式绑定（[report.rs:8294](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:8294)）；根 presented type、按 AST element/rank 得出的实际类型和物理 descriptor 返回类型必须完全相同，allocation/return primary origin 必须为 direct，末条物理指令必须是 `areturn`（[report.rs:8303](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:8303)）。这拒绝伪造的不同返回节点、未知或不匹配的根 presented type、错返回 BCI 和带额外 statement/ragged 标记的 Program。

物理闭集循环逐项对齐 raw Code、SSA、canonical effect 和解码 Operation，检查 BCI/opcode 一致、无 effect handler，并要求每个非 NOP BCI 出现在 Return/完整数组 AST 的来源集合中（[report.rs:8344](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:8344)）。根 BCI 必须解码为相同 element、单个分配维度及相同 total rank 的 `NewArray`；最后且唯一的返回 Operation 必须对应实际末条 `areturn`。返回 stack `ValueId` 只能沿 `dup`、逐步唯一 reader、相同值类型和逆向逐步减小的物理 BCI 回溯到 allocation 输出（[report.rs:8581](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:8581)）。这不是仅凭 descriptor 或候选枚举认定身份。

AST origin census 和 SSA 逆向 lookup 都在同一个传入 `Budget` 上逐节点 poll/charge；内部 lookup 没有未计费 `.find` 预扫。report 的预算/取消测试把 rank 停止锚在 allocation BCI 1，把 lookup 内部停止锚在 BCI 37。报告生产函数实际消费从同次 recovery 返回的 candidate；facade 将它传给同一物理成员的 `project_method_signature`（[facade.rs:7737](/Users/lordcasser/workspace/projects/jarde/src/facade.rs:7737)），投影入口仍要求 `Recovered` 且原 markers 为空，成功投影后会保留成功证明 marker（[class_source.rs:7234](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:7234)）。候选不由调用者输入，不会独立于完整 body recovery 发布源码。

可见的拒绝控制包括缺 initializer、错误 root type、错误 return origin、缺少物理 effect 的表达式来源、ragged 与多 statement（[report.rs:14417](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/report.rs:14417)）。这些是对候选 helper 输入的直接负控制；真实构建路径另由同次 Builder/facade source proof 门约束。仍有一项单向闭集约定：report 要求每个非 NOP 物理指令在 AST origins 中，但不要求每个 AST origin 都能反向映射到真实 BCI。当前 AST 是同次 Builder 输出而非外部可注入输入，未找到能让候选路径产生伪造多余 origin 的生产路径；这属于依赖 Builder origin 合同的边界，不能表述为 helper 独立验证了双向 origin 集相等。

## Signature 形状与拒绝边界

`ordinary_parameterized_declaration` 的 `ArrayCreation` 分支要求 Signature 无方法 type parameters、无形式参数、无 Signature/Exceptions throws，candidate 参数为空；随后调用 `wildcard_array_return_matches`（[class_source.rs:7560](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:7560)）。匹配器要求 candidate 数组拼写与 reader 从物理 descriptor 得出的拼写完全相同；两边 rank 非零且相同，Signature leaf 是单一 class segment、带至少一个类型实参且全部为 unbounded `?`，其 simple name 与物理 leaf 相同（[class_source.rs:7675](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:7675)）。因此 exact、extends、super、type variable、无 wildcard 参数、wrong rank/leaf 与缺 candidate 都走拒绝分支。源测试列出的这些 helper/声明负例见 [class_source.rs:15527](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:15527)。Root 报告 facade integration 5/5 通过；该结果属于 root 的真实门禁，不是本次静态审计运行。

Reader 的 Signature erasure proof 保证类型变量作用域、类型擦除与 descriptor/Exceptions 对齐，但 `SignatureType::Class` 擦除只取最后 segment 的 binary name，不会读取其类声明来核对 generic arity（[signature.rs:595](/Users/lordcasser/workspace/projects/jarde/crates/jarde-reader/src/signature.rs:595)）；本数组匹配器只数出“至少一个且全 Any”，也没有 leaf 声明的元数事实。因此本片已拒绝的边界是已列出的形状不匹配，**未知 leaf 的真实 generic arity 仍未证明**。例如物理擦除为 `java.lang.String[]` 的 Signature `String<?>[]`，或物理擦除为 `java.util.Collection[]` 的 `Collection<?,?>[]`，可能通过当前擦除关系和全 Any 形状比较，却不是可编译 Java 类型。此共享 Signature 拼写债务单列于 [flat-signature-arity-debt-v1.md](/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-reifiable-wildcard-array-return-signatures/results/flat-signature-arity-debt-v1.md)；不扩本片为通用类型元数系统，也不把此情形写成当前已拒绝。

## initializer 结构与 Java 赋值分层

当前子数组证明在 [build.rs:13433](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:13433) 后组合 parent/child candidate；`array_initializer_reader` 通过唯一 reader 与精确 expression interval 闭合消费窗口（[build.rs:13688](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:13688)）。它产生结构/ownership 事实，不直接声明 child 值可赋给父 component。Builder 的 `array_initializer_element` 独立检查实际 `aastore` component、值的 presented type 与 null/exact/Object 等规则；否则仅接受既有 array/platform/scalar 或该 store BCI 上 source/target 完全匹配的 snapshot widening（[build.rs:26673](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:26673)、[build.rs:29434](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:29434)）。unknown/non-reference presented 值和没有相容证明的 reference component 会使 Java initializer 呈现失败。于是 child 结构可闭合而类型门拒绝；不能把 candidate/source 存在说成 Java 正文成功。

静态检查没有发现 child structure 直接覆写 component 类型门，也没有发现这片 wildcard 声明分支跳过 Builder。尚未由本报告独立执行的项目包括完整六类双 JDK 新候选 fresh replay 和真实非法 Signature arity 控制；它们不应从旧 CI、旧 CLI 或上述 unit helper 的通过推导出来。Root 正在进行五源 CLI。
