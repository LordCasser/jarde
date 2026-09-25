# Fixture notes

`tests/fixtures/p3-primitive-conversions/` contains one permanent Java 8 class,
`PrimitiveConversions.class`; the support, effect, and runner classes remain source-only.
The class has direct methods for all 15 explicit JVM primitive conversion opcodes, conversion
results consumed by overloads, intermediate float/double rounding chains, and left-to-right
effecting producers whose results are widened afterward.

The frozen class is 1,720 bytes, Java major version 52, with 31 methods and 31 Code attributes;
its SHA-256 is
`553c412824027fd41ad1176034bae95d3f2cf72fee7eb4fc2a06f02188b2179d`. `javap` counts all 15
conversion opcode families in the committed class. The evidence audit records the exact count
for each family, the exact runner line count, and the original/JADX/Jarde compile and execution
stages. The runner currently emits 41 lines, including 8 effect-order cases. The original class
is executed with `java -Xverify:all`; the original side is compiled first and then the exact
frozen class bytes are installed before execution.

The audit is `openspec/evidence/java-syntax-2026-09-22/numeric-conversions/fixture/run_audit.py`.
It runs the complete Engine class-source output without deleting or replacing generated methods,
and records the CLI SHA-256 before and after. The current CLI hash is recorded by the audit rather
than assumed from the older 7527 baseline; a mismatch during one run aborts so the audit can be
replayed in a fresh temporary directory.

root独立复跑保存在`numeric-conversions/fixture/root/`：冻结hash与重新javac的bytes一致，31 Code、41行原class实际输出，JADX完整类能编译但其中7行不同；jarde68引用、完整javac失败，未执行其正文。该次CLI前后同为feed5c构建。root删除了Rust测试对测试源码内容的冗余contains断言；实际Rust RED/green、census与fingerprint仍待Cargo窗口，不能把fixture输入准备当作生产恢复完成。
