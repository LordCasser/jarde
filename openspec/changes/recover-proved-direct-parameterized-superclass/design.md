## Context

`ClassSourceDeclaration::project_generic_signature` 已从同一 class 读取 class `Signature`、调用 `prove_class_signature_erasure` 并为普通 `Parent<T>` 写出泛型头。它目前在 `parsed.type_parameters.is_empty()` 时提前返回，并对任何参数化 superclass 一律拒绝。`facade::prepare_physical_class_source` 在类头投影之前持有选定环境与当前物理定义，可复用现有物理解析接缝取得唯一父定义。固定 class 的 child `Signature` 为 `Ldt21parent/Parent<Ljava/lang/String;>;`，物理父名为 `dt21parent/Parent`。

## Goals / Non-Goals

**Goals:** 一个普通非嵌套 child、一个普通非嵌套父类，child 没有自有形参/接口/type-use 注解，父类有唯一无额外界的 `<T extends Object>` 且可证明准确选中。恢复 `extends Parent<String>` 并保留原构造、方法和物理报告。

**Non-Goals:** 多级继承、类型变量作为实参、参数化接口、多个形参、泛型成员类、方法继承代换、bridge 删除、泛型局部推断或从源码文本/名字猜父类存在。

## Decisions

1. **沿现有 class Signature 投影接缝。** 先保留 child 签名的完整解析与物理擦除证明；当无自有形参但有一段参数化父类时进入本片，而非早退。只接受一个 `String` 实参与准确普通父类形状，其他 class header 继续按已有路径。签名重复、错误、截断或 unsupported 时保持原 raw 头。
2. **父类形参需要选定物理证明。** 用当前请求环境唯一绑定 `super_class` 对应定义，从其真实 `Signature` 证明单个 `T extends Object` 与物理 `java/lang/Object` parent；若不在当前环境、来源歧义、签名缺失或形参数量不同则拒绝。不得仅凭 child `Signature` 拼 `Parent<String>`，因为目标可能其实是非泛型类或另一加载域的同名类。预算与取消贯穿选定读取，不开启全局类层级搜索。
3. **只原子改变类头。** 物理 `super_class` 是 JVM 继承身份，泛型 `Signature` 及选定父类形参决定源码实参；现有构造方法恢复保持独立，不把验证 class 头的工作扩成构造体证明。不修改方法 body、不推断 inherited return。仍用现有 `class_declaration_with_types` 与物理 class identity，拒绝不产生半个泛型头。
4. **正反例可重复。** 重放固定 class/JAR、原/JADX/Jarde 完整源码与 Runner；用 verifier-valid 的错 arity、非泛型父、错误擦除、缺/冲突定义及预算/取消控制证明拒绝或停止。直接字段/方法行为保持不变。

## Risks / Trade-offs

- 父类 `Signature` 属于选定环境中的另一个物理定义，跨类读取必须保持同一预算、definition identity 和加载域，不能隐式回落到同名 JDK/工作目录类。
- `Signature` 与 Java 8 泛型形参绑定本身不改变 JVM 运行值，但错误投影会令源码不可编译或反射不同；因此以完整重编和反射验收。
- 将来多层父类和 bridge 需要额外闭合代换与物理转发，不能从此首片直接推广。
