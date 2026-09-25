## Why

一个由 `javac --release 8` 生成、JVM 验证通过的 `result = left || rhs()` 在 Jarde 中仍被完整引用；生成类可以编译，却把字段留在默认 `false`，并跳过 `rhs()`。同一冻结 class 经 JADX 1.5.6 反编译、重编和运行与原件一致；[三方证据](../../evidence/java-syntax-2026-09-25/short-circuit-shared-true/analysis.md)给出真实差异。现有短路字段写入 Region 已能一次认领这个共享 true 图形，缺的是在 SSA/CFG 证明和赋值发射中处理另一种边极性。

## What Changes

- 将现有 `ShortCircuitValue` 的静态 `Z` 字段写入证明从共享 false 的 `&&` 型推广到共享 true 的 `||` 型；按真实 CFG 边决定外层的立即值和内层的延迟求值，不能只倒置文本条件。
- 复用现有 `Conditional` AST、字段消费处的 JVM `Z` 最低位转换、来源追踪和完整引用回退。只有唯一 1/0 Phi、唯一 `putstatic` 读者、确定字段身份和无额外入口/效果/异常边时才原子发射。
- 用原 class、JADX 与 Jarde 完整源码的 Java 8 重编及执行，验证左真时 RHS 零次、左假时一次，字段值和来源一致；补充误极性、第二消费者、非 1/0 与拒绝边界控制。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对共享 true 的短路值写入静态布尔字段，在证明充分时给出等价 Java 表达式，证据不足时保留完整指令缺口。

## Impact

只涉及 `jarde-java` 的局部 CFG/SSA 证明、现有表达式构造和聚焦回归；不改变 `jarde-jvm` IR、reader、类装配、CLI API 或跨类身份机制。前置条件是 `recover-conditional-field-writes` 的共享 true Region 所有权与完整拒绝回退已存在。此 change 不推断原源码一定用了 `||`，不引入通用逻辑运算 AST，也不处理实例字段、循环、try 内异常边或任意多前驱 Phi。
