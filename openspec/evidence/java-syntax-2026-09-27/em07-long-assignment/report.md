# EM-07：实例 long 字段赋值结果差距

固定 JADX `jbc/TestDup2x1.java`（SHA-256 `ea4e2e910465126c71a887c1f21886f8d18bc39bab6237cbfc03c546fa4e1335`）的 Java 11 profile 断言 `this.value = v;`。该字节码并非 Java 11 专属：[input/em07/](input/em07/) 的 Java 8 `return this.value = input` 经 `javac --release 8 -g:none` 同样生成 `aload_0; lload_1; dup2_x1; putfield value:J; lreturn`，物理指令见 [javap](baseline/javap-Assignment.txt)。它是字段赋值表达式，不是 EM-19 数组访问。

[replay.py](replay.py) 对固定 JADX、原始源码和 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 做完整类源码对照。原始与 JADX 以 Java 8 重编、`java -Xverify:all` 均输出：

```text
4294967297:4294967297
-1:-1
```

JADX [源码](baseline/source/jadx-Assignment.java) 选择 `this.value = j; return j;` 两句。Jarde [源码](baseline/source/jarde-Assignment.java) 将 BCI 2 当作未支持操作，字段写入与返回值都因其 SSA 来源无法渲染；`setValue` 只有说明注释，完整源码重编报告“缺少返回语句”。[baseline/](baseline/) 保存工具版本、源码、编译和运行日志、CLI/输入 SHA-256。独立第二次基线重放的摘要与两份反编译源码逐字节相同。

现有 `jarde-jvm` frame/SSA 已识别 `dup2_x1` 的栈复制；缺口在 `jarde-java` 对复制的两个结果与字段写入/返回消费者缺少闭合证明。普通 `field_write` 和 `Return` 的结构化 AST/发射已经存在。按 [窄 OpenSpec](../../changes/recover-proved-long-field-assignment-result/)，只在直接 `long` 参数、本实例准确 `J` 字段、单块无 handler、复制值各有唯一指定消费者时把该字节码写成两句。固定 JADX 的输出证明首片不必引入通用赋值表达式节点；其它 `dup2_x1` 形式、任意右值及跨类字段留在各自验收单元。
