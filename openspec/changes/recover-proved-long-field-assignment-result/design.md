## Context

见 [proposal](proposal.md) 与 [冻结基线](../../evidence/java-syntax-2026-09-27/em07-long-assignment/report.md)。`jarde-jvm` 已识别并验证 `dup2_x1` 的栈形，SSA 保留每份输出的定义与消费者；`jarde-java` 把该 opcode 留作 `Operation::Other`，普通字段写入的值渲染因此停在 BCI 2。现有 AST 有 `FieldAssign` 语句和 `Return` 语句。固定 JADX 也把原始 `return this.value = input` 写成 `this.value = j; return j;`，故首片无需新增赋值表达式节点。

## Goals / Non-Goals

**Goals:** 仅把准确的实例 `long` 字段写入结果拆成两个有共同来源证据的既有 AST 语句，并在完整方法证明后发布。

**Non-Goals:** 通用 `dup2_x1` 语义恢复、任意右值/接收者、静态字段、`double`、多重赋值、字段复合更新、数组写入、异常处理路径、跨类字段解析的新规则，或把两个语句美化为一个赋值表达式。

## Decisions

1. **在现有方法 builder 内增加窄证明，不建新 pass/AST 类型。** 从同次已解码 Code、frame/SSA、字段证据核对完整单块五步 `aload_0; lload_1; dup2_x1; putfield self.field:J; lreturn`。`putfield` 必须指向当前物理类唯一可见的准确 `J` 字段；`aload_0` 是本实例，`lload_1` 是方法的唯一 `J` 参数。没有额外操作、异常 handler、未达块、stack clone 或第二个读写消费者。考虑过扩展 `Operation::Duplicate` 泛化所有栈重排，但 `dup2_x1` 的 category-1 双值形式及其它消费者不由该夹具证明；不在首片放宽。

2. **精确跟踪复制前后的两份 category-2 值。** 验证 `dup2_x1` 读取一份本实例引用和一份宽度为 2 的参数值，其输出中被 `putfield` 消费的 receiver/value 与 `lreturn` 消费的 value 分别是该复制的正确位置；两份 value 均追溯到同一 `lload_1` 且只被各自指定消费者使用。不能因为 opcode 名称或数值相同就推断别名。结合 canonical effect/handler 事实确认没有隐藏的异常路径或越界归属，预算/取消在发布前完成检查。

3. **从证书向既有 writer 原子写两句。** 在 BCI 3 写 `StmtKind::FieldAssign`、BCI 6 写 `Return`，两者的值都渲染原始参数；BCI 2 只作为导出来源锚点且不再单独回退。字段写入仍经过已有字段值类型与源名检查，返回仍经过 `J` 描述符检查。完整组合未构造成功时原方法保留已有拒绝，不得只写字段、不写返回或把参数重新求值。另一选择是添加 `ExprKind::Assignment` 再让通用 precedence/emitter 支持它；固定 JADX 本身不需要这个语法，增加的范围不符合首片边界。

4. **以固定三方重放验收。** Java 8 原 class 的 `javap` 必须确认 `dup2_x1`；原/JADX/Jarde 完整 `Assignment` 与同一 Runner 重编并在验证器下输出两次返回值/字段值，包含跨 32 位边界与负数。另测错误 owner/描述符、第二消费者、插入效果或 handler、错误宽度及预算/取消；这些都不能偶然发布字段赋值结果。物理分析报告和原始指令仍可查询。

## Risks / Trade-offs

- [把复制误认为重复求值] → 只接受直接参数来源，按 SSA 输出值与两个指定消费者建立双向唯一对应。
- [拆语句改变右值副作用] → 直接 `long` 参数读取没有副作用；其它右值一律拒绝。
- [在部分预算下发布半个方法] → 先证明并暂存完整两句，再通过现有方法 artifact 的原子发布和停止传播。
- [未来要恢复单条赋值表达式] → 另以有具体输入的 OpenSpec 评估通用 AST 节点；本首片的源码与固定 JADX 输出形态一致。
