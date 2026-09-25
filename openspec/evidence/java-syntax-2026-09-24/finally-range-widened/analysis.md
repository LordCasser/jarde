# `finally` 保护范围包含正常清理调用的合法反例

源类为 [ImplicitCleanup.class](../../java-syntax-2026-09-22/finally-completion/implicit-cleanup/ImplicitCleanup.class)（原始 SHA-256 `924437916dc278eefe3b83cdcf3bad14bfb3cf8b9e44c027786a89f0712247b6`）。它由 `javac --release 8 -g:none` 生成，`run()` 的 catch-all 范围为 `[0,20)`，正常路径的 `cleanup()` 位于 BCI 20，异常处理器的同一调用位于 BCI 26。

本目录冻结一个**仅修改异常表端点**的变体：在唯一的 `00 00 00 14 00 19 00 00` 八字节记录中，将 `end_pc=20` 改为 `end_pc=23`，其余 class 字节不变。变体 SHA-256 为 `c5407d3f29b135818f003e24b218b5e4b7348d261665f0c71605471a7492935e`；`javap.txt` 显示 `[0,23) → 25 any`。这使 BCI 20 的正常清理调用受同一处理器保护，而 BCI 25 的处理器和 BCI 26 的第二次清理不在范围内。变体是有效 Java 8 class，`java -Xverify:all` 运行通过；它不是声称来自原 Java 源码的第二种编译结果。

原始 class 清理抛错时，正常返回的 trace 为 `29`；变体在相同输入上先于 BCI 20 清理追加 `9`，该调用抛错进入 BCI 25 处理器，再于 BCI 26 清理追加 `9`，得到 `299`。其余三组输入输出仍与原始类相同。`original-run.txt` 保存变体四行结果。若仅凭“两个清理调用相同 + handler 重抛”把变体折成 Java `finally`，正常清理抛错只执行一次，行为就会错误地回到 `29`。

JADX 1.5.6 的 `jadx-raw.java` 保留 `try { … cleanup(); return …; } catch (Throwable th) { cleanup(); throw th; }`，仅为重编译移除生成的 `package defpackage;` 行；完整类经 `javac --release 8 -g:none`、`java -Xverify:all` 后四行与变体原 class 逐字相同，包括 `299`。Jarde CLI SHA-256 `95362354d3de2af1ac57f69ea9f7492731ca580afd2e5f7e3b16fc4144b3aa8c` 仍给 `run()` 报 `jre_guard_finally_copy` 并引用相关 BCI，完整类因缺方法返回而不能重编；这里不声称 Jarde 的执行结果。

本例要求证明 `finally` 前先检查异常表的**半开覆盖**：两个候选清理副本均不能被该 catch-all 范围覆盖，否则清理自身的异常会重入 handler，`finally` 的完成优先级与次数就不等价。该检查属于现有 `recover-proved-finally-cleanup` 2.1/2.3 的拒绝条件，不需要新增恢复机制。正例 `[0,20)` 仍单独验收，不以扩大范围的反例替代。
