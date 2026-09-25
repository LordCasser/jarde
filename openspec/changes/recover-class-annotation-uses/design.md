## Context

见 [proposal.md](proposal.md)。修前 `ClassMemberFacts` 只保留字段/方法的属性壳，类级属性壳虽能由其它 header 视图读取，却**未在 class-source 使用的同次成员读取中发布**；因此本项先在这一结构读取完成 fields+methods 后，按物理顺序以现有 `read_attribute_shell_records` 纳入 class `attributes`，当场计费壳的 `AttributeBytes` 与 `ResultItems`，内容仍按需读取。`attribute_facts` 目前只解析 `AnnotationDefault`，`RuntimeVisibleAnnotations`/`RuntimeInvisibleAnnotations` 被跳过。现有 `read_element_value` 已有全部 Java 8 标签、嵌套数组/注解、32 层上限、budget poll；`class_source::resolve_default` 已能对具名值作受限源码拼写。`ClassSourceDeclaration::of` 只收公开 `ClassDeclarationItem`，该项不携带属性壳，因而类注解必须从扩展后的同次 `read.facts.attributes` 交到源码装配，不能从声明文字或方法体反推。固定证据在 `../../evidence/java-syntax-2026-09-22/class-annotation-uses/`，三套完整类均编译和执行，差异仅是两个类级注解使用。

## Goals / Non-Goals

**Goals:** 单次、可计费的类属性读取；完整无参/具名值注解的忠实拼写；类源码文本与 JSON 并列给出拼写、拒绝及原属性壳；编译后反射/原始属性对照。

**Non-Goals:** 字段、方法、参数、type-use 注解；外部注解类型解析或注解处理器语义；任意类路径加载；从 `Deprecated` 属性、注解类型名、普通字节码推断注解；改动方法 IR/AST。`recover-floating-annotation-defaults` 的数值词法规则可供 F/D 元素复用，未被该 change 验收的位模式仍拒绝。

## Decisions

1. **复用读树，扩展两个类级属性。** 在现有 `AttributeFacts` 增加运行时可见/不可见两组有序 `ElementValueFacts::Annotation`，把 `read_element_value` 的 `@` 分支共享成一个读 annotation 体的私有函数，类级 `u2 count + annotation[count]` 调用同一函数。每条属性必须读到精确末端并遵守现有 `attribute_content` 计费、递归上限与取消；重复同名属性仍由 `ensure_unique` 报结构错误。类级缺注解时不读取属性内容。noak 已负责物理属性壳；另引依赖或复制第二套值解析都会增加预算/来源分歧，故不做。
2. **沿同一类读取装配，不扩 `ClassDeclarationItem`。** `ClassMemberFacts` 在本次成员读取中补齐类级壳，`Engine::class_source` 从 `read.facts.attributes` 筛选两种壳并交给 `attribute_facts`；有注解且没有 prepared pool 时，现有常量池惰性读取谓词增加这一条件。类级注解只解析一次，不通过 `list_members` 或第二次 materialize。`ClassSourceDeclaration` 增加属性壳、可拼写行和拒绝标记三个并列字段；原 `item` 的类头/物理身份不变，`source_text` 在 `package` 后、类头前写这些行。JSON/文本来自同一计算结果；输出预算覆盖最终文本及报告。
3. **一条注解是原子提交单位。** 类级 annotation 的类型与每个具名元素用已存在的 Java 标识符和描述符检查；每项值走 `resolve_default` 同一受限拼写路径。全条成功才写 `@Type(name = value, ...)`，零元素写 `@Type`。同一注解类型出现多次时，没有注解类型解析不能证明它可重复，因此该类型的所有实例都保守拒绝并标注；其他类型的已证注解不受影响。拒绝只表示源码无法忠实拼写，不能把 reader 已读出的属性说成不存在。`Deprecated` 标记属性不参与。
4. **保留属性自己的顺序，明确跨属性限制。** 每个属性内的注解/元素按原字节次序写；两个属性的条目按物理属性壳顺序写，供复现。类文件已把 visible/invisible 分成独立属性，没有足够事实证明原源码跨组的写法顺序；不宣称恢复原始排版。异常值、非 Java 名字、原始位无法拼写时不通过宿主浮点转换、常量折叠或文本猜测归一化。
5. **解析停止不伪装为无注解。** 读属性前/过程中预算或取消、长度错误、CP 类型错误按现有 execution/diagnostic 传播；类声明有真实属性壳但没有完整解析结果时不输出半条注解。不要为语义拒绝构造 reader 错误，也不要让某一条语义拒绝吞掉其它独立注解。

## Risks / Trade-offs

- **跨 visible/invisible 源码顺序不能证明** → 只承诺物理属性及组内顺序；运行语义/属性内容验收，不宣称原排版。
- **注解类型未加载，重复性与元素类型无法外部解析** → 对同类型重复及本地无法拼写的值保守拒绝；不从名字猜 `@Repeatable`。
- **类注解使原本无需池的 annotation type 多一次池视图** → 只在属性确实存在时复用现有惰性池读取，测试预算和单次 materialize。
- **类头可编译不等于反射等价** → 验收固定完整类与 runner 的真实 `-Xverify:all` 输出，并对 `RuntimeInvisibleAnnotations` 比较 class 属性而非反射可见性。
