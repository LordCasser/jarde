# EM-07：实例 long 字段赋值结果差距

固定 JADX `jbc/TestDup2x1.java`（SHA-256 `ea4e2e910465126c71a887c1f21886f8d18bc39bab6237cbfc03c546fa4e1335`）的 Java 11 profile 断言 `this.value = v;`。该字节码并非 Java 11 专属：[input/em07/](input/em07/) 的 Java 8 `return this.value = input` 经 `javac --release 8 -g:none` 同样生成 `aload_0; lload_1; dup2_x1; putfield value:J; lreturn`，物理指令见 [javap](baseline/javap-Assignment.txt)。它是字段赋值表达式，不是 EM-19 数组访问。

[replay.py](replay.py) 对固定 JADX、原始源码和 Jarde CLI (`8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`) 做完整类源码对照。原始与 JADX 以 Java 8 重编、`java -Xverify:all` 均输出：

```text
4294967297:4294967297
-1:-1
```

JADX [源码](baseline/source/jadx-Assignment.java) 选择 `this.value = j; return j;` 两句。Jarde [源码](baseline/source/jarde-Assignment.java) 将 BCI 2 当作未支持操作，字段写入与返回值都因其 SSA 来源无法渲染；`setValue` 只有说明注释，完整源码重编报告“缺少返回语句”。[baseline/](baseline/) 保存工具版本、源码、编译和运行日志、CLI/输入 SHA-256。独立第二次基线重放的摘要与两份反编译源码逐字节相同。

现有 `jarde-jvm` frame/SSA 已识别 `dup2_x1` 的栈复制；缺口在 `jarde-java` 对复制的两个结果与字段写入/返回消费者缺少闭合证明。普通 `field_write` 和 `Return` 的结构化 AST/发射已经存在。按 [窄 OpenSpec](../../changes/recover-proved-long-field-assignment-result/)，只在直接 `long` 参数、本实例准确 `J` 字段、单块无 handler、复制值各有唯一指定消费者时把该字节码写成两句。固定 JADX 的输出证明首片不必引入通用赋值表达式节点；其它 `dup2_x1` 形式、任意右值及跨类字段留在各自验收单元。

## 修后闭环

`fixed/summary.json` 记录修后重放。输入源码 SHA-256 为 `Assignment.java=ef8ff2517c93c28fa38c6a638ab0daecad3d96bc2e69928fe8dcba7e3b543148`、`Runner.java=6b34b34058de73b7df5ea5d743c131416878880d8e83846d24a90c124eec18e5`；固定 Java 8 class SHA-256 为 `a868e3f37906c695d69e07058457805ba9533b6740ea007b2debb3e891ac63c6`。CLI SHA-256 `d41bcdca204c62d6add0ad93b1cf98c956e853a84a30688742e9d8502a4d2345`。重放中原/JADX/Jarde 都以 `javac --release 8 -g:none` 编译完整类与同一 `Runner`，再以 `java -Xverify:all` 执行；三个结果均为 `4294967297:4294967297` 与 `-1:-1`。完整来源分别保存在 `fixed/source/Assignment.java`、`fixed/source/jadx-Assignment.java`、`fixed/source/jarde-Assignment.java`，每方的编译/运行日志与工具版本也在同目录。固定 JADX checkout 仍为 `2fb1b16386941660fda07e9017285aec40fcb37f`。

`tests/p3_long_field_assignment_result.rs` 覆盖准确正例及错误 owner、`I` 宽度、第二个 category-2 消费者、额外 NOP、异常 handler。builder 检查五条物理指令与单块完整 CFG；SSA 中的 `dup2_x1` 只能读入 `aload_0` 和 `lload_1` 的值，且必须产生三个互异输出：receiver/value 两份分别只由 `putfield` 消费，另一份只由 `lreturn` 消费。字段计划和 class header 同时确认目标是当前类唯一非静态 `J` 字段。BCI 2 为字段句保留 derived 来源，BCI 3 与 BCI 6 各有 direct 来源；输出不重复字段写或返回。

两句作为一组预扣两个 `ir_items` 后才进入方法 builder。`output_bytes` 只差一字节的重放仍返回 partial class-source report，但文本只可能同时呈现这两句或同时没有；`ir_items=100` 与预取消请求在目标方法 artifact 完整前返回 incomplete，不会发布半个 artifact。对比预算前后执行状态见定向测试。额外的复制/栈重排、非当前类字段、其它参数形状、含副作用的右值和有控制流/handler 的方法尚未覆盖，继续作为独立验收形态，不在本 OpenSpec 中扩大。

代码验收通过：`cargo test --test class_source --test p3_field_increment --test p3_long_field_assignment_result`（75 + 3 + 3 项）、`cargo test -p jarde-java --test p3_conditional_field_writes`（1 项）、`cargo check --workspace`、`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-proved-long-field-assignment-result --strict`。固定重放参数和原/JADX/Jarde 运行结果记录在 `fixed/summary.json` 与日志中。
