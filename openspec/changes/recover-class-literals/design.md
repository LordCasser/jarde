## Context

`decode::constant` 只把整数、长整数和字符串常量转为 `Operation::Push`，把 Class 项归为 `Other`；`build::constant`、`Expr::presented`、`emit::expr` 因而接不到可写的 Class 值。已冻结851 B/7行输入见 `../../evidence/java-syntax-2026-09-22/class-literals/analysis.md`：原 class 与 JADX完整编译执行相同，jarde 11处引用、五个缺返回。`java.lang.Integer.TYPE`/`java.lang.Void.TYPE` 是另一路字段访问，当前健康。

## Goals / Non-Goals

**Goals:** 只恢复 `ldc`/`ldc_w` 的已验证 Class 项；在引用类、数组类、本类以及调用参数位置保持 `Class` 身份和来源。

**Non-Goals:** 不折叠基本类型/void的 `TYPE` 字段，不实现浮点、MethodType/MethodHandle/dynamic 常量，不引入类加载或跨类解析，也不扩大类型限定符避让任务的范围。

## Decisions

1. `decode::constant` 只对 `CpEntryKind::Class` 且 Modified UTF-8 与池引用完整的项产生新的**类常量事实**，保留原池名与 index；按现有 MUTF-8 decoder 还原名称后，拒绝任何替换字符，再验证内部名或数组 descriptor。当前类型标识符采用既有 ASCII 命名子集，并拒绝含 `$` 的内部名：单类池项无法判断它需要嵌套类的 `Outer.Inner` 源拼写，还是可在另一编译/类路径条件下访问的顶层 `$` 名。Java 8 合法的非 ASCII 类名（包括 U+10400）也在本项保守拒绝，避免把新 Unicode 版本近似规则误当成 Java 8 的精确标识符规则。用 Java 8 `Character.isJavaIdentifierStart/Part` 表支持 Unicode、并处理二进制名与源名映射，属独立后续范围。不能写的类型保持 `Other`，并由现有引用保留指令。
2. 在现有 `ConstantValue → Expr` 链添加一个 Class literal 形状（可用字符串或已有 `Type` 携带已验证源拼写），`Expr::presented` 为 `java.lang.Class`，发射器写 `T.class`。这不是普通字段访问：`.class` 无字段解析或读写副作用，伪造 `Field {name: "class"}` 会混淆来源/类型；一个必要的表达式形状比增设一套类型/常量 pass 更小。
3. `.class` 是 Java 的类型上下文：root 的 Java 8 实编探针确认 `int ClassLiteralProbe` 与 `ClassLiteralProbe.class`、`int java` 与 `java.lang.String.class` 均可共存，局部变量本身不构成类型限定符遮蔽。真正风险是同名**类型**：同包 `class java` 含嵌套 `lang.String` 时，`java.lang.String.class` 实际取得 `java$lang$String`。当前单类输入能可靠识别的只是当前声明/已读成员的已知名字冲突，应在此边界内保守拒绝；未知同包旁类不能仅凭当前 class 证明不存在，仍沿系统现有类型路径政策处理，不在本项引入跨类 resolver。返回与调用参数照旧由消费者在最终位置呈现，`ldc` 只生产一个值，不插入独立语句或重复求值。
4. 直接来源锚定 `ldc` BCI 与 CP index，消费位置的派生来源沿现有表达式传播；新的节点/字符串输出按现有结果、深度、预算和取消计费。默认/all 的语句由同一 AST 决定。reader/SSA 不改，外部 Java/字节码库不能提供比本 class 池项更多的局部事实，不增依赖。

## Risks / Trade-offs

- 数组池名是 descriptor，而普通类是内部名 → 分别固定 `[[I`、`[[Ljava/lang/String;` 与 `java/lang/String`；错把 descriptor 直接写出会编译失败。
- 类型上下文中的局部变量不遮蔽 `.class`；同名类型却可能使原本看似全限定的路径指向另一类 → 固定正反成对 javac/JVM 探针，拒绝当前单类证据已知的冲突。旁类冲突在单类策略下不可证明不存在，记录为既有类型路径边界，不以本项新增查询机制；与 `preserve-type-qualifier-bindings` 的相邻规则回归。
- JADX 为无包类补 `defpackage`，本类 `Class.getName()` 会产生无关文本差异 → runner 对本类以对象身份而非名字比较，其他类型仍固定名字/数组层数。
- 现有 `TYPE` 字段虽然不是 `.class` 拼写但语义正确 → 保留其原有路径，后续形态优化须单独证明。
