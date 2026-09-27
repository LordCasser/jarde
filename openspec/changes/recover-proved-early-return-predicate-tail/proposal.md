## Why

CF-02 的判空、`instanceof` 和提前返回样本中，Jarde 的区域遍历丢掉一个正常可达的布尔返回块，却已输出可编译的无条件 `return false`；原 class/JADX 对 `"x"` 返回 true，Jarde 重编源码返回 false。[三方冻结对照](../../evidence/java-syntax-2026-09-27/cf02-predicates/report.md)已把比较运算本身与这个控制流归属错误分开。

## What Changes

- 让带提前返回的谓词区域必须交代每条正常后继：内层分支若有未消费的返回合流，不能把它静默丢在外层 one-armed `if` 后。
- 在无异常边、SSA 值与区域归属完整的 Java 8 形状上，恢复 `if (invalid) return false; return predicate;` 或等价短路表达式，确保每条路径的返回值、cast 后调用次数及异常顺序不变。
- 对无法证明的分支整段原子回退并保留可达 BCI/来源，不能让一条无条件返回掩盖未恢复的真实路径。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证提前返回谓词尾路径恢复，以及未证后继不得形成语义错误的可执行前缀。

## Impact

前置条件是既有 canonical CFG、SSA、Region walk 和布尔值 builder；主要涉及 `crates/jarde-java/src/region.rs` 与必要的既有 `build.rs` 证书消费，不增加新 crate、通用 CFG 重写层或目标代码执行。范围不包含 `TestTernary3` 的外部 `InsnArg` 类型层级、复杂异常 handler、循环出口或 CF-03 的所有 if/else-if 美化。JADX 的条件合并可参考，验收以原 class 与完整 Java 8 重编源码运行行为为准。
