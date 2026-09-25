## Why

现有 `produce; pop; invokestatic` 限定符形状会把同一生产者既写成独立语句、又嵌入后续静态调用。普通 Java 8 完整类中的 `receiver().value = rhs()` 因此被恢复为两次 `receiver()`；原 class/JADX 四行相同，Jarde 完整类可编译运行但一行调用次数由 1 变 2，属于静默错义。

## What Changes

- 紧邻生产者本身是可独立表达的方法调用且 `pop` 只消费其返回值时，复用已有弃值调用语句，再按常量池目标呈现后续静态调用；其它已证明可承载的限定表达式仍只发射一次，保持求值顺序、异常与调用结果。
- 限定表达式无法独立呈现时，只有其 Java 类型能合法选到原常量池目标才写成表达式限定调用；接口静态方法及异属主不猜测目标。
- 若承载调用不能呈现，拒绝范围保留限定表达式的生产者、`pop` 和调用的物理来源，不因为延期消费而丢失效果。
- 用源码/JADX/Jarde 整类编译执行和正负例锁定普通静态字段写入、表达式限定静态调用、纯字段读、抛错及预算/来源边界。

前置：既有 `discarded_evaluations`、调用值延期、source map/预算路径。非目标：推断原源码中的 `receiver().value = rhs()`、`value = receiver().rhs()` 与 `receiver(); value = rhs()` 哪一个写法曾出现；三者在已证明的字节码形状下可语义等价。也不恢复新的静态字段 AST、任意 `pop`、一般栈别名、复合字段写入或跨块区域。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：同一静态调用限定表达式在恢复正文中只求值一次；拒绝时保留其延期生产者。

## Impact

限于 `jarde-java` 既有 build 中的 pop 认领、读者/延期与拒绝来源路径及相邻测试。无新 crate、AST 类型、依赖或外部接口。实证见 `../../evidence/java-syntax-2026-09-22/static-field-qualifier/`；本 proposal 不声称修复已经实现。
