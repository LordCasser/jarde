## Context

[EM-23 冻结证据](../../evidence/java-syntax-2026-09-27/em23-field-updates/report.md) 的 `increment()` 是 `aload_0; dup; getfield I; iconst_1; iadd; putfield I; return`，Jarde 已输出 `this.instanceField += 1`。`decrement()` 是 `getstatic I; iconst_1; isub; putstatic I; return`，Jarde 输出等价的读写赋值。`build.rs::CompoundAssignments`、`FieldIncrements` 和 `PostIncrement` 已证明更复杂的字段/数组更新；`field@1` 声明物理成员，SSA 则携带值与消费者。新任务只扩展同一证明位置的末端语法。

## Goals / Non-Goals

**Goals:** 对无结果使用的准确 int 字段加一/减一，写出 `++`/`--` 并保持字段身份、单次读取/写入、接收者求值和异常边。

**Non-Goals:** 局部 `iinc` 文风、数组更新、带返回值的前/后缀区分、任意常量复合赋值、`String +=`、嵌套 `this.a.f`、volatile 字段、跨类字段绑定或通用代数化简。

## Decisions

1. **限制物理更新链。** 首片只接受同一无 handler 单块中严格闭合的两条链：本实例 `aload_0; dup; getfield self.f:I; iconst_1; iadd; putfield self.f:I; return`，或本类静态 `getstatic self.f:I; iconst_1; isub; putstatic self.f:I; return`。字段必须是同一准确物理 `int`、非 volatile 成员。使用 `field@1` 与 SSA 双向确认每份 read/write 的唯一消费者、没有插入效果、第二用途或值返回；源码接收者只允许当前实例或本类静态 owner。其它字段更新仍保留现有恢复。

2. **在既有后缀节点中表达方向。** 将 `PostIncrement` 统一为一个带增/减方向的后缀更新节点，复用现有目标表达式、来源集合、优先级和 emitter。方向只取已证明的加一或减一；不新增一套字段语句 AST 或后处理文本替换。构造 `StmtKind::Expr` 时保留物理 read、constant、arithmetic、write 的来源，且整条链由该语句一次认领；若任何检查失败，保留原有 `FieldAssign` 或现有拒绝。

3. **无返回值是本首片必要条件。** 单纯语句的 `field++` 与 `++field` 在更新后不使用值时结果相同，故后缀文本可统一。若 bytecode 返回旧值或新值，必须走现有返回值证书或另开形态；绝不把有值消费者的更新缩成普通语句。不能在异常边或未完成块中隐藏字段读写。

4. **原/JADX/Jarde 全类验收。** [replay.py](../../evidence/java-syntax-2026-09-27/em23-field-updates/replay.py) 增加 fixed 拼写断言，三方完整 `Updates` 与同一 Runner Java 8 重编、`java -Xverify:all` 输出仍相同。`append` 的低级 StringBuilder 链保持原状，由 EM-27 单独处理。补充不同 owner/descriptor、额外消费/效果、handler、非本实例 receiver、volatile 与预算/取消负例。

## Risks / Trade-offs

- 把任意 `x = x ± 1` 文本改写成后缀会误折叠两次接收者求值；只消费上述准确物理链，拒绝复杂接收者。
- 将有结果使用的更新当语句会改变前/后缀返回值；首片严格要求唯一 `return;` 在字段写入后且无旧值/新值用途。
- 后缀节点增加方向但不扩张证明范围，旧 `PostIncrement` 的数组/返回值测试必须保持原行为。
