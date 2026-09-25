# 静态初始化器的正常完成规则

2026-09-23 使用已包含 Throw 的 debug CLI。`run_audit.py` 保存三个完整自写类、javap、原始 jarde/JADX 文本及编译日志。`ThrowSwitch` 的三个 case/default 都抛异常，jarde 无引用，实际完整类直接重编译通过，没有多余的不可达 break。

`ThrowClinit` 的源码是 `static { if (true) throw ...; }`；`ThrowClinitBranch` 在两侧分别抛异常，其中一侧用 `if (true)`。javac 消除了恒真条件，两个合法 class 都只有异常出口。jarde 已正确恢复抛出表达式及分支，均无引用，但其静态块不能按 Java 的规则正常完成，完整重编译失败。JADX 的两个完整输出也在同一限制处失败。这是声明上下文的呈现问题，不是缺少 Throw 操作数或异常 CFG。

[JLS 8.7](https://docs.oracle.com/javase/specs/jls/se23/html/jls-8.html#jls-8.7)要求静态初始化器按语言规则能够正常完成；[JLS 14.22](https://docs.oracle.com/javase/specs/jls/se23/html/jls-14.html#jls-14.22)对没有 else 的 if-then 使用专门的可达性规则。源码使用恒真 if 合法，编译后的字节码无须保留该条件。

## 独立方案实验

`hypothesis.java.txt` 是明确标注的方案实验，**不是 jarde 输出**：仅在实际恢复的静态块外增加 `if (true)`，保持所有内部表达式、顺序和语句。两个类均重编译通过。每个类使用独立 JVM，连续两次触发类初始化；首次的 ExceptionInInitializerError 与原始 cause、第二次的 NoClassDefFoundError，合计四行观测与原 class 一致。原始失败文本也原样保留。

这说明受限修复可以复用既有 if/block 呈现，不需要异常调度器、额外方法或重做 CFG。后续必须先决定证明边界：只处理已完整恢复的普通类 clinit，证明最后语句不能正常完成；不能仅凭“不含 return”猜测，也不能为未知或引用片段作完整性承诺。应复用现有声明感知的 body 发射及来源计费，明确合成条件没有原字节码 BCI，保持内部节点来源和 commit/replay 一致。

本项暂记为独立债务，不扩张 `recover-throw-statements` 或静态 final 字段写入。接口初始化所有权、任意内部 return 和通用可达性分析仍需各自论证。
