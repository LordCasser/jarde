# OpenSpec 规划入口

P0–P5 与分层已按各阶段范围归档：P2 29/29、分层 7/7、P3 12/12、P4 10/10、P5 10/10；主规格现为 **18 份**。2026-09-20 对 `cd6f2f0` 的独立复核确认的两条 P1——嵌套表达式旧值重算（P3-R8）与字段生产者在 fallback 中丢失（P3-R9）——已由 `close-recovery-correctness-gaps`（4/4，`fd0aae8`）关闭并归档，历史归档记录保留。见 [收尾验证](changes/archive/2026-09-20-close-recovery-correctness-gaps/verification.md)。

- [架构基线](../JVM_Rust_Engine_Final_Architecture.md)：目标、I1–I12、模型与管线。
- [当前路线](roadmap.md)：已完成范围、正确性收尾和后续覆盖边界。
- [依赖选型](dependencies.md)：复用决策与准入要求。
- [验收映射](acceptance.md)：A01–A18 的适用范围与证据。
- [当前完成复核](completion-review.md)：T1–T4 按约定四项判据关闭，三项归档及 CI 已核对；新发现 P1/T5 为 `append(int)` 消费 char 时丢转换，独立登记，不宣称 concat 全面完成。
- [2026-09-22 持续语法巡查](evidence/java-syntax-2026-09-22/README.md)：以同 class 的源码、jadx、jarde 及实际执行定位缺口；独立交付为 [一元取负](changes/recover-unary-negation/tasks.md)、[特殊调用分派](changes/preserve-special-call-dispatch/tasks.md)、[显式引用转换](changes/recover-explicit-reference-casts/tasks.md)、[数值比较条件](changes/recover-numeric-comparison-conditions/tasks.md)、[调用参数类型](changes/preserve-invocation-argument-types/tasks.md)、[静态初始化完成](changes/recover-static-initializer-completion/tasks.md)、[普通throw](changes/recover-throw-statements/tasks.md)、[final静态字段写入](changes/recover-final-static-field-writes/tasks.md)、[instanceof](changes/recover-instanceof-expressions/tasks.md)、[位运算](changes/recover-bitwise-expressions/tasks.md)、[浮点常量](changes/recover-floating-point-constants/tasks.md)。具体验收以各 change 为准，不从大 change 的历史任务数推断当前质量。
- 本轮待实施的独立语法任务还包括 [窄整数返回](changes/recover-narrow-integer-returns/tasks.md)、[窄数组写入](changes/recover-narrow-array-stores/tasks.md)、[boolean字段整数写入](changes/recover-boolean-field-stores/tasks.md)、[数组初始化器](changes/recover-array-initializers/tasks.md)、[条件表达式值汇合](changes/recover-conditional-values/tasks.md)、[do-while体内跳转](changes/recover-do-while-body-transfers/tasks.md) 与 [Class类字面量](changes/recover-class-literals/tasks.md)；[窄字段写入B/C/S](changes/recover-narrow-field-stores/verification.md) 与 [部分维度数组创建](changes/recover-partial-array-allocations/verification.md) 已通过根验收。先按各自证据闭合，再调整主规格。
- 数组引用的调用边界另见 [已证明的数组上溯与重载目标](changes/preserve-array-invocation-widening/tasks.md)，不并入数组创建或普通引用转换。
- 字符串 switch 的[语法形态恢复计划](changes/recover-string-switch/tasks.md)已有同 class 三方执行对照；现有两级 Java 行为正确，局部折叠属于较低优先级的独立质量任务。

| 阶段 | 记录 | 当前状态 |
| --- | --- | --- |
| P0 | [establish-p0-foundation](changes/archive/2026-09-17-establish-p0-foundation/proposal.md) | 已归档；reader、快照、身份与预算 |
| P1 | [p1-query-xref](changes/archive/2026-09-17-p1-query-xref/proposal.md) | 已归档；X0/X1 与物理视图 |
| 分层 | [layer-jarde-crates](changes/archive/2026-09-19-layer-jarde-crates/proposal.md) | 7/7，`7a5f994` 归档 |
| P2 | [p2-jvm-ir](changes/archive/2026-09-19-p2-jvm-ir/proposal.md) | 29/29，`7a5f994` 归档 |
| P3 | [p3-java8-recovery](changes/archive/2026-09-20-p3-java8-recovery/proposal.md) | 12/12，`250fe1f` 归档 |
| P4 | [p4-modern-semantics](changes/archive/2026-09-20-p4-modern-semantics/proposal.md) | 10/10，`88416ab` 归档；结构/推断，不是完整现代源码恢复 |
| P5 | [p5-measured-optimization](changes/archive/2026-09-20-p5-measured-optimization/proposal.md) | 10/10，`cd6f2f0` 归档；cache 默认 off，性能阈值未定 |
| 收尾 | [close-recovery-correctness-gaps](changes/archive/2026-09-20-close-recovery-correctness-gaps/proposal.md) | 4/4，`fd0aae8` 归档；R8/R9 关闭，反例与正向对照进入永久语料 |
| Benchmark 批次 1/4 | [expose-recovery-content](changes/archive/2026-09-20-expose-recovery-content/proposal.md) | 4/4，`7d095ce` 归档；`RecoveryReport.content` 三值闭合 |
| Benchmark 批次 2/4 | [carry-declaring-class-evidence](changes/archive/2026-09-20-carry-declaring-class-evidence/proposal.md) | 4/4，`618de49` 归档；driver class name/flags 同源交接 |
| Benchmark 批次 3/4 | [bound-container-lookup](changes/archive/2026-09-20-bound-container-lookup/proposal.md) | 8/8，`b22ea04` 归档；定向 container 访问与有界复用 |
| Benchmark 批次 4/4 | [bind-prefixed-load-roots](changes/archive/2026-09-20-bind-prefixed-load-roots/proposal.md) | 6/6，`fbcf06b` 归档；container+prefix root 与 CLI 树枚举 |
| 性能专项 | [optimize-demand-workloads](changes/optimize-demand-workloads/proposal.md) | 0/22；O1 子交付已归档，专项测量/归因/调查处置未完成；新候选须声明本次恢复缺口与修正范围 |
| 并行全量导出 | [add-parallel-bulk-recovery](changes/add-parallel-bulk-recovery/proposal.md) | 规划齐全，实现 0/22；类准备复用、有界多 worker、共享总预算和流式 JSONL，承接 O2/O4/O7；未声称已达到 jadx 性能 |
| 已证明的源码结构 | [present-proved-java-structure](changes/present-proved-java-structure/proposal.md) | 39/81；单臂 `if`、命名 `catch`（含 `try` 前的普通赋值）、`int` 数组与引用数组，以及 `package`/`throws`/`...`/标量转义已落地。声明拼写和数组还在各自的 worktree，没有并入主工作区。其余：前向汇合、`dup` 的两次存储、字段的 `x = x + k`、数组元素 `++`、比较的 0/1、数组元素宽度、比较与 `super`、静态调用的类型、取负、条件里的赋值、接口 `default`、常量字段、带标记的 `break`/`continue`、accessor/lambda/匿名类。验收是自写源码与 jadx、jarde 的三方对照 |
| 易用性 1/3 | [add-artifact-navigation](changes/archive/2026-09-20-add-artifact-navigation/proposal.md) | 8/8，`e0c83b6` 归档；候选/确认两级列举与身份交接 |
| 易用性 2/3 | [add-task-oriented-operations](changes/archive/2026-09-20-add-task-oriented-operations/proposal.md) | 11/11，`2428752` 归档；目标选择、阶段调度、预算与环境策略 |
| 易用性 3/3 | [add-task-oriented-cli](changes/archive/2026-09-20-add-task-oriented-cli/proposal.md) | 9/9，`85828c4` 归档；五个薄子命令、text/JSON 同源、四态退出状态 |
| 停止传播 | [preserve-task-operation-stops](changes/archive/2026-09-20-preserve-task-operation-stops/proposal.md) | 5/5，`8586356`；R1/R2 已关闭 |
| 递归停止 | [bound-recovery-recursion](changes/archive/2026-09-20-bound-recovery-recursion/proposal.md) | 已归档，`4f62e26`；用户指定三个方法不再 abort，返回停止报告 |
| 二元分组 | [fix-nested-arithmetic-value](changes/archive/2026-09-20-fix-nested-arithmetic-value/proposal.md) | 已归档，`445a277`；`inverse32` 原反例关闭 |
| Receiver 分组 | [group-call-receivers](changes/archive/2026-09-20-group-call-receivers/proposal.md) | 已归档，`fa6dc6e`；substring 原反例通过 |
| Boolean 上下文 | [type-boolean-contexts](changes/archive/2026-09-20-type-boolean-contexts/proposal.md) | 已归档，`5a8c36a`；原反例通过；其后 T2/T3 已由下列独立修正关闭 |
| 整数比较 T2 | [decide-comparison-contexts](changes/archive/2026-09-20-decide-comparison-contexts/proposal.md) | 17/17，`f4d1044`；原判据关闭 |
| 局部类型 T3 | [unify-local-type-decisions](changes/archive/2026-09-21-unify-local-type-decisions/proposal.md) | 21/21，`2895bc4`；原判据关闭，类型忠实度与通用赋值边界保留 |
| concat / 深链 T4/T1 | [re-express-string-concatenation](changes/archive/2026-09-21-re-express-string-concatenation/proposal.md) | 22/22，`5c35cbf`；原判据关闭；新 T5 参数转换及 CLI 文档预算债务单列 |
| 数组类型 | [spell-array-types](changes/archive/2026-09-20-spell-array-types/proposal.md) | 已归档，`66bd2d0`；原反例通过 |
| 报告读法 | [clarify-structural-output-planes](changes/archive/2026-09-20-clarify-structural-output-planes/proposal.md) | 11/11 已归档；Produced/content、平面证据与计数口径明确 |

本仓库使用 OpenSpec 1.11.0 的 `spec-driven` schema。`specs/` 包含 P0–P5 的 18 份主规格；收尾 change 的两份 delta 已随归档并入 `java8-recovery` 与 `recovery-validation`（各保留原有 scenario，并各补两条反例/验收 scenario）。`isPlanningComplete`（旧字段 `isComplete`）只表示规划工件齐全，实施以 tasks、代码和验证为准。

```sh
openspec list
openspec validate --all --strict --no-interactive
openspec validate --archived --strict --no-interactive
```

A15/A18 的历史已验收路径保留；不存在的 index/parallel/merged 为不适用，不因此重做 P5。R8/R9、R1/R2 及 T1–T4 的原判据均已关闭；T5 和其它边界按[路线](roadmap.md) 独立处理，不把原反例关闭外推为所有形状通过。MethodParameters/InnerClasses 恢复消费、handler 根策略、现代源码输出与规模测量仍分别处理；driver 类名/flags 与缓存字节容量已经交付。当前不发布 crate。
