# 成员注解位置边界证据

`BoundaryTagged.java` 是合法 Java 8 输入：同一 `CLASS` 保留类型 `BoundaryMark` 分别标记字段、方法和参数，因此 classfile 使用 `RuntimeInvisibleAnnotations` 与 `RuntimeInvisibleParameterAnnotations`。`wideAndVarargs(JD[Ljava/lang/String;)I` 的参数顺序是 `long`、`double`、末尾 varargs；实例接收者占 slot 0，三个参数起始 slot 是 1、3、5。两个宽值占用相邻的局部变量槽，但注解属性仍按参数位置 0、1、2 编码。另一个方法、它的两个参数以及字段也复用同一注解类型，证明跨声明位置不是同位置重复。

运行 `python3 openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/boundaries/run_boundaries.py` 可重建全部结果。使用 `javac 23.0.1 --release 8 -g:none`；冻结 Jarde CLI SHA-256 为 `f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544`，JADX 为 1.5.6。脚本留存法律输入、两份 runner/tagged class、两个受控 patch、`javap -v -c -p`、Jarde 文本与 JSON，以及各命令的退出码。`summary.json` 记录源/class 哈希、classfile 属性内容偏移与长度、patch 说明和验证结果。

合法类及两个 patch 都通过 `java -Xverify:all`，`javap` 和冻结 Jarde CLI 对所有输入均返回 0。合法类运行输出 `7 / 7 / 0 / 3`：两个方法结果正确；由于注解是 CLASS 保留，反射看不到字段注解，参数反射仍按 descriptor 返回三组空注解数组。`javap-legal.txt` 显示字段、方法与全部五个参数位置的不可见注解；宽参数方法的 `Code` BCI 为 0–9，另一方法为 0–6。

脚本另外生成两个**手工 classfile patch**，它们不是 Java 源码编译结果：

- `duplicate-same-position` 将字段的同一条 `BoundaryMark` 编码复制一次，属性 annotation 数从 1 改成 2，`RuntimeInvisibleAnnotations` 内容长度从 11 改成 20。JVM verifier 与 `javap` 均接受，后者在同一字段位置列出两个相同注解。
- `parameter-count-mismatch` 将宽参数方法的参数注解属性从 3 组截为 2 组，同时把 `num_parameters` 从 3 改为 2 并更新属性长度 34→23；descriptor 仍有三个参数。JVM verifier 与 `javap` 均接受，`javap` 显示位置 0、1 而没有位置 2。此例可验证按属性 `u1` 计数与 descriptor 个数不符的拒绝路径，不应猜测缺少组对应哪一个参数。

所有输入可加载，因此本轮没有需要另作 malformed 内容 patch 的 verifier 边界。若后续测试需要验证损坏 annotation body，应单独构造并保留它的解析器/VM结果，不能把上述合法 classfile patch 当成损坏属性。冻结 Jarde 输出当前仍不拼写这些注解；这些输出是实施前的基线证据，不是修复验收。
