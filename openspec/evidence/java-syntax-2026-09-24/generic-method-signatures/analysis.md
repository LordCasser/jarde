# Java 8 泛型方法声明审计

输入是 `GenericMethodProbe.java` 与独立 `GenericMethodRunner.java`。`javac 23.0.1 --release 8 -g:none` 生成的 subject 为 346 B、SHA-256 `928311a035b4716e846804dfcf78165a7958a7531da1f635ea28f040d54af2e3`。其 `choose` 物理 descriptor 是 `(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;`，同一成员的 `Signature` 是 `<T:Ljava/lang/Number;>(TT;TT;Z)TT;`。这是方法自己声明的类型变量，未依赖类泛型参数。

完整原类在 `java -Xverify:all` 下打印 `3\n1\n`：第一行是调用值，第二行是 `getDeclaredMethod(...).getTypeParameters().length`。JADX 1.5.6 输出 `public static <T extends Number> T choose(T, T, boolean)`，完整类用 `javac --release 8` 重编并以相同 JVM 选项运行，也打印 `3\n1\n`。当前 Jarde CLI SHA-256 `a3a29b59aa85c2d174b06033f874eb7d60a104dbccf1d20f5d88908273e92dd5` 输出擦除声明 `public static java.lang.Number choose(java.lang.Number, java.lang.Number, boolean)`；它能重编，运行打印 `3\n0\n`。三方正文保存于本目录；实际 class 与 `javap -v -p` 也保存。

失真不是普通方法体条件值：三方 `choose` 均只选择一臂，普通调用值都是 3。差异是类源码声明没有消费该成员自己的 `Signature`，导致 Java 编译器不再生成泛型签名，反射可观察的类型变量消失。更大样本的调用点还暴露了独立的 reference 参数转换拒绝，属于现有 `preserve-invocation-argument-types` 范围，不混入本项。

架构接缝已存在：reader 的 `AttributeFacts.signature` 按原字节暴露属性；`jarde-query/src/xref/metadata.rs` 已按 JVMS 4.7.9.1 解析签名语法以收集类型引用，但不保留可发射的类型结构；`src/class_source.rs::spell_method_declaration` 目前只用 descriptor、`Exceptions` 与 `AnnotationDefault`。应复用现有属性读取/预算及签名语法，把完整可证明且擦除后吻合物理 descriptor 的方法局部泛型类型呈现到类级声明。不能仅做 `Signature` 字符串替换或把不一致/缺依赖类型变量猜成 Java 源。方法体原恢复报告仍以物理 descriptor 为事实；类级投影另记来源和拒绝原因。

当前已知边界：含类级类型变量的签名还要求类头一并恢复，本项不可只改方法头；方法 type-use 注解、内类分段泛型、通配符、数组、throws 类型变量、桥接/合成方法、varargs 与反射元数据都需在准入/测试中逐一考虑。第一阶段只对 parser 能完整拼写、类型变量作用域与 descriptor 擦除交叉一致且方法体仍能按新声明编译的形状准入；其它物理成员保持既有擦除呈现并在报告中说明。
