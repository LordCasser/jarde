## Why

DT-06a 的冻结 Java 8 类把两个有副作用的实参传给匿名类父类构造器；Jarde 的物理类源码能正确重编运行，却仍写 `new AnonymousSuperDirect$1(next(), next())`。JADX 写出源级 `new Base(next(), next()) { ... }`。在 DT-05 已有准确分配 BCI、闭合 owner 普查和同次 AST 发射能力上，可以补齐这一独立语法切片，同时避免把捕获参数误当成父类实参。

## What Changes

- 对无捕获、无实例字段和初始化效果、直接返回、唯一分配点的匿名父类子类，证明物理构造器把全部有序实参原样转发给准确选中的父类构造器，再呈现 `new Base(args...) { ... }`。
- 复用 DT-05 的 typed `InnerClasses`/`EnclosingMethod`、完整分配扫描、闭合 owner XRef、同次方法 AST 和原子类源码发布；实参仍由原调用者 AST 发射，保留左到右求值、副作用和异常顺序。
- 用冻结原/JADX/Jarde 完整源码对照验证重编与运行。构造参数中混入捕获值、实例字段写入、重载目标不明、第二分配点、跨类引用或正文不完整时保留物理类源码。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：补充已证单站点、无捕获匿名父类构造实参的源级匿名表达式，并在参数角色、身份或正文证明不全时保持物理表示。

## Impact

修改 `jarde-java` 的同次匿名构造参数证明与 AST 发射，以及 `src/facade.rs` 的类族装配门槛；复用现有 reader、解析器和 XRef，不新增全局扫描器、公开报告 schema 或外部依赖。DT-05 的无参接口切片保持独立；DT-06 含捕获、实例初始化器和非直接返回位置仍需后续任务。物理子类及方法报告继续可独立查询。
