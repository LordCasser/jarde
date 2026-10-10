# Array field initializer baseline

This prepared Java 8 comparison follows JADX `arrays/TestArrayInitField.java`, method
`TestArrayInitField.test()`: a static `byte[] a` and an instance `byte[] b` remain field
initializers. Ordered `mark` calls inside the arrays, two scalar static markers around `a`, and a
constructor-body marker make initialization order observable; the Runner observes static state and
array contents before construction and after two constructions. `mark` returns `byte` so array
element typing is part of the target shape without introducing an unrelated narrowing cast.

The original Java class, fresh JADX default/none outputs, and frozen Jarde CLI `class-source` output
are each compiled as complete source with empty classpath/sourcepath and run using only newly built
classes with `-Xverify:all`. The original stdout, stderr, and exit from each JDK leg are the oracle;
there are no hard-coded expected outputs. No generated source is edited. A candidate compile failure
is retained and does not prevent later comparison cases from running.

Root 已实际执行 `prepare-baseline-luna-v1.py`，完整结果保存在 `baseline-root-v1/`。
每组 31 条真实命令、8 个完整类对照腿：Corretto 8/OpenJDK 23 原程序 2/2，fresh JADX 1.5.6 default/none 4/4，冻结 Jarde CLI 2/2 均完整重编并匹配原始三流。
两组由 [root 独立验收](../array-field-initializers-root-verification-v2.json) 核验，共 **2093 checks/0 errors**，包含闭合文件 hash、真实物理成员/flags 和逐方法 BCI。

显式非 final `trace = 0` 触发当前静态组的 constant-expression 阶段拒绝，整组保留 static 块，语义通过。修复见 [OpenSpec](../../../changes/preserve-nonfinal-static-initializer-phase/design.md)；实例 b 的跨构造器提升另见 [架构分析](instance-promotion-architecture-root.md)。两次构造后的原始输出含 Java int 溢出，直接保留 raw 为 oracle，不改预期以规避结果。
