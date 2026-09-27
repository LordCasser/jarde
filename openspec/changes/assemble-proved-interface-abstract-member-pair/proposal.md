## Why

固定 JADX `TestClassGen` 方向的 EM-01 `Shape` 根类同时声明一个 `public interface I` 和一个 `public abstract static class A`。原 class 与固定 JADX 的完整 Java 8 根源码重编运行通过；Jarde 的物理 child 报告各自存在，却在根源码同时漏掉 `Shape.I`、`Shape.A`，使外部 Runner 编译失败。现有单 child 声明切片已能恢复孤立的 `A`，但 `scan_family_root` 把接口 row 当作普通类拒绝，家族模型也只能容纳一个 child。

## What Changes

- 在现有成员家族扫描/装配边界接纳**恰好两个**直接、无构造使用的成员声明：一个 Java 8 静态接口和一个已获证的静态抽象类；两者各有准确双向 `InnerClasses` 关系、完整物理方法声明和唯一选定定义。
- 复用 `A` 的声明型证书，为 `I` 证明无构造器、无 Code 的抽象接口方法；在同一根源码事务中按源级顺序写出两个合法成员，保留物理 child 报告与派生来源。
- 以固定 `Shape` 独立完整类和 Runner 做原/JADX/Jarde Java 8 重编、验证运行；缺一 child、第三 child、关系/flags/方法/预算错误时整组不投影。`Generic.A` 留给独立泛型成员任务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在受证的 declaration-only 双成员家族中同时输出接口和抽象类声明，保证完整根源码可重编。

## Impact

影响既有 `member_inner` 扫描、class-source 成员家族准备和根源码装配；不新增 JVM IR、语法 pass、CLI 选项或第三方依赖。[EM-01 固定基线](../../evidence/java-syntax-2026-09-27/em01-declarations/report.md)中的 `Generic.A` 泛型字段/接口 Signature/bridge 不在本切片。
