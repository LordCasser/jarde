# `FinallyOnce.main`：完整类的独立恢复缺口（已闭合）

> **2026-09-30 闭合**：拆分取证见[前置语句具名 catch 证据](../../java-syntax-2026-09-30/finallyonce-main-catches/README.md)，实施见 [recover-preceded-statement-catches](../../../changes/recover-preceded-statement-catches/)。合并主线后原始完整类全方法恢复，`handled`/`escaping` 输出不变。下文为闭合前的登记原文。

固定原 class `FinallyOnce.class` SHA-256 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2` 的 `handled` 和 `escaping` 已分别按同布局最小完整类验收；原始完整类的 `main([Ljava/lang/String;)V` 仍为 explanation-only，不能据两方法的结果声称整类源码可重编。这是独立于 CF-16 清理副本证明的后续单元，不混入 Test14 的条件 finally 实现。

`javap -c -p` 显示 `main` 在 BCI 3、37、86 各建一条 `StringBuilder` 拼接链；第三条位于具名 `IllegalStateException` handler BCI 82 之后，真实 `toString` 在 BCI 111。当前 Jarde 报告却给第三条 `jre_concat_split`：称 BCI 86 的链结束于 **BCI 28**。`crates/jarde-java/src/concat.rs::verify` 在当前版本对全方法 `all` 使用首个同 owner 的 `toString` 来判断跨块，未以 head 之后的同一 builder 值流限制候选；因此这个诊断至少不能作为第三条真实链跨块的证据。前两条拼接已被接纳，不应为修这个诊断引入新的泛用拼接机制。

同一运行还报告 `jre_guard_resource_init`（BCI 65）和 `jre_region_uncovered_blocks`（未覆盖的 live block 117、82），最终 `main` 只生成两个 region、两个 statement 并整方法回退。即使修正上述首个 `toString` 误配，也没有证据证明具名 catch 的区域就能恢复。下一步应以这份固定方法建立独立同布局最小完整类，先分别测多条拼接链与后置具名 catch，再沿 `concat::Plan` 和普通 catch Region 核首个拒绝原因；只有完整 Java 8 源码重编与原 class 运行等价、负例及来源闭合后才开实施任务。
