## Context

见 [proposal.md](proposal.md) 与 [类型使用证据](../../evidence/java-syntax-2026-09-22/type-use-annotations/README.md)。reader 的 release registry 已认识两种 `Runtime*TypeAnnotations` 名字，却没有读取其内容。类级及成员声明注解 change 已建立 `AttributeShell`、共享 `annotation`/`element_value` 读树、完整值拼写与原子拒绝；本项必须等成员 change root 验收再改同一 reader/class-source/facade 文件。原始 `TypeUseSubject.class` 429B，在 `FIELD`、`METHOD_RETURN`、`METHOD_FORMAL_PARAMETER` 各有一个运行时可见类型属性。原完整类/JADX 可编译，Jarde 完整类因另一处 runner 缺返回而编译失败；单独替换未编辑的 Jarde subject 证明三处反射丢失，不能把整套 Jarde 宣称为可运行。

## Goals / Non-Goals

**Goals:** 对非数组、无泛型路径的限定引用类型，在字段、返回和 descriptor 参数位置忠实呈现类型注解；JSON 同时提供物理壳、目标/路径、完整值、实际拼写或拒绝。原/JADX/Jarde 的对照说明必须区分完整类与隔离 subject。保持不带这些属性的类不增加注解内容读取。

**Non-Goals:** 外部 annotation `@Target`/`@Repeatable` 解析；基本类型、默认包单段名、数组维度、泛型内部层级的类型注解拼写；类继承/实现/类型参数、方法 receiver/throws、Code 局部变量与表达式的其它 target；方法体恢复和原有声明注解的通用双目标推断。上述不可证明处保留事实与拒绝，另立任务，不为本 change 新增类型系统或跨类 resolver。

## Decisions

1. **仅补一个有界的 type-annotation 包裹读法，复用既有注解体。** `attribute_facts` 在需要时读取 visible/invisible 属性的 `u2 num_annotations`，每项记录 `target_type`、完整 `target_info`、`type_path` 和现有 `ElementValueFacts::Annotation`；其注解值继续走同一 `read_annotation` 和深度/预算/取消规则。`target_info` 按 JVMS §4.7.20 的有限 0x00–0x4B 形状逐字节消费（包含不在本次拼写范围的 target），路径验证 kind/index 并要求精确属性末端；未知 tag、长度错、池项错为结构错误。保留 visible/invisible 与每属性内顺序，不把路径扁平化为文本。备选的第二套注解 parser 或外部 classfile 库会引入不同预算和事实语义，现有 reader 已有完整值树，故不采用。
2. **读取归属与声明注解并排，不凭前缀猜类型语义。** 字段/方法只把自身 `Runtime*TypeAnnotations` shell 交给同次按需属性读取；类级和 Code 里的同名 shell 不搬到成员。独立属性的内容失败继续使用已有 member 局部错误/stop 规则，原 shell 始终可见。`FIELD` 只接受字段，`METHOD_RETURN`/`METHOD_FORMAL_PARAMETER` 只接受方法；参数索引按 descriptor 位置（非 JVM slot）检验范围。其它合法 target 作为原始事实拒绝呈现，不当作无注解。类级/成员级共享的既有 parser 只是解析层；registry 的版本识别不等于 Java 源恢复，也不执行目标代码。
3. **用限定类型名内部的 Java 8 位置保证 type-only。** 对 `java.lang.String` 一类由 descriptor 证明的非数组引用名，只在最后一个限定段之前写 `java.lang.@A String`，而不写字段/方法/形参的普通 `@A java.lang.String` 前缀。后者对 `@Target({FIELD,METHOD,PARAMETER,TYPE_USE})` 会同时生成声明与类型属性，前者经 `placement-boundaries` 的 javac/javap/反射证明只产生类型属性。要求路径为空、名称可拼写且有可插入的限定段；primitive、单段名、数组、`$` 命名歧义与其它路径先拒绝。注解类型名、具名元素和值复用已验收的声明注解拼写，整条注解失败则不输出半条。多条不同类型按物理顺序输出；同一目标/路径/位置同类型重复在没有 `@Repeatable` 证明时保守拒绝。
4. **同位置跨平面冲突先拒绝，不复制。** 如果同类型的声明注解和类型注解同时属于字段/返回/某参数，独立写一个声明前缀再写一个类型内部注解可能让 javac 产生两条类型属性或报重复；本项不自行读取外部 `@Target` 元数据来选择合并。此类类型使用保留事实并拒绝新增内部拼写；声明注解按其既有 change 继续处理，验证其生成 class 的实际两平面属性并把无法完全保真的情况标为边界，不声称自动 round-trip。只含 type-use 的独立 fixture 闭合主验收。另一个方向——只有声明属性但外部注解兼有 TYPE_USE 时前缀可能新生类型属性——是成员声明恢复的独立债务，记录但不扩本次范围。
5. **验收以重编译后的物理属性和反射为准。** 原 `TypeUseSubject`、JADX、冻结修前 Jarde 仍保留三方可复现对照；修后用未编辑的 Jarde subject + 原 annotation/runner 做隔离执行，要求三行与原 class 相同且声明注解仍为空。若修后完整 Jarde runner 独立缺返回仍在，完整集继续如实记编译失败。可见属性用 `AnnotatedType` 反射，CLASS retention 用新旧 `javap` 的 owner/target/value/可见性对照；双目标位置与受控 primitive type-only 用独立边界脚本。正文、来源、预算和无属性类测试独立核对。

## Risks / Trade-offs

- **多数类型位置暂不能写** → JSON 保留精确 `target_info`、路径、值和拒绝原因；下一轮按独立可验证的位置扩展，不给错误 Java 源。
- **双目标注解会把声明和类型位置耦合** → 本 change 只在类型名内部写已证明 type-only 的位置，碰到同类型跨平面重合时拒绝并单列声明侧漂移债务；不从缺席属性倒推 `@Target`。
- **类源码方法体仍可能有独立编译错误** → 隔离 subject 的 Java8 编译、验证、反射是类型注解闭环；完整类编译状态照实报告，不用手改生成源码冒充整类通过。
- **reader 的目标格式比本轮恢复范围广** → 解析完整有限格式以免将合法非本轮目标误判为损坏，源码恢复只挑可证明的三种 target；二者分别测试。
