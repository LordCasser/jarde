## Context

调用层先前已让 `MethodFacts::parameter_types` 保留真实引用/数组描述符，并用 `Expr.presented` 与被调用 Methodref 的参数 `Type` 决定 `cast_argument`。`invocation_argument` 当前仅接受引用同型、目标 `java.lang.Object` 和 null；不同名数组一律拒绝。它不读取外部类层级，这对任意继承转换是正确的保守边界，但数组有一部分子类型关系由 Java 语言规则和数组形状本身封闭给出。

`array-reference-conversion/` 的 source-only 核心类为 1632 B/16 Code、SHA `549e371ac98362c3ca0d0c992e7a7cb2292a90a5f9b46ac58ee468338dd9ae0d`。28 项原 class 与 JADX 完整编译/执行逐字节相等；冻结 CLI `ecab8244…` 在三处协变调用拒绝、整类 javac 失败。较宽的37项类含另一个 `new Object()`/数组写入恢复问题，只作边界。另一个 source-only `ArrayReferenceOverloadTarget` 通过 `Object[] widened = value; return overload(widened);` 产生 `astore; aload; invokestatic overload(Object[])`，无 `checkcast`；同类还有更具体 `overload(String[])`。jarde 把该局部呈现为 `String[]` 并拒绝调用，说明新的上溯准入必须同时固定目标参数类型；无需在本案更改局部声明推导。

本地 JADX `2fb1b163` 的 `TypeCompare.compareTypes` 逐层比较数组组件，`compareArrayWithOtherType` 单独接受数组到 `Object`；`TypeUpdate.invokeListener`/`InvokeUpdateCallback` 将方法形参传给 SSA 类型更新，而 `InsnGen.generateMethodArguments` 最终直接打印已推断的实参。可借鉴其「数组组件递归 + 调用形参事实」的落点；不移植其外部类层级查找或未知类型宽松更新。本案还在调用消费处显式固定 Methodref 形参类型，因为局部源级类型与字节码调用目标可以不同，仅证明转换可用并不足以保证 Java 重载选择不变。JADX 当前目标选择样本可执行一致，这是观察结果，不作为省略目标 cast 的普遍保证。

## Goals / Non-Goals

**Goals:** 恢复仅由数组结构和 Java 内置数组超型可证明的引用上溯调用，并保持真实重载目标与原数组对象。

**Non-Goals:** 不推断外部 `Integer[]→Number[]` 等类层级关系、不从 Methodref 目标反推未知源类型、不补一般局部声明类型、赋值或返回的数组协变、不恢复数组写入的独立构造/复制缺口。

## Decisions

1. 只读两个已呈现的 Java 类型与真实 Methodref 参数 descriptor，定义有界的纯数组上溯判据：递归比较数组组件；相同原始组件可保留，原始组件不能转成引用组件；相同引用组件可保留；任何数组都可上溯到 Java 内置的 `java.lang.Object`、`java.lang.Cloneable`、`java.io.Serializable`。`int[][]→Object[]` 与 `int[][]→Cloneable[]` 成立，因为其组件 `int[]` 是数组引用；`int[]→Object[]` 和 `int[]→Cloneable[]` 不成立，因为其组件 `int` 是原始类型。`int[]→Cloneable` 则是数组整体的内置关系。实现可剥离已有 `[]` 拼写或复用现有 descriptor 事实，不新增公开 Type 层级或 resolver。每剥一维须受 Java 数组 rank 上限/既有预算约束。
2. 对非同型而已证明的数组转换，`invocation_argument` 继续使用 `cast_argument(argument, required, bci)`。这是 Java 中对**已证明上溯**的目标类型声明，不由 descriptor 单独发明可能失败的窄 `checkcast`；Cast 保持原实参来源、附加调用 BCI 的 derived 来源，且只求值一次。即使当前重载集合恰好直接选中正确方法，也固定目标，防止另一重载使相同文本改变目标。
3. 缺少源类型、目标不是上述内置数组超型、源/目标原始组件不兼容时维持现有来源完整拒绝。内置 marker 接口使用精确全名，不据此推断用户接口或外部层级。现有 null、目标 `Object`、同型、数值和函数式目标路径不变；赋值、返回、数组写入仍用自己的消费合同，不把 `meeting_position` 改为通用引用子类型器。
4. 普通 `String[]` 经 `Object[]` 视图写入不相容元素仍可抛 `ArrayStoreException`；本项只保留引用对象及调用目标，不复制数组、不改变其运行时 component type。核心 fixture 验证目标字符串与 null/零长控制；若扩展此行为对照，用源辅助方法而非引入 `new Object()` 的独立构造缺口。
5. 不增加依赖。JLS 对数组的封闭规则足以覆盖本批样本；任意类/接口层级需要其它证据，应拆案。一个小型纯判据比新求解器或 classpath 搜索更容易审查、计费和维护。

依据：[JLS 数组子类型](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.10.3)、[widening reference conversion](https://docs.oracle.com/javase/specs/jls/se8/html/jls-5.html#jls-5.1.5)、[重载选择](https://docs.oracle.com/javase/specs/jls/se8/html/jls-15.html#jls-15.12.2)、[JVMS 8 字段描述符的 255 维上限](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.3.2)。

## Risks / Trade-offs

- 错把 `int[]` 当作 `Object[]` → 原始组件不接受引用组件规则；正例需区分 `int[][]→Object[]` 与负例 `int[]→Object[]`。
- 只放行而不固定 Methodref 目标 → `Object[] widened = String[]; overload(widened)` 会被新文本选成 `overload(String[])`；目标选择样本必须执行比对。
- 把数组 upcast 当作任意引用 cast → 保留未知接口与外部类层级拒绝，不生成可能改变异常的运行时检查。
