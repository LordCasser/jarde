## Context

`src/class_source.rs::class_declaration` 已根据类 `ACC_ENUM` 写出 `enum`，但 `ClassSourceField::of`、`ClassSourceMethod::recovered` 和 `source_text` 仍逐成员使用普通类拼写。`src/facade.rs::class_source` 已一次准备该类、按字段/方法表顺序恢复每个 body，最后调用 `source_text`；原始 `ClassSourceReport` 发布每个字段、方法身份与方法运行结果。两组完整类证据在 `../../evidence/java-syntax-2026-09-22/enum-declaration/`：`Stage` 只有枚举初始化，`user-static-boundary/Measure` 在编译器前缀后还执行用户 `totalUnits = sumUnits()`。原/JADX 全类可执行，Jarde 全类在普通字段式常量声明处失败。

## Goals / Non-Goals

**JADX 算法对照。** 本地 `jadx` checkout（`2fb1b163`）的 `EnumVisitor.convertToEnum` 先取 `<clinit>` 的线性 Region 前缀，再由隐式 values 数组反查常量构造；`ClassGen.addEnumFields` 将提取的构造实参写在常量名后。这个“先定位构造链，再装配类级语法”的顺序可复用。其 `searchValuesField` 在多个候选间优先 synthetic 与 `$VALUES` 名，`removeEnumMethods` 会按方法形状消隐、甚至重命名自定义 `values`；`simpleValueOfMth` 只认 `Enum.valueOf` 调用形状，没有证明原方法参数被原样转发。随后 `DONT_GENERATE` 跳过物理成员，[已执行的辅助方法变体](../../evidence/java-syntax-2026-09-22/enum-declaration/helper-constructor-prefix-boundaries/)确实让重编后的 `values()` 顺序及 `valueOf` 异常行为失真。本设计因此把字段、所有隐式方法及 `<clinit>` 效果合为一个原子证明，未证时不消隐任何成员；JADX 的 Region 前缀仅作为候选定位，不能代替 SSA 使用关系、精确成员身份和完整 Code 检查。

[额外 `$VALUES` 读取反例](../../evidence/java-syntax-2026-09-25/enum-values-access/analysis.md)进一步确认 `EnumVisitor.fixValuesAccess` 会把用户方法的 backing-array `SGET` 改成 `values()` 克隆，导致 verifier-valid 原 class 的 `B` 在 JADX 重编后变成 `A`。因此证明不能只检查五个预期枚举方法的内部形状；同次运行还须核对其余 Code 对将被隐藏成员的使用，存在额外引用则整组拒绝。无需为这个不可直接用 Java enum 源码表达的变体新增通用投影机制。

**Goals:** 有限、可证明的两常量 Java 8 enum 完整类能编译并保持原运行结果；类级源码投影与物理字段/方法报告并列，任何未证明的成员不被吞掉；现有预算、取消和来源语义不后退。

**Non-Goals:** 常量专属类体、常量数目任意化、一般构造参数/泛型/注解、复杂 `<clinit>` 控制流、跨类合并、反编译所有枚举方言。主类的辅助 `$values()` 当前方法恢复失败，此 change 仅在**确证编译器模式**后由 Java enum 语义替代它；不因此放宽一般数组构造恢复。

## Decisions

1. **类级局部证明是必要的新机制，通用枚举 IR 不是。** 常量字段、构造器与 `<clinit>` 分属不同记录，任何单方法规则或 `ACC_ENUM`/名称过滤都不足以知道源参数和用户静态后缀。只在现有 class-source 装配期建一个私有、有界的 enum 投影计划；它消费同一 prepared class 的结构化字段、Code/IR 与既有恢复语句，避免重新读类、解析已发射 Java 文本、加 crate 或公开一个泛用左值/初始化框架。具体接缝是 `Engine::class_source` 现有逐成员 `recover_prepared_member` 路径：`jarde_jvm::analyze_prepared_method_ir` 返回的 `MethodIrAnalysis::ir()` 在送入 `recovery_presented` 前仍可借用，之后公开 `MethodAnalysisReport` 不再含块、BCI 或 SSA。类源码路径应在该借用点按既有 `Budget` 提取仅供本类证明的小型结构化事实，尤其 `$values()` 自身恢复可能失败时也要检查它的完整 Code；不能事后解析 `RecoveryReport.text`，也不能另做一次无预算分析。提取的事实与原始成员报告并行保存到本次装配结束，不扩公开 JSON schema。
2. **整组证明后才改变文本。** 要求类 `ACC_ENUM` 且合法父类；恰好两条表序 `ACC_ENUM` 本类字段各被 `<clinit>` 中一次 `new/dup/name/ordinal/source-int/<init>/putstatic` 初始化，name 等于字段名、ordinal 为 0/1、属主/描述符相同，所有消费唯一，前缀无额外效果。构造器须恰是隐式 `Enum(String,int)` 调用后的一次源整数参数字段保存，且没有其它未解释效果。`$VALUES` 字段/构造链、`values()`、`valueOf(String)`、合成 `$values()` 方法的**完整规范效果**均须逐一证明，不能按这些名字或 synthetic 标志单独删除。证明遇到分支、异常、额外使用或不完整成员读取就不投影。
3. **保留用户初始化后缀。** `<clinit>` 的已证常量和隐式数组前缀由常量列表承担，剩余已恢复语句按原始顺序留在 `static` 块；`Measure.totalUnits = sumUnits()` 是验收边界。当前 `RecoveryReport` 只有文本与 source-map，`report::recover` 的 `program.stmts` 在发射后即丢弃；因此 class-source 若要消费已证前缀，确需从这次现有 build/emitter 流程短暂带出仅该方法的结构化语句/BCI 归属，作为私有装配侧车，不写入公开报告。只凭字符串截取方法正文或删除整个 `<clinit>` 都会丢掉用户效果；投影按结构化语句的完整来源认领前缀，后缀由既有 emitter 对剩余节点发射并按同一预算收费。若无法证明完整语句边界和无跨边界依赖，整组拒绝。
   实施次序上，2.2 只投影 `<clinit>` 在已证 `$VALUES` 写入后仅剩正常 `return`、同次语句侧车也没有后缀的类，因此可原子地省略整条物理 `<clinit>` 的源码呈现；`Measure` 此时保持原样。2.3 才消费精确前缀并重新发射用户后缀。不得让 2.2 的 `Stage` 正例成为删除 `Measure` 用户静态效果的理由。
   2.3 的第一片可直接复用现有 `ClassInitializerCandidates::steps` 与 `emit_class_initializer_value`：`Measure` 的物理前缀结束在 BCI 34，余下为 BCI 34 `sumUnits()`、BCI 37 `putstatic totalUnits:I`、BCI 40 `return`；侧车中的后一条顶层 `FieldWrite` 记录写入身份、顺序和 RHS AST。先核对其原字段、赋值操作、来源与整个后缀的 Code/语句边界，再用既有表达式 emitter 一次生成 RHS，原子地把 `static { totalUnits = sumUnits(); }` 放在枚举常量之后。若后缀有其它未分类语句、跨边界依赖或表达式发射停止，就保持未投影的整组，不通过字符串截取旧 `<clinit>` 文本。
4. **报告保留物理事实，文本呈现源语法。** `ClassSourceReport.fields/methods` 仍按物理表顺序保留 `item`、成员 outcome 和原始 recovery report。常量的类源码用逗号列表一次输出，编译器隐式成员在类源码中无重复声明；成员级 `declaration/text/markers` 的文档与必要测试须说明投影后贡献及不呈现的原因，不能继续声称每个物理成员都成为普通字段/方法行。默认/完整 evidence 正文相同，底层 report/source map 不伪造 BCI。不要改 reader 公共 schema。
5. **与旧 delta 边界协调。** `present-proved-java-structure` 的旧枚举场景禁止把其局部 `putstatic` 规则直接合并到类头；那仍适用于单方法恢复和未获类级证明的类。实现前将旧活跃场景写清这一前提，本 change 只允许类源码投影在完整证明后消费同一链，保持两项要求同时成立。

## Risks / Trade-offs

- **误认标准辅助方法会吞用户效果** → 比较结构化 Code/SSA 的全部效果、成员身份与使用关系，任何差异整组回退；专设同名/篡改及额外效果反例。
- **投影改变报告的“成员文本直接拼接”承诺** → 物理记录不丢失，明确记录每项文本贡献和原始恢复结果；更新文档与 JSON/来源测试，不伪装为旧普通成员格式。
- **用户 static 后缀可能依赖编译器前缀** → 常量先由 Java enum 自身构造，后缀只在此前缀无额外效果且无跨边界临时栈值时投影；验证正常、异常和初始化一次性。
- **范围过窄** → 未证复杂 enum 保守呈现，另案审计；先让两组真实完整类闭环，不以放宽参数/控制流令这一 change 失控。
