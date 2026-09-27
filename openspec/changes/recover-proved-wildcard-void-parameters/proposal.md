# Recover proved wildcard parameters on empty void methods

## Why

DT-17 的五个固定 Java 8 通配符方法可由原源码和 JADX 完整源码重编并保留运行时反射泛型类型。Jarde 当前也能编译运行，却把 `List<?>`、`? extends`、`? super` 和数组界全部投影为原始 `List`，原反射 API 行为改变。[三方报告](../../evidence/java-syntax-2026-09-27/dt17-wildcard-void/report.md)给出可重放证据。

## What Changes

- 扩展已有同轮空 `void` 返回候选，允许一个未使用的参数槽在完整 Code/AST/SSA 中证明无读取、写入、副作用或异常路径。
- 在现有 reader Signature 擦除和 class-source 声明门下，仅对一个 `java.util.List` 参数且唯一参数化实参为 `?`、`? extends` 或 `? super` 的已支持简单类/基本类型数组界恢复泛型方法参数；无 Signature 的原始 `List` 保持原始。
- 用完整类源码重编、`-Xverify:all` 和反射参数类型对照验收正例与真实负例，预算/取消时拒绝局部投影。

## Impact

只覆盖 DT-17 的静态、单参数、空 `void` 方法切片；带效果或控制流的正文、多个参数、字段、泛型返回、复杂成员界以及 DT-18 其它参数化类型不在本次改动内。物理成员、单方法报告和现有普通 Signature 路径继续保留。
