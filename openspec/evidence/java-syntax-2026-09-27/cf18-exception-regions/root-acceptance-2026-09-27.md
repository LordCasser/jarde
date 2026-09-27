# CF-18 主线独立复核

root 以合入 CF-14 后的主线 CLI（SHA-256 `0c299a1c0143259d5cfe2580ad2ba30e101168ba5c1cec972874952a366086f4`）从 [归档原 class](baseline/ExceptionRegionsAudit.original.class)重新生成 Jarde 完整源码，SHA-256 `b6869f5b3b7862c82ad6d3c6dfb2f404bd2d30899da9d910d7c85995aa3c029d` 与审计归档逐字节相同。root 独立重编原源码、固定 JADX 与 Jarde 完整源码：原 class 通过 `java -Xverify:all` 输出 `124:115`；固定 JADX 完整类可编译但运行时让 `IllegalStateException` 逃逸；Jarde 的 `run()I` 保守引用，完整类因缺少返回语句不能编译。

物理异常表和 BCI 对照见[审计报告](report.md)。固定 JADX 的错误是真实运行差异，而 Jarde 没有发布错误的控制流；两者都未达到该 Java 8 正例的恢复目标。这是 CF-18 的具体首片差距，不从六项测试的弱文本断言推定全部异常区形态均有问题。
