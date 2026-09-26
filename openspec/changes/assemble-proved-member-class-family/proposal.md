## Why

Java 8 命名非静态成员类的字节码被拆成多个物理 class；当前 `class-source` 只呈现所选的一个定义，不能输出可编译的嵌套声明，也不能凭同类型区分显式参数 `other` 与捕获的 `Outer.this`。[家族边界审计](../../evidence/java-syntax-2026-09-26/named-member-outer-receiver/family-assembly-analysis.md)还指出，现有成员构造调用证明只接纳 public 目标，无法代表 javac 的非 public 成员声明。

## What Changes

- 对精确选中的根定义，依据双向 `InnerClasses` 和选定环境中的唯一物理 child，装配已证明的命名非静态成员声明；根与 child 的原始字段、方法和报告仍保留各自身份。
- 证明唯一构造捕获参数与 synthetic 外层字段的写入和全部相关读取，只在证明完整时把它们投影为 Java 嵌套作用域与 `Outer.this`；同类型的显式 receiver 保持独立。
- 使同一源码家族内的成员构造调用按已证明关系呈现，不再把目标 class/构造器原始 `ACC_PUBLIC` 当作成员可访问性的唯一证据。
- **BREAKING**：已证明的根 `class-source` 文本和报告将包含嵌套成员的源码投影与物理覆盖；不能再将输出默认为只含所选物理类。缺失或冲突的家族证据保留保守报告，不伪称可编译。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：对完整证明的命名成员类家族、捕获接收者与源码声明给出可编译的 Java 8 投影，失败时保持物理引用。
- `source-maps`：家族文本中每个 child 的声明、方法和被省略的捕获构件保留准确物理 owner、BCI 与派生来源。

## Impact

主要涉及 `src/facade.rs` 的 class-source 准备与请求预算、`src/class_source.rs` 的声明/文本装配、`src/member_inner.rs` 的关系证明，以及 `crates/jarde-java` 的捕获值与成员构造呈现。JADX 源码仅作算法顺序对照，不进入生产依赖。本变更只覆盖无 `access$` 方法 super 桥的命名成员子集；`Outer.super`、local/anonymous、构造委托、多构造器与泛型外层作用域另行立项。
