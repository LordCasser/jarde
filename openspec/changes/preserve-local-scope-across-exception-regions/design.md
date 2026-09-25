## Context

参见 proposal.md 的 Why。当前恢复器先由 Frame/SSA 与 `reuse::Plan` 给局部身份，再用 RegionPaths 对 `slot_uses` 求声明共同 region；若某次使用位于 fallback region，`declaration_region` 返回无声明计划。之后 Builder 仍在按 region 顺序输出时以全局 `declared` 集记录已声明身份，catch 局部离开词法块后不会从该集合移除。真实边界 class 的 BCI 17–19 handler 只写 slot 1 的一个赋值，BCI 20 却在 try/catch 之后读 slot 1；保护区含异常边，Region 将其拆入 fallback，最终 AST 把 slot 1 首次可恢复声明放在 catch 块里，再在外层输出 `return local1`。

该缺陷位于 jarde-java 的 Region → 局部声明规划 → AST 建造交界处。`src/class_source.rs` 的 `ClassSourceMethod::recovered` 将同轮 `RecoveryReport.text` 放入方法体，不重写局部变量，故类级拼装不应承担作用域修复。

## Goals / Non-Goals

**Goals:**

- 用一致的局部身份、region 路径及真实使用位置决定声明作用域；离开嵌套块时局部可见性随之结束。
- 让“可合法提升的声明”与“必须拒绝的依赖切片”成为规划阶段的明确结果，避免构建完 AST 才发现变量越界。
- 对拒绝结构保留原 bytecode、BCI/origin 和既有 refusal/stop 平面，不输出误导性的半个 try/catch 加越界读取。

**Non-Goals:**

- 不补齐异常 CFG、reaching-definition 或 Frame/SSA 中缺失的 throw-site 语义；若相关证据不足即走拒绝路径。
- 不扩展条件值表达式准入，不改 class-source 拼装、bridge 或 CLI 的实现职责，也不要求此变更普遍恢复 try/catch。

## Decisions

1. **把 Java 可见性作为声明规划的输出事实，而非 Builder 的遍历历史。** 基于已存在的局部身份与 region tree，为每个声明标出其 lexical owner，并为每个读取确认声明 owner 支配该读取的 region。considered alternative 是只在进入/离开 arm 时增删 Builder 的 `declared`：这虽能阻止 catch 局部泄漏，但不能判断跨 try 汇合读是否应提升，也不能处理分支写入的 definite assignment，因此不单独采用。

   无 LVT 时 `reuse::Plan` 目前把同一 slot 视为一个 `LocalVariable::whole`，而 `RegionPaths.catch_parameters` 仅记录 `u16 slot`。新定向回归证实这两者不足以认定 catch 参数逃逸：`p3_nested_try` 和 `p3_typed_catch::two_rows_with_two_handlers_are_two_clauses_in_table_order` 的内外/兄弟 handler 会复用 slot 1，当前 `declarations` 因 slot 相同误报逃逸。声明规划必须以 catch clause 的词法路径、handler 入口存储及 SSA 定义—使用关系区分绑定；只有同一个实际值越过 catch 范围才判逃逸。可以扩展现有局部身份/规划结果，不新增仅凭 slot 名称的全局机制，也不能把每个同槽访问无条件拆开：真实的 try/catch 后汇合局部仍须按共同值与 DA 证明提升。

   本地 JADX `2fb1b1638694` 的 `CollectUsageRegionVisitor` 逐条收集 SSA 变量在 region 内的赋值/使用位置，`ProcessVariables` 再按 `CodeVar` 合并，这是可借鉴的先按值身份再找词法 owner 的顺序。但其 `declareVar` 在找不到合适赋值位置时仍将变量声明放到方法开头，源码还留着“search closest region”的 TODO；此回退不能替代 Jarde 对 catch 可见性与 definite assignment 的证明。

2. **只在证据闭合时提升；fallback 会阻断跨越它的提升证明。** 局部读写都在同一 lexical region 时保留就地声明。若读写位于 sibling/外层 region，仅当共同祖先处的声明合法且所有相关写入和可能抵达读点的路径均被可执行 AST 覆盖，并能证明汇合前已赋值时才提升。含 fallback/未认领区块、跨越无法解析的异常边或不完整 slot 身份时，不把注释引用当成对 Java 局部的访问；改为在构建前计算依赖闭包并拒绝共同结构。

   `LoopTryHandlerEntry.loopTry(II)I` 给出了不能把“写入可呈现”简化成“栈值来自直接整数常量”的边界：BCI 1 的 `result = 0` 在循环与 try 前已初始化，BCI 13 的正常写入消费 `result + maybeFail(...)`，BCI 19 的 handler 写入 `-1`，BCI 27 在循环后读取。`build.rs::all_reads_reach_presented_writes` 当前虽追踪 SSA 定义—使用，却只准入 `Operation::Push(Int)` 的 store operand，故在入口区域已修复后仍整方法引用。实现应复用已有 Region/SSA 的作用域、表达式单次消费与 Builder 的写入呈现约束，证明计算值及其可能抛错的调用仍在受保护范围内、每条路径的赋值与读取有可编译的声明；不要仅扩大 opcode 白名单或把调用移到 try 外。若无法预证某次写入会生成语句，则在发布任何依赖读取前按决定 3 的闭包拒绝。原 class、JADX 和参数累加器隔离正例的冻结证据见 `openspec/evidence/java-syntax-2026-09-24/loop-try-handler-entry/analysis.md`。

3. **拒绝按定义—使用依赖形成原子切片。** 起点为不能安全安置声明的局部；切片包括支撑其值的 try protected range、相关 handlers、连接它们的汇合/transfer 块以及所有读取该局部的消费者。选择包含整个切片的最小可拒 region；若局部读取落在方法顶层，则提升到方法 body。考虑过无条件拒绝整个 method，但仅在最小安全 region 无法包住完整切片时才扩展到 method，避免掩盖与缺陷无关的已证明结构。

4. **在既有声明规划内完成一致性守卫。** `Declarations.at_region` 已经使用 `RegionPath` 规划共同词法 owner；不再对建成的 AST 或姓名做第二次全树扫描。扩展现有规划结果，核对提升声明覆盖每个已知读写、catch/resource 头声明只在其子路径可见，以及 fallback/未归属使用是否阻断该提升。跨异常路径的 DA 仍须由可证明的写入与边事实支持；证据不足时让所属 `Region::Try` 在建造前进入完整拒绝闭包。不能在 AST 生成后删文本来补救。

5. **分别验收合法恢复与拒绝证据。** 对含完整 try 与 handler 定义的样本，要求整方法 javac + 原类/输出类运行一致；对异常边 fallback 或交叉 handler 样本，要求相关方法/切片拒绝、所有相关 BCI/source-map origin 被记账、不可见名称不出现在被拒切片外。边界至少包含 catch 内局部、try/catch 后汇合变量、嵌套 try、共享 handler/多 catch、无 LVT、预算/取消停止。

   `Frame::arm` 继承外层 try 的 handler 所有权与 join 边界后，`p3_exception_scope` 的两种合法形状已能单独恢复并通过三方运行对照；`p3_nested_try`、`p3_try_local` 及 `p3_typed_catch` 的若干旧回归仍失败。前两种诊断分别是上述同槽 catch 参数误判，以及 `declarations` 遇到被引用 fallback 就拒绝跨区提升；在暂时撤回 frame 两处改动的独立 A/B 中失败完全相同。后者不能靠跳过 fallback 守卫解决：要先确认该 fallback 是否属于同一局部依赖切片，若是则保留完整拒绝并修复上游区域证据；若否，只阻断真正跨越该 fallback 的局部。把这一组回归作为独立规划债务处理，不混入 frame 或普通 switch 修复。

## Risks / Trade-offs

- [Risk] Java DA 对异常可能发生的位置敏感，粗略按 CFG 可达性 hoist 会生成 javac 拒绝的未赋值变量 → 仅对能够从 throw-site 和 exception-table 证据闭合证明的路径做提升，否则拒绝。
- [Risk] 拒绝闭包太小会留下同一变量的外部 consumer 或局部声明，太大则损失无关可恢复代码 → 以局部依赖切片找最小共同可拒 region，并对无包围 region 的情况显式用 method body 作边界。
- [Risk] 新校验可能在大 region trees 上增加工作量 → 对访问、队列和检查步骤使用当前 run 的 IR/output 预算与 cancellation；停止必须维持现有 Stopped/无部分产物语义。

## Migration Plan

1. 先加只读边界测试并以旧 CLI 留存的 javac 错误确认证据来源。
2. 实现声明 owner/作用域校验与不完整定义的拒绝闭包，先跑局部恢复定向测试。
3. 构建新 CLI 后重放原、JADX 与 Jarde 完整类；合法样本执行逐行对比，拒绝样本核对 refusal coverage。无存储迁移；失败可通过回滚该恢复逻辑恢复旧行为。

## Open Questions

- 现有测试 harness 能否对 `Mixed` 方法仍生成可执行等价的局部结构，还是拒绝结果只承诺 bytecode/origin 覆盖？此问题不改变本 spec：不要将被拒字节码声称为语义等价 Java；实现时按既有 representation/verification 契约明确标注。
