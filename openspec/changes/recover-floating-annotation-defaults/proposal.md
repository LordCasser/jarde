# 恢复注解成员的 float/double 默认值

`@interface FloatDefaults` 是 293 字节的普通 Java 8 注解类，四个成员的 `AnnotationDefault` 分别保存 `-0.0f`、最小 double 次正规值、正无穷和标准正 quiet NaN。原 class 与 JADX 的完整类经 javac、`java -Xverify:all` 反射，raw bits 四行相同；当前 Jarde 已正确写出 `@interface` 头，却静默省略四个 `default`。Jarde 的完整类也能编译，但反射首项返回 `null` 并使 runner 抛出 NPE。固定源码、class SHA、`javap -v -c -p`、完整三方源码与运行结果在 `../../evidence/java-syntax-2026-09-22/annotation-float-defaults/`。

reader 的 `ElementConstantTag::Float/Double` 和 `CpEntryKind::Float { bits }/Double { bits }` 已保存所需原始位模式；缺口仅在 `src/class_source.rs::resolve_default` 对 F/D 不构造私有 `MemberDefault`。因此在现有成员默认值词汇中增加两个原始位叶子，使用有界且精确的 Java 8 常量表达式拼写。有限值保留负零/次正规位模式；标准无穷和标准正 quiet NaN 使用已验证能作为注解默认值的 Java 常量；对无证明的 NaN sign/payload 保守省略整个 default。数组沿用已有全有或全无规则。reader 的属性事实继续可读；class-source JSON 只保留其现有物理成员身份，并不发布解析后的默认值树。

本项只修 `AnnotationDefault` 的 F/D，不扩字段 `ConstantValue`、方法体浮点 Push、普通注解使用位置或任意 NaN 重建。与 `recover-floating-point-constants` 共享的是纯位模式到 Java 字面值的规则，避免另立浮点 IR 或新的 classfile 解析；该 change 的运行时运算/NaN 折叠证明仍是独立任务。
