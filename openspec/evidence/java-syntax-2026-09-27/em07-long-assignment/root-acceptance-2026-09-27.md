# EM-07 主线独立验收

Luna 实现提交 `d3e764fa420b377faebea2e384353ec7644797e9` 在主线 `568627b7` 合并后，root 构建 `jarde-cli` SHA-256 `9b8d04bef2954ec7b93253a218c78f799a5c9611f6685a462cd57eb7b71b8fa7`。独立执行固定 `replay.py fixed`，结果保存在 `/tmp/jarde-em07-root-accept/`；摘要除 CLI SHA 外与实现者的 [fixed/summary.json](fixed/summary.json) 逐字段一致，JADX/Jarde 源码逐字节一致。原源码、固定 JADX 和合并态 Jarde 的完整 Java 8 源码都能重编并通过 `java -Xverify:all`，共同 Runner 两行输出为 `4294967297:4294967297`、`-1:-1`。

实现只接受五条物理指令 `aload_0; lload_1; dup2_x1; putfield self.field:J; lreturn`、单块完整 CFG、准确当前类实例字段与一个 `long` 参数。SSA 证明复制前的 receiver/参数与复制后的三个互异值，分别由字段写入和返回消费。已有 `FieldAssign`、`Return` AST 足以表达固定 JADX 同样采用的两句结果；未新增通用赋值表达式。错误 owner/宽度、第二消费者、额外指令和 handler 负例拒绝，物理 class/report 仍可查询。两句在一次 `ir_items` 扣费后共同进入 builder；输出预算测试未看到半份赋值结果。

主线 `class_source` 82/82、`p3_field_increment` 3/3、EM-07 定向测试 3/3、`p3_conditional_field_writes` 1/1 通过；`cargo check --workspace --locked`、格式检查、严格 OpenSpec 校验和 diff 检查通过。任意右值、其它 `dup2_x1` 栈形及控制流仍需后续单独证明。
