## Why

DT-05 的冻结 Java 8 样例中，Jarde 能重编并正确运行匿名接口实现，却把使用点写成 `new AnonymousInterfaceBasic$1()`，将匿名体作为另一份物理类源码呈现。JADX 的 `new I() { ... }` 证明该语法恢复有实际价值，但其按使用方法数判断唯一性的算法会在同一方法的双分配点改变类身份；需要以 Jarde 已有的物理分配证据实现更严格的最小闭环。

## What Changes

- 对单个无捕获、无初始化效果、实现一个接口、仅有一处分配的匿名类，完整证明物理身份、构造器和所有方法体后，在准确使用点呈现 `new I() { ... }`。
- 用现有同次分配扫描和所选输入范围的完整 XRef 排除第二处分配、外部类身份引用及未决目标；证明失败时保留当前物理类表示。
- 用冻结的原/JADX/Jarde Java 8 源码、重编及执行对照检验源码形态与语义；把捕获、基类构造实参、初始化器和多处分配留给后续独立变更。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：已证单站点无捕获匿名接口类在使用点呈现完整匿名体，证据不全时维持物理类输出。

## Impact

涉及 `jarde-java` 同次方法 AST 投影和 `src/facade.rs` 类源码装配；复用 reader 的 `InnerClasses`/`EnclosingMethod`、已有 `AnonymousAllocationScan` 与 `jarde-query` 的范围 XRef。物理方法报告和身份仍可查询；不增全局索引、通用匿名类机制、CLI schema 或目标代码执行。
