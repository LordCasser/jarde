## Why

EM-20 的固定 Java 测试包含“在同步区域写入局部变量，离开同步区域后由循环使用”的形态。受控 Java 8 样例在 `-g:none` 下把同步监视器和异常清理临时引用所在的槽位复用为循环整数；原 class 与固定 JADX 的完整源码重编、验证运行一致，Jarde 把同槽位的 `Object`/`int` 视为一个局部并拒绝循环，完整类缺少返回语句。

## What Changes

- 在已有局部槽位复用计划中，为**已证明的同步监视器清理区域之后**的引用到整数生命周期分割建立窄准入；按 SSA 值链、异常边、区域归属和可达性证明先前引用不流入后续循环。
- 让分割出的循环局部通过现有命名、类型和声明位置计划恢复；证明不足时继续拒绝，保留 BCI、物理来源和停止语义。
- 用固定 EM-20 样例的原/JADX/Jarde 完整 Java 8 源码重编、`java -Xverify:all` 和输出对照验收；覆盖真实异常清理边及一个不能分割的反例。
- 不处理任意异常处理器后的槽位复用、不同类型组合、Smali 专有形态、通用同步语句恢复或循环表达式美化。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：在准确证明同步区域临时引用的生命周期已结束时，将复用的整数槽位作为另一个局部呈现，避免已支持的后续循环因类型冲突丢失。

## Impact

主要涉及 `jarde-java::reuse` 的现有 SSA/CFG 分割证明与 `build` 的声明消费；不新增 AST、crate、依赖或输出协议。固定证据见 [EM-20 对照](../../evidence/java-syntax-2026-09-27/em20-local-scopes/report.md)。本变更以现有 reader、JVM IR、同步区域证明及 Java 8 类源码发射为前提。
