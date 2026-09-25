# 混合 `&&`/`||` 的字段短路值：原件 / JADX / Jarde

[冻结 Java 8 源码、class 和 Runner](../../../../tests/fixtures/p3-conditional-values/mixed-short-circuit-field/README.md)由架构师重新以 `javac --release 8 -g:none` 重建，class 逐字节相等（SHA-256 `13deab71668ec6c26f1a636f5a646bc87156d9aa46444a5b1a1eac9896bcbe5f`）。`Runner` 在 `java -Xverify:all` 下遍历两种表达式各 8 组输入，分别记录字段值与 `b()`/`c()` 调用次数。原始输出见 [16 行轨迹](original-run.txt)。

`andOr` 的条件 BCI 1 `ifeq 10` 从左假跳到 `c()`，BCI 7 `ifne 16` 从 `b()` 真跳到 true producer，BCI 13 `ifeq 20` 从 `c()` 假跳到 false producer，BCI 16/20 分别生产 1/0，BCI 21 唯一 `putstatic result:Z`。`orAnd` 的 BCI 1 `ifne 10` 从左真跳到 `c()`，BCI 7 `ifeq 20` 从 `b()` 假跳到 false producer，BCI 13 `ifeq 20` 从 `c()` 假跳到 false producer，其余生产者/消费点同样为 16/20/21。完整反汇编见 [javap](javap.txt)。两例的第三次测试都有两个正常前驱；不是现有“每个后继测试只从前一个顺序边进入”的同极性链。

[JADX 1.5.6 完整类](jadx-MixedBooleanField.java)给出原来的两个表达式；[Jarde 实现前完整类](jarde-MixedBooleanField.java)在两方法均留嵌套空 `if`、重复消费点并报 `jre_region_loop`。[Jarde 全证据](jarde-report.json)两方法均为 `quality=fallback`、`representation=mixed`，未把缺字段写入误称为结构化 Java。Jarde 同时漏掉部分已解码生产者/转移 BCI 的来源，并在字段 rule-detail 中把未发射的 BCI 21 写成 `presented=true`，这些归逐指令来源、所有权、字段报告三个独立债务。

三份**完整类**分别用 `javac --release 8` 重编，再用同一 Runner 在 `java -Xverify:all` 下执行。JADX 与原 class 16 行逐字一致；Jarde fallback 文本虽可编译，却有 **8/16** 行字段值错误（均把应为 true 的结果写成 false），调用次数与原 class 相同。编译及输出日志为 `jadx-javac.log`、`jadx-run.txt`、`jarde-javac.log`、`jarde-run.txt`。JADX 为无包名 class 加的伪 `package defpackage;` 只在重编副本中去掉，类体未改。

本地 JADX `IfRegionMaker.mergeNestedIfNodes` 对两条条件路径找可合并分支，`mergeIfInfo` 用 AND/OR 组合；这为候选遍历提供线索。Jarde 不宜仅按路径相等拼文本：可以把现有 `ShortCircuitValue` 的有序测试列表推广为**小的无环决策图**，从 decoded target/canonical 正常边证明每个测试的两个出口、每个合流测试的精确前驱、两个 1/0 生产者、唯一同槽 Phi 和唯一 `putstatic Z`。从末端递归构造已有 `Conditional`，在互斥分支里重复文本允许，因为每条执行路径仍各调用 RHS 一次；必须为表达式深度、共享子图复制和输出大小设预算。真正额外入口、异常边、独立效果或第二消费者仍整体拒绝，不把 [JADX 在三元值嵌入 OR 上的误极性](../short-circuit-chain-extra-entry/analysis.md)复制进证明规则。该正向恢复应在通用来源与重叠所有权修复验收后独立实施。

该正向恢复现已独立实施并由主代理复验：[当前 Jarde 完整类](jarde-after-graph-MixedBooleanField.java)可用 Java 8 重编，[`java -Xverify:all` 的 16 条输出](jarde-after-graph-run.txt)与冻结原 class、JADX 逐字一致；[全证据](jarde-after-graph-report.json)中两方法均为 `structured/java`、无引用。实现只在原有 `ShortCircuitValue` 中保存有序测试的真实 fallthrough/taken 身份，按 canonical 前驱与 SSA/Phi/字段消费证明；没有引入新的公开语法节点。额外入口仍按独立[所有权拒绝](../short-circuit-chain-extra-entry/analysis.md)处理。上述“当前 Jarde fallback 8 行错误”是实现前基线，不代表现在状态。
