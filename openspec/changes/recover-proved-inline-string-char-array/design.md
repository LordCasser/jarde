## Context

见 [proposal.md](proposal.md) 与 [EM-27 冻结审计](../../evidence/java-syntax-2026-09-27/em27-string-concat/report.md)。`build::ArrayInitializers::prove` 已用 SSA、字段计划、元素写入和异常效果证明完整数组初始化，且允许其唯一消费者为 `Invoke`；它目前在方法 builder 内运行。`init::sites` 在它之前证明外层 `new@1`，对分配副本至构造调用间的 `newarray`、`dup` 和 `castore` 一律当作独立效果拒绝。先把数组存入局部的控制形态能分别由这两个计划恢复，证明缺口是同轮计划的接缝。固定 JADX 把两种 `new String(char[])` 都写成字面量，重编后 `== "abc"` 从 `false` 变 `true`；这种常量折叠不能作为我们的恢复规则。

## Goals / Non-Goals

**Goals:** 让现有数组初始化计划在构造站点证明时可读，仅为 Java 8、准确 `String([C)V`、唯一内嵌 `char[]` 实参、直接返回的闭合形态承认该数组为构造实参表达式。沿用同一份计划生成 AST 与来源，不重新推断数组写入。

**Non-Goals:** 通用“任意构造器接任意数组”规则、把构造改成局部数组语句、字符串常量折叠、byte[]/charset、Java 11 concat、数组别名或额外可观察效果。

## Decisions

1. **提前且只运行一次既有数组证明。** 在字段计划之后、`init::sites` 之前调用现有 `ArrayInitializers::prove`，将结果借给 `init::sites` 后移交给 builder。只开放跨模块读取所需的窄证书查询，不复制元素/SSA/handler 检查，也不增加新的 pass 或 AST 类型。考虑过在 `init` 内重新写一份 `char[]` 扫描，它会与 builder 对可内联数组的判断分叉；也考虑过把数组提前输出到新局部，但会把数组分配移到外层 `new String` 之前，违反原字节码的分配及异常次序。两者均不采用。

2. **只豁免准确子表达式，不放宽一般构造效果。** 外层 `new`/`dup`/`invokespecial` 必须仍由 `new@1` 的原有实例身份、唯一消费者和单块顺序证明；其 owner/描述符固定为 `java/lang/String.<init>([C)V`。唯一构造实参须为同一 SSA block 内的已证明 `char[]` 初始化结果，数组证书的 `consumer` 恰为该构造调用，所有元素写入、复制、长度及来源 BCI 位于外层副本与调用之间。只将这个证书拥有的指令及其已证明元素生产者视为构造实参范围；任何其它效果、别名、第二消费者、异常 handler 差异或覆盖缺口仍拒绝外层站点。不能仅以 `String` 类型或 `"abc"` 内容猜测子表达式。

3. **复用原有表达式和来源发布。** builder 从移交的同一数组证书写 `ExprKind::NewArray`，外层站点用既有 `New` 表达式读取它，并由原有 `Return` 放置。数组各 `castore` 和分配/构造 BCI 留在衍生/直接来源中；物理 IR 与类身份不改写。完整方法 artifact 在预算扣费及发射成功后才发布，失败保留原物理回退。原/Jarde 输出应为 `new java.lang.String(new char[]{...})` 一类保身份语法；固定 JADX 的 `"abc"` 仅作反例，不作文本验收目标。

4. **保持相邻已验路径。** 普通 `StringBuilder` concat、先存局部数组再构造、非 String 构造器及无数组方法原样经过已有站点规则。提前数组证明可能改变预算计数顺序，因此定向回归必须核对这些控制、预算停止和取消，并确认未引入多次收费或部分发布。核心库自行作决定，CLI 仍只呈现报告。

## Risks / Trade-offs

- [数组证书被错配到另一调用] → 同时核对数组最终 SSA 值、唯一消费者 BCI、实参位置与准确 `([C)V` 描述符；错误 owner 或第二读者拒绝。
- [外层分配与内层元素效果被重排] → 子范围必须完整落在外层 `dup` 与构造调用之间，且沿用数组计划的原始元素次序和 handler 检查；不得生成提前存局部的替代源码。
- [计划提前后预算或取消出现部分报告] → 数组证书在同一预算中只建立一次；站点、builder 和输出对停止传播作原子回归，保留执行状态与原 BCI。
- [JADX 的内容折叠看似更简洁] → 身份运行反例已否定其等价性；本切片明确保留显式 `new`，不从字符值推断 interned 字面量。
