# DT-25：无捕获 lambda helper 与完整类源码

## 结论

固定 Java 8 样例的窄差距已修复。原始源码、指定 JADX revision 与修后 Jarde 的完整类源码都通过 `javac --release 8`，经 `java -Xverify:all` 的 source-only `Runner` 输出 `7`、`15`、`42`。修前 Jarde 把每个私有 static synthetic `lambda$...` helper 作为成员输出，且 lambda 体调用该方法；javac 重编时生成同名 helper，报 `compiler-synthesized symbol conflicts`。修后 Jarde 只在精确证明所有使用和完整算术体后，原子内联并省略源级 helper；物理方法报告仍保留。

该记录将 DT-25 标为**冻结差距已修复、单元待扩验**。首片 OpenSpec 仅涉及无捕获、同类、私有静态 synthetic、原始类型 0/1/2 参数和有界直线 helper body；DT-26 捕获、DT-27 方法引用、泛型 SAM 适配、其它编译器与复杂 body 独立处理。

## 可复现材料

完整源文件、原/JADX/Jarde 输出、`javap -p -c -v`、原/JADX/Jarde 编译运行记录、输入与输出 SHA-256、固定工具位置及 `replay.sh` 均在 [冻结证据目录](../../../changes/recover-lambda-synthetic-helpers/evidence/java8-lambda/)。该脚本清理临时 class 输出并保留完整的源码、诊断和运行记录。JADX 参考 revision 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。

## OpenSpec

[窄提案](../../../changes/recover-lambda-synthetic-helpers/proposal.md)已按精确 BSM helper 所有权、物理指令 BCI 与 AST 来源覆盖、全类引用清点和有界遍历实施；任一不完整事实、未支持形态、预算或取消都阻止 helper 集合被部分省略。[修前与修后三方回放](../../../changes/recover-lambda-synthetic-helpers/evidence/java8-lambda/baseline.md)保留完整输出与稳定哈希。
