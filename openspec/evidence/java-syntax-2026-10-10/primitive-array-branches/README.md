# Primitive array branches

This fixture isolates the `test4(int type)` method from
`/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arrays/TestArrays2.java`.
It removes the JUnit/JADX integration-test scaffolding, keeps the private method's
branch body, and adds `choose(int)` as a public wrapper so the standalone Runner
can exercise the method without changing its visibility or branch behavior.

`PrimitiveArrayBranchesRunner` checks the returned primitive array type and
contents for selectors 1–4, and checks null for 0, 5, -1, `Integer.MIN_VALUE`,
and `Integer.MAX_VALUE`. It prints each observed value to stdout for raw-stream
comparison.

`prepare-baseline-luna-v1.py` prepares eight complete-source comparison legs:
two fresh original javac runs, four JADX runs (default/none source each compiled
on both JDKs), and two Jarde runs. JADX decompiles one javac23 jar containing
only `PrimitiveArrayBranches.class`; each generated source set is compiled with
the frozen Runner. Each Jarde leg renders the fresh original class with all
evidence and compiles the emitted source unchanged with the Runner.

This is a focused adaptation of one JADX fixture method. It does not represent
the full `TestArrays2` test class or completion of the broader EM18 test set.

## root 实际执行与独立验收

root审查后执行prepare-baseline-root-v2.py，baseline-root-v1实际为25命令/8腿：原双JDK2/2、fresh JADX1.5.6 default/none完整源码双JDK4/4、冻结Jarde2/2，全体exit/stdout/stderr一致。空CP/SP、fresh classes与-Xverify:all，生成正文原样重编，仅自己的Runner适配package。JADX两profile从同一fresh javac23单class jar各提取一次，不冒称每JDK独立提取。

results/verify-baseline-root-v7.py由root实际执行接受，结果baseline-verification-root-v7.json；99个闭合文件、完整三method/零field及所有原物理BCI来源核对通过。v2准备脚本schema误写、root v3 javap class brace假设、v4跨JDK常量池索引/owner注释逐字误比、v5某些case无冗余complete_class_set键、v6 derived origin实际为列表的错误已修正；未改原始基线/raw。物理BCI/opcode逐项核对，newarray element类型独立校验；常量池编号不是跨编译器身份。BLAKE3身份不当SHA-256，实际input完整class字节绑定SHA/length和CLI argv。

结论只限TestArrays2.test4适配片：当前Jarde已经覆盖，无需新增机制或实施change。EM18整单元和71分母不变。
