## Context

见 [proposal.md](proposal.md)。固定完整类在 `../../evidence/java-syntax-2026-09-22/member-annotation-uses/`。`MemberHeader.attributes` 已以物理跨度保留字段/方法自身的属性壳；`ClassSourceField::of` 当前只拿字段 `ConstantValue`，方法的 `MemberAttributes` 只含 `AnnotationDefault`/`Exceptions`。`spell_method`/`arguments` 根据 descriptor 的参数**位置**和 JVM slot 生成签名，后者仅决定名字/宽值占槽；运行时参数注解属性未解析。`recover-class-annotation-uses` 正在扩展类级 annotation 读树/拼写，本项必须在其验收后复用，不并行编辑共享实现。

## Goals / Non-Goals

**Goals:** 各字段、方法、参数只消费自身声明属性；一条注解原子拼写；参数索引与 descriptor 一一对应；完整类重编译后字段/方法/参数反射四行与原 class 相同；raw shell、文本、JSON、预算/取消状态可对照。

**Non-Goals:** 类级注解（二者另案）、type-use 或接收者注解、`MethodParameters` 名字/flags、外部注解类型解析、注解继承或处理器语义、方法体恢复修改。不修复其它数组元素静态类型问题；夹具用源码中真实显式 cast 避开那项独立债务。

## Decisions

1. **复用属性树，仅增加参数属性的包裹结构。** 字段/方法的 `RuntimeVisibleAnnotations`/`RuntimeInvisibleAnnotations` 使用类级 change 的同一 `AttributeFacts` 与 annotation 读法；参数属性另按 JVMS 的 `u1 num_parameters`、每参数 `u2 num_annotations`、各 `annotation` 解析到 `Vec<Vec<...>>`，保留 visible/invisible 区别、字节顺序与原始 `u1` 个数。内部每个元素仍调用已有限深度和预算的 annotation 读取，不写第二个 element_value 解析器，也不引新库。重复属性、长度错误、非法池项走现有结构错误；类级/成员级不同属性位置不能混用。
2. **成员交接仍是一次 read。** `src/facade.rs` 的字段循环把本字段 `MemberHeader.attributes` 的受支持 shell 及 `ConstantValue` 同次交给拼写；方法循环把本方法的注解、参数注解、`Exceptions`、`AnnotationDefault` 同次交给现有 `MemberAttributes`。只在确有相应壳时读取内容和惰性池，不追加类/方法体 materialize。`ClassSourceField`/`ClassSourceMethod` 的 JSON 与正文并列给出注解行、拒绝标记和原壳；原 `item`、body outcome、来源映射和其他成员声明语义不变。
3. **参数位置独立于局部槽。** 在 `arguments` 的 `.enumerate()` 位置用参数属性组 `position` 放 `@Type(...) ` 前缀；`slot` 继续只用于参数名及恢复正文。`long`/`double` 两槽不跳过 annotation group；varargs 最末参数的注解在 `T...` 前。若 `num_parameters` 不等于 descriptor 参数个数，因 synthetic/mandated 参数可能省略且本项不读 `MethodParameters`，整个参数注解组拒绝并标记，不猜左/右对齐。字段/方法本身的可拼写注解仍正常呈现。
4. **一条注解原子、同位置同类型重复拒绝。** 注解名、元素名和值使用类级 change 的同一受限拼写与 `MemberDefault` 词法。不可拼写的一条及同一位置重复类型的实例都不输出，理由写入标记；不同位置或其它类型继续呈现。类级、字段、方法和每个参数是不同位置，不能把不同位置的同类型当重复。`Deprecated` 标记属性、`Runtime*TypeAnnotations` 均不触发本项。
5. **真实执行与物理属性并验。** 使用 362B `MemberTagged.class` 及未改 runner，原/JADX/Jarde 各自完整 Java 8 编译和 `-Xverify:all` 四行反射。另用 wide 参数、varargs、运行时不可见属性和参数计数 patch 验证位置/拒绝；对不可见属性比较重编译 class 的 `javap`，不以运行时反射不可见推断遗失。

## Risks / Trade-offs

- **参数属性的 `num_parameters` 可少于 descriptor 个数** → 不依赖未读的 `MethodParameters` 或名称猜对齐，明确拒绝并把匹配形状先闭合。
- **成员方法的原声明和正文分别生成** → 注解前缀由结构化成员事实装配，绝不在已输出 Java 文本上做字符串替换；定向比较 `ClassSourceMethod::declaration`、`text` 和报告。
- **类级 parser 尚在实施中** → 本 change 先冻结独立证据与规格；共享代码实施排在类级 change 的 root 验收之后，避免两代理同时改 reader/facade。
- **raw 属性可读但源码注解不可拼** → 保留原壳与明确标记，不把语义拒绝升级为读取错误，也不输出半条注解。
