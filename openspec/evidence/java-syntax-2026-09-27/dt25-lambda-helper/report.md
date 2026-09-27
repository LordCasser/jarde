# DT-25：无捕获 lambda helper 与完整类源码

## 结论

固定 Java 8 样例确认一个窄差距。原始源码与指定 JADX revision 的完整类源码都通过 `javac --release 8`，经 `java -Xverify:all` 的 source-only `Runner` 输出 `7`、`15`、`42`。Jarde 可读地呈现 0、1、2 个原始 int 参数的 lambda，但把每个私有 static synthetic `lambda$...` helper 作为成员输出，且 lambda 体调用该方法。Java 编译器又会为这些 lambda 生成同签名 helper，因此 Jarde 全类源码重编失败，诊断为 `compiler-synthesized symbol conflicts`。

该记录将 DT-25 标为**已证差距**，不表示整个单元完成。首片 OpenSpec 仅涉及无捕获、同类、私有静态 synthetic、原始类型 0/1/2 参数和可证明的直线 helper body；DT-26 捕获（root 的独立探针已发现同类 helper 冲突，留作相邻单元）、DT-27 方法引用、泛型 SAM 适配、其它编译器与复杂 body 独立处理。

## 可复现材料

完整源文件、原/JADX/Jarde 输出、`javap -p -c -v`、原/JADX/Jarde 编译运行记录、输入与输出 SHA-256、固定工具位置及 `replay.sh` 均在 [冻结证据目录](../../../changes/recover-lambda-synthetic-helpers/evidence/java8-lambda/)。该脚本清理临时 class 输出并保留完整的源码、诊断和运行记录。JADX 参考 revision 为 `2fb1b16386941660fda07e9017285aec40fcb37f`。

## OpenSpec

[窄提案](../../../changes/recover-lambda-synthetic-helpers/proposal.md)要求先证明精确 BSM helper 所有权、完整 helper body 与全类引用清点，再原子内联和省略；任何不完整事实、未支持形态、预算或取消都阻止 helper 集合被部分省略。
