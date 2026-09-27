## Context

[固定证据](../../evidence/java-syntax-2026-09-27/dt20-diamond-local/report.md)中 `-g` 的 LVT 与 LVTT 均声明 slot 1、名称 `map`、BCI `[8,20)`；LVTT 是 `Map<String,String>` 唯一来源。`jarde-reader::MethodCodeFacts::debug` 从同一 `Code` 内容读取 LVT，`facade::debug_locals` 传递范围，`jarde-java::names` 以物理 slot 和范围识别复用局部，`build::Declarations` 在生成语句前为 `LocalVariable` 定型。reader 已有 Signature parser/擦除证明，AST `ExprKind::New` 已有 `diamond` 位用于经过证明的成员类构造。

## Goals / Non-Goals

**Goals:** 对固定 `-g` 字节码证成一个局部 `Map<String,String> map = new HashMap<>()`，保持返回 cast 是否保留的既有决定。`-g:none` 必须仍按擦除事实写 raw/cast。完整方法及类源码继续通过 Java 8 重编与 verifier。

**Non-Goals:** 无 debug 泛型推断、任意集合/子类型关系、跨分支/循环局部、slot 复用、多次写入、构造器参数、嵌套类型、方法签名推断，或通过字符串替换已生成源码补尖括号。

## Decisions

1. **扩展现有同次 Code debug 事实。** 在 reader 读 LVT 的同一嵌套属性遍历中识别 LVTT，保留原始签名和 slot/名称/BCI 范围；只有唯一、完整、与 LVT 精确一致的记录可用于投影。畸形/重复/冲突记录不影响已读指令及名称，不被当作泛型事实。不得在 class-source 另读 `Code` 或从输出文本解析。
2. **局部身份先于类型。** facade 把该 raw 事实随既有 `DebugLocal` 交给 recovery；`reuse` 确定一个 `LocalVariable` 后，声明规划才解析 LVTT 的 `Map<String,String>`，证明其擦除是 LVT `Map`，范围覆盖唯一 store/use，且同轮 Code/SSA 只有已证的 `HashMap.<init>()` 对应的值写入。缺任一条件保持现有 raw 规划；已有 `Type::Reference` 可承载经过验证的源码类型，但不得绕过决定一次的契约。
3. **菱形只绑定确切构造。** 必须证成该 `new java.util.HashMap` 无参构造结果正是被赋给该局部的值，且 Java 8 标准库 `HashMap<K,V>` 可赋给目标 `Map<K,V>`；`ExprKind::New.diamond` 与局部声明在同一次原子 AST 决定中提交。不能把其它 `new HashMap`、raw/no-debug 方法或其它用途顺手变为菱形。若局部类型可证但构造目标不能证成，整个本片泛型投影拒绝。
4. **验证生产证明和对照。** 对真实 `-g`/`-g:none` class，检查原/JADX/Jarde 完整源码重编与验证运行。再构造 verifier-valid 的错范围、重复 LVTT、不同泛型实参、第二次写入及低预算/取消负例；不以唯一正例的文本断言替代闭合证据。

## Risks / Trade-offs

- LVTT 是调试属性而非 JVM 执行语义 → 只在其与 LVT/物理 descriptor/SSA 精确一致时用于源语法；缺失时保留 raw，不赋予其改变运行行为的权力。
- `HashMap` 到 `Map` 的泛型赋值不是当前 class 字节码的显式类型参数 → 此首片仅对 Java 8 平台的确切这对 JDK 泛型类建立关系，后续扩张必须有独立类型关系证据；不从任意同名类猜测。
- 作用域或预算内只有部分 debug 表 → 不写半个局部声明或半个 `<>`，报告保持同轮停止/拒绝来源。
