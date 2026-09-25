## Why

普通 Java 8 类中的 `receiver().value += rhs()` 与 `data[index()] += rhs()` 会被当前 Jarde 输出成可编译但漏执行 RHS 与写入的正文，改变数值、调用次数和异常。独立 write-only 整类对照中，原类/JADX 七行相同，Jarde 六行不同；因此应先修复这一静默错义，而不是仅补齐后置递增的不可编译返回。

## What Changes

- 对已证明的 `int` 实例字段和 `int[]` 元素 `+=` 语句恢复单次求值的复合左值写入，保持读旧值在 RHS 之前、接收者/索引调用一次及空值/越界异常时机。
- 在当前 SSA/事实/语句/emitter 路径中有界认领 `dup`/`dup2`、读、`iadd` 和写的完整身份；无法证明时完整引用，不生成重复求值或遗漏效果的 Java。
- 固定源码、JADX、Jarde 完整生成类的编译/执行、拒绝、来源和预算回归。

前置：已有字段/数组读写事实、SSA 值身份、调用延期消费、结构化语句和来源/预算机制。后置递增返回旧值单列后续工作。

非目标：局部 `+=` 的风格化重写、`++/--` 的返回值、其它复合运算符、long/float/double、byte/char/short 的隐式窄化、通用 `dup`/`dup2` 别名机制、任意跨块更新或反编译器对源码原样的推断。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：对有界可证明的实例字段和数组元素复合加法写入生成语义等价的 Java 8 语句。

## Impact

涉及 `jarde-java` 既有 build/AST/emit 与字段、数组值消费边界；无需新 crate、解析器、外部依赖或目标代码执行。CLI/Engine 接口及报告平面保持原有约定。实证及可重放脚本在 `../../evidence/java-syntax-2026-09-22/compound-assignments/`，本 proposal 不声称生产实现已经存在。
