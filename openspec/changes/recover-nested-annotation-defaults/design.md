## Context

reader 的 `ElementValueFacts::Annotation { type_descriptor, elements }` 与 `ElementValuePairFacts` 已按字节顺序保留嵌套类型、成员名和子值；`Array(Vec<ElementValueFacts>)` 也可递归包含它们。`src/class_source.rs::MemberDefault` 已能拼写常量、enum、class 和数组，`resolve_default` 却对 `Annotation` 明确返回 `None`，使整个嵌套数组的默认值也随 all-or-nothing 解析被省略。`header-minimal/nested/` 有合法源码、两类完整 class、JADX 与反射 runner；当前 Jarde 另有独立非法 `@interface extends Annotation` 头，必须先修复再作执行验收。

## Goals / Non-Goals

**Goals:** 对合法 Java 8 属性树递归产生嵌套注解默认值，保留成员顺序和现有原始属性事实，完整类反射行为与原 class 相同。

**Non-Goals:** F/D 特殊常量、runtime annotations 的类/字段/方法使用位置、泛型/层级解析、enum 声明修复、class 头修复。不能因为这些邻项仍缺就把本项扩成通用注解系统。

## Decisions

1. **扩展现有私有值词汇。** `MemberDefault` 增加一个注解值，携带已拼写的类型名及按原顺序解析的 `(name, MemberDefault)` 列表。`resolve_default` 在 `ElementValueFacts::Annotation` 分支复用现有 descriptor 类型拼写和递归子值解析；不再解析属性字节，也不新建 annotation-use pass。
2. **拼写完整的 Java 8 注解使用。** 输出 `@Type(name = value, …)`；空列表可写 `@Type()`。所有成员名必须是当前 Java 标识符拼写可接受的名字，descriptor 必须是可写的引用类型；无法证明时返回 `None`。不猜 `value` 单成员简写，因显式成员对已可表示同样语义。
3. **数组保留现有原子性。** `MemberDefault::Array` 只在每个子值都完成解析时构造；一处失败时整段默认值不呈现，reader 的原始 `AnnotationDefault` facts 和输入 class 字节不变。当前 class-source JSON 不发布解析后的默认值树；本项不扩充公开报告。reader 既有深度与预算为递归提供边界，source 层不加第二套限制或状态。
4. **整类执行在头部修复后验收。** 用冻结 Nested/Inner 类和 runner，原/JADX/Jarde 各自完整编译并反射比较 `6`、`2` 及数组形状；不手改生成类，不能用单独方法文本断言替代运行。普通 defaults、空/多成员嵌套和无法拼写子值需定向回归。

## Risks / Trade-offs

- 只输出数组中可识别的前缀会改变默认值长度和反射行为；沿用全有或全无递归解析。
- 不核验成员名/类型就写出 `@Type(...)` 可使合法 class 的输出不可编译或异义；限制 Java 可拼写 descriptor 和标识符，对不确定的引用保持弱表示。
- 当前头部错误与 enum/F/D 缺口会遮住此项整类运行；先独立完成头部最小修复，专用 nested fixture 不含 enum/F/D。
