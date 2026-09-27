## Context

[EM-22 冻结对照](../../evidence/java-syntax-2026-09-27/em22-arithmetic/report.md)中，Jarde 的 `Operation::Bitwise` 已通过 `boolean_evidence` 判定有种子证据的布尔位运算，并将两操作数 `boolean_spelling` 后建 `ExprKind::Binary`。`ExprKind::Not` 已在同一 AST/emitter 处理其它否定。固定 JADX `SimplifyVisitor` 的 XOR 分支也只在左值为 boolean、右侧常量为 0/1 时替换为 MOVE/NOT；本设计借用这个窄条件，但仍以 Jarde 的真实 SSA/描述符证据判断，不从结果方法返回 `Z` 倒推左值一定是 boolean。

## Goals / Non-Goals

**Goals:** 将准确布尔 `value ^ true` 输出 `!value`，`value ^ false` 输出 `value`，不重复、不跳过有副作用的左值求值，保留正确来源与 Java 类型。

**Non-Goals:** 通用代数化简、交换左右操作数、数值 `^ -1` 恢复为 `~`、Smali/Dex `not-int`、复杂布尔表达式简化、任意常量折叠或更改 eager `|`/`&` 语义。

## Decisions

1. **直接复用现有布尔证明与 AST。** 对 `Operation::Bitwise { Xor }` 精确核对 JVM `ixor`、两个栈消费者、左值由参数/调用描述符或已决布尔局部独立证明为 `Z`、右值由准确单个 `Push(Int(0|1))` 定义且只在该位置被取作常量。必须在原本 `ExprKind::Binary` 可合法输出 boolean 的位置选择 `ExprKind::Not` 或左表达式；不能仅因返回描述符是 `Z` 或值碰巧为 0/1 就重写 int 位运算。

2. **只渲染左表达式一次。** 按原 `render_value` 路径取得左值，`^ 1` 用其建已有 `Not`，`^ 0` 直接返回它；右常量无效果，故这两种替换不移动真实效果。对 `^ 0` 保留 XOR BCI 作为派生来源，对 `^ 1` 的 `Not` 使用 XOR BCI 为主来源。预算/停止或验证失败沿现有 `Bitwise` 路径拒绝/回退，不写部分表达式。

3. **拒绝边界按原事实收紧。** `long/int` XOR、左值布尔证据不足、右值不是准确常量、操作数个数或 opcode 不符，都不做此简化。不要把 `|`、`&` 变成短路运算；EM-22 的 `left() | right()` 运行计数为 2，是独立护栏。

4. **用固定 Java 8 三方全类验收。** 原/JADX/Jarde 与同一 Runner 在八行结果和副作用计数上一致；Jarde 改后 `flip`/`flipCall`/`sameCall` 呈现 `!arg1`、`!left()`、`left()`（可保留显式 `this.`）。数值 `notInt`/`notLong` 仍为 XOR -1，因固定 JADX 对此 Java 8 class 也是同样写法。补充未知类型、非 0/1、右侧调用、长整型、预算/取消定向负例。

## Risks / Trade-offs

- 将所有 `ixor 1` 当作 `!` 会改变整数值域和源码类型；左值必须有独立 `Z` 来源，不能借结果上下文猜测。
- 省略 `^ false` 时若左值调用被误删会改变效果；同一表达式实例只消费一次，运行计数负例覆盖。
- Smali 的直接 `not-int` 与 Java 8 `ixor -1` 不同；此任务不解释反编译器为何选择 `~` 或 `^ -1`。
