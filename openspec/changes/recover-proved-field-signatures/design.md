## Context

见 [proposal](proposal.md) 和[三方证据](../../evidence/java-syntax-2026-09-24/field-generic-signatures/)。reader 已解析 `FieldSignature`，并有可复用的类变量第一边界擦除；类源码的 `ClassSourceField::of` 只按 descriptor 写类型。JADX 1.5.6 的 `SignatureProcessor.parseFieldSignature` 解析属性、展开变量、比较类型后立即更新字段节点。这个顺序可参考，但其 `validateParsedType` 在非冲突比较后即放行，未证明更新后的字段类型与本类方法体同样可编译。字段类型还会影响外部泛型调用方和 `getGenericType()`，所以解析、擦除及源码使用是三个不同门槛。

## Goals / Non-Goals

**Goals:** 在不新增泛型 pass 的前提下，恢复顶层普通类中没有本类字段引用的参数化、通配符和类变量字段；同一 class 的完整源码可重编，并让字段反射及泛型调用方与原 class 一致。对合法但互相矛盾的属性和正文留有精确拒绝。

**Non-Goals:** 不推断通过本类字段读写形成的泛型值流、重载绑定或 `<clinit>` 初始化表达式的静态类型；不恢复成员/局部/匿名类的外层变量作用域、嵌套泛型类型、type-use 注解路径或外部依赖闭包。它们分别记为后续语法或源码闭包任务，不把 JADX 的输出当作行为真值。

## Decisions

1. **reader 只证明字段擦除和作用域。** 沿 `parse_field_signature` 的树递归检查类型变量存在于已发布类作用域，并用既有 `erase_type`/`DescriptorKind::Field` 对比整个物理 descriptor。证明按预算收费并轮询取消。复用 reader 的单一语法与擦除实现；不复制字段专用 parser，也不加载被引用的类推断缺失变量。输出无需新 proof 实体，成功/失败即可交给源码层。
2. **源码层先做局部可拼写门。** `ClassSourceField` 使用现有普通泛型类型拼写器从结构化类型重新构造声明，不替换已拼好的字符串中的某个子串。字段来源需唯一、名称与 flags 可保留、无无法安置的 type-use 注解；类变量只从成功发布的类头获取。声明注解与 `ConstantValue` 仍沿原字段记录。候选、来源说明及输出预算完整满足后一次发布；失败保持 descriptor 声明并记拒绝。
3. **本类正文的首片安全门采用常量池 Fieldref 不存在性。** 在同一次 class 读取的常量池里，只要存在指向该类同名同 descriptor 字段的 `FieldRef`，就保守拒绝泛型投影；任意本类字节码字段指令都必须通过此类条目，因此不存在条目足以证明本类正文无法因该字段静态类型变化而失去 Java 可编译性。即使某条目未被指令使用也会拒绝，这是可解释的覆盖率损失，后续若扩大覆盖，应复用同轮 IR/AST 证明每处使用与泛型静态类型兼容。替代方案是像 JADX 一样先更新全局字段类型再靠事后编译发现问题，违反本项目原子发布与来源契约；首片不采用。
4. **选择在完整类读取内发布，不影响 query 或成员证据选择。** `src/facade.rs` 因字段 `Signature` shell 按需复用已解析的 pool，并把本次类头发布的作用域与字段物理身份传给字段候选。每字段投影只依赖本次 class 属性和 pool，不从 optional `RuleDetails` 或 serialized 文本推断。字段候选独立，失败不改变相邻字段和方法；预算/取消作为现有请求停止传播。成功请求的 essential/all 正文一致。

## Risks / Trade-offs

- **有些真实源码字段由构造器、getter 或 `<clinit>` 使用** → 首片会保守拒绝，即便它们实际可重编；用单独正文兼容证明任务扩大，不借纯签名猜测。
- **字段 Signature 擦除一致仍可能命名不存在的类型** → 保持现有 classpath 边界，只有依赖齐备的 fixture 算作重编验收；缺类问题单独记录。
- **类变量字段失去类头作用域** → 只使用成功发布的类头 proof；失败时字段保持物理类型并说明未绑定来源。
- **注解或初始化位置被重写后失真** → 有 type-use 路径或本类 Fieldref 时拒绝；声明注解、物理顺序及常量值仍按原字段路径写出。
