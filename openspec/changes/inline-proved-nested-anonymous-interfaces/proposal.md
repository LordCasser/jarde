## Why

DT-07 的固定 `TestNestedAnonymousClass` 形态把一个匿名接口实现作为另一个实现方法的直接返回值；当前 Jarde 只投影根处一个物理 child，并因内层 child 对外层匿名类的捕获字段关系无法闭合而拒绝根级源码。冻结 Java 8 对照中原源码与固定 JADX 完整源码重编、验证运行均输出 `1`，而 Jarde 完整物理源码不能按 Java 8 重编，证明了独立的类级投影差距。

## What Changes

- 在现有单站点匿名接口投影之后，只为一个准确两节点的直接返回嵌套匿名接口链证明并一次性呈现完整嵌套源码。
- 绑定每个物理类、分配/构造 BCI、接口合同、`InnerClasses`/`EnclosingMethod` 及内层唯一 immediate-parent synthetic capture；关系、方法体、引用清点或预算不完整时整棵树保持物理类表示。
- 复用同次方法 AST/分配扫描、匿名表达式发射器和结构化源码放置/范围翻译；不按 `$` 名称推断、不重跑恢复、不全局替换文本。
- 以 `TestNestedAnonymousClass` 派生的冻结完整源码作 Java 8 重编与 `-Xverify:all` 三方验收；`TestAnonymousClass12` 的匿名父类和外层字段捕获读作为明确的后续边界，不纳入本变更。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：增加已证明的两层匿名接口返回链在根源码中的原子嵌套投影及保守拒绝契约。

## Impact

影响 `src/facade.rs` 的类级匿名关系闭合与 `src/class_source.rs` 的结构化文本装配，使用 `jarde-java` 同次 AST/`AnonymousAllocationScan`、reader 的 typed nesting、`jarde-query` 的范围 XRef 和现有预算。保持物理 class/method 身份及单类查询可用；不新增公开 API、全局索引、CLI 协议或依赖。审计证据见 [DT-07 双层匿名接口审计](../../evidence/java-syntax-2026-09-27/dt07-nested-anonymous/analysis.md)。
