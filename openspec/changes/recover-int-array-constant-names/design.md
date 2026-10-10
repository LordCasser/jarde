## Context

见proposal及 `openspec/evidence/java-syntax-2026-10-10/array-literal-boundaries-next` 的真实8腿对照和两份extension audit。现有候选限定同类完整字段表中唯一static final I ConstantValue；report只改switch labels与选中arm的直接int return，facade用case/return needle定位。ordinary array return既未保留AST，也没有可表达的元素范围。

## Goals / Non-Goals

**Goals:** 最小扩展既有整数名称投影，让直接返回的一维int数组的直接typed-Int literal可呈现为唯一字段名；每个名称有确切输出span和Field/MethodPoint来源，完整类语义保持。

**Non-Goals:** 不改变现有switch范围与行为，不建立第二constant table/typed RHS tree/global pass，不改公开schema或依赖；不处理this/constructor字段提升、跨类或继承搜索、long extrema、数组长度/嵌套数组/算术/cast/调用内常量名。本片不复制JADX GPL代码，参照目标与算法证据自行复用Rust架构。

## Decisions

1. **复用同轮AST及候选。** 候选非空时，保留原switch capture，额外允许含newarray opcode的普通方法沿同一opaque AST通道保留；真正准入由descriptor精确`)[I`及AST `Return(NewArray { element:Int, lengths:[], initializers:Some, total_dimensions:1 })`决定。只检查顶层直接Return及其直接typed-Int Integer元素，不全树替换整数。已命名或不合形状的叶保持原样，无替换就无投影。

2. **保持唯一值和词法范围条件。** 延用完整字段表、原flags/ConstantValue、可spell标识符、唯一name/value和方法所有parameter/local/for/resource/catch占名过滤。相等整数无法证明原源码名字，描述为唯一同类常量名称呈现；歧义与遮蔽保持literal。参数名不完整、AST/Code/执行不完整、fallback及当前body不再是原AST emission均拒绝新数组投影。已有member_texts占用必须拒绝，不能重放物理body覆盖派生body。

3. **精确范围复用既有replay。** `emit_class_source_statements`继续只commit文字。新增crate-private body-only适配器，用已有 `Emitter::replay`、`EvidencePhase`、`SegmentPublication::Whole`对已发射body调用同一`stmts`，沿每字节核对并要求完整覆盖；无需RecoveryFacts/envelope，也不在commit收集Vec。为新数组use按(primary BCI,完整名称文本)匹配唯一Segment并验证物理owner、BCI存在与range不重复；同BCI同名多段/缺段/未完成就拒绝或Stop。这里匹配的是formatter给出的range，不用match_indices猜offset。

4. **不牵动旧switch映射。** switch label不是单独Expr leaf，不能假设普通source map提供名称segment。既有case/return use保持现有路径；只扩展内部handoff让新数组use携带可选body range，已有use为None。不得用一个新range规则强行替换所有switch处理。对应跨crate Rust内部适配器可变，公开JSON不增加种类。

5. **方法装配复用原writer。** 新数组range是纯statement body的offset，不能直接传要求envelope artifact的`projected_body_span`。在`ClassSourceMethod`整数投影接缝内复用原`Artifact { original.envelope, statements:body }`、`placed_artifact_span`与相同annotations/declaration/marker placement，给出对应method-relative range并逐字核名称。字段与literal physical MethodPoint仍走原DerivedProjection、source_text_with_method_projections；不造envelope parser、不扫描整类文本找名称。

6. **预算与一次发布。** ASTclone/rewrite、用途与临时map/范围/名称集合先计IrItems，扫描先计AnalysisSteps并poll，回放由原EvidencePhase计费，输出按既有OutputBytes；回放Stop须沿真实预算/取消停止路径传播，不以None吞掉。全部method texts、derived ranges和root text在local staging，完整class text一致性成立且最后poll后才发布。物理RecoveryReport/text/source_map不修改；任何新数组拒绝不虚构derived record。旧switch行为用原回归验证。

## Risks / Trade-offs

- 同值不是原token证明 → 唯一候选约束、源码对照与保守名称遮蔽；不引入JADX globallookup及magic数值threshold。
- replay临时完整segment表有额外成本 → 仅新数组名称投影按需启动，遵守原预算；不预先增加选择器或collector机制。
- 相同BCI重复叶有歧义 → 明确拒绝，构造重复名称但不同BCI的控制验证可接受路径。
- 新实例片修改同一report/facade文件 → Luna只准备隔离补丁；静态修复链6fd51a18确切CI先验收、实例片单独提交检查点后，root再顺序应用本片、重新冻结和验收，不能借另一产品CI。
- 本机低于20GiB编译守卫 → 准备与读审可继续，实际Rust gate由root在资源满足时执行；未跑不得勾实施任务。
