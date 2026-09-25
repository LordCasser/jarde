## Why

Java 8 的 `result = extra || left || rhs()` 在三个条件出口共用 true 生产者。Jarde 当前把它拆成嵌套 `if`，漏掉 BCI 14/15/18 的来源并把字段消费点误报为循环；完整类虽能重编，却把三个应为 true 的结果变成 false。JADX 1.5.6 对同一冻结 class 能生成可执行的 `||` 链，这个 parity 缺口已超出既有“两次测试”短路证明。

## What Changes

- 将现有短路字段值 Region 的所有权从固定两次测试推广到受预算限制的同极性测试链，使共享生产者和唯一 `putstatic` 消费块仍只被认领一次。
- 在已有 CFG/SSA/字段证明内逐个验证链上的解码跳转、精确前驱、值来源、异常边和单消费；可证时用现有 `Conditional` 表达式保留逐级惰性求值。证明失败时完整引用整条链和写入，不能丢失真值生产者。
- 冻结源码、原 class、JADX、Jarde 的三方编译/执行及拒绝边界；已验收的两次测试 `&&`、`||` 和异常边拒绝不得回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 可证明的同极性多测试短路布尔链在唯一静态 `Z` 字段消费处保持原 class 的值、调用次数和来源；不可证明时保留完整缺口。

## Impact

预计只修改 `crates/jarde-java` 现有 `Region::ShortCircuitValue` 的私有数据形状、局部 CFG/SSA 证明及 `Conditional` 构建；复用预算、source map 与 class-source，不增加 crate、公共协议或通用 CFG 重写 pass。处理器边、混合 `&&`/`||`、实例字段、任意 Phi 消费及匿名类投影不在本次恢复范围；现有 `recover-conditional-field-writes` 和 `recover-disjunctive-field-writes` 的两次测试行为是前置回归门槛。
