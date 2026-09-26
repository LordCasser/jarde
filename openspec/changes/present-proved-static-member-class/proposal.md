## Why

DT-02 的冻结 Java 8 对照中，Jarde 的完整源码虽能重编运行，却把真正的静态成员类保留为独立的 `StaticMemberBasic$Leaf`，而 JADX 写出 `static class Leaf` 与 `new Leaf()`。现有具名成员家族装配只接受非静态捕获型子类；应把已证静态成员关系接入同一源码单元，而不能按 `$` 猜嵌套关系。

## What Changes

- 在所选物理定义的双方 `InnerClasses` 行准确一致、静态成员声明及成员正文可完整呈现时，根类源码包含 `static class Leaf`，对应类型和创建使用点写源级 `Leaf`/`Outer.Leaf`。
- 对无关系的顶级 `Named$Top`、多义/缺失定义、不完整方法或预算停止保留物理类文本；独立物理报告仍可查询。
- 以冻结原/JADX/Jarde 三方 Java 8 编译运行和负例核对源码形态、身份与语义。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：准确证明的单个直接静态成员类可作为根类的嵌套源码声明及其使用点源名呈现。

## Impact

涉及 `src/member_inner.rs` 的成员关系候选及 `src/class_source.rs`、`src/facade.rs` 的家族装配，尽量复用已有成员关系与物理报告。Reader、JVM IR、CLI 命令和匿名类投影无须改变；家族报告需要明确区分“静态无捕获”与“非静态捕获”状态。本变更不扩展到多子类、泛型嵌套或任何基于名称的全局改写。
