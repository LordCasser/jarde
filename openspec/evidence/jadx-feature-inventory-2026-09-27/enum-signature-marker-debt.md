# 枚举 class Signature 的多余拒绝标记

在主线 `85117144` 用当前 CLI 重放固定 [DT-13 嵌套枚举](../java-syntax-2026-09-27/dt13-enum-shapes/replay.py)时，三方完整源码仍通过 Java 8 重编与 `-Xverify:all`，原 API consumer 输出逐字相同；但新生成的五份 Jarde 枚举展示文本比仓库内冻结文本多出两行 `class Signature ... not projected` / `class Signature projection refused: class_generic_source_unproved` 标记。例如 `NestedShape$Major` 的物理 Signature 为 `Ljava/lang/Enum<Ldt13/NestedShape$Major;>;`，源码枚举头已经正确写出。

这是独立的报告质量债务：需核对类 Signature 拒绝标记是否应在已证明的枚举常规 `Enum<E>` 超类上出现，以及它与真实未恢复泛型边界如何区分。重放没有发现枚举语法或运行语义回退；DT-22 提交未纳入这些旁支快照变化。后续应以冻结 DT-13 类和一个真正不支持的类 Signature 作正反例，再决定是否调整诊断门槛。本项不混入成员注解或 lambda 改动。
