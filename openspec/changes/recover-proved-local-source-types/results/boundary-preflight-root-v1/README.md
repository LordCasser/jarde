# 原始 Java 边界预检

root 审查 Luna 原稿并保存 private 副本；仅将 BoundaryRunner 首个 char-call-dot 的 index1 改为0，使标签与实际字符一致。JDK8/23 × debug/no-debug 四腿，每腿实际 compile/runtime/javap，共12命令exit0；8个class保持major52，四腿原始stdout/stderr完全相同。此处只接受原始输入，不代表 JADX/Jarde 候选已跑或类型恢复通过。

样例覆盖 C 调用/字段/i2c/entry参数、范围内 literal、越界/算术/copy、null-first String/构造写、混合引用/all-null/未知引用和候选槽复用。javac23 的实际字节码中 exactStringWritesAfterNull 为 String.<init>@40→astore43；possibleSlotReuse 有同一slot2的istore10与astore17/24/37，需后续同次IR核reuse身份，不能仅凭Java源码断言生命周期。

执行与独立验证脚本在父 results 目录。原稿和已执行源码/完整class/原始输出均保留。负例可以通过既有独立准确路线保持类型，也可以明确拒绝；不强制每个负例产生fallback。
