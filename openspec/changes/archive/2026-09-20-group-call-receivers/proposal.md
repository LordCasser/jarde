## Why

受控 Java 8 样本 `return (a + b).substring(1);`（`javac --release 8 -g:none`）恢复为 `return arg0 + arg1.substring(1);`：接收者的文本被直接拼接，Java 把它解析成 `arg0 + (arg1.substring(1))`。以 `("a", "bc")` 分别执行原 class 与呈现文本，原值 `"bc"`、恢复值 `"ac"`；报告仍是 `Complete`/`Structured`/`ContainsStatements`，没有拒绝。

根因位置由 [完成复核](../../completion-review.md) 指出：`crates/jarde-java/src/emit.rs::Emitter::expr` 的 Call 分支先打印 receiver 再追加 `.`，而 `445a277` 加入的括号只接在**二元父节点**上（`binary_operand`）。二元子式内部正确，不等于它在接收者位置已经分组。

归档 verification 的覆盖面声明（“所有表达式都经 `Emitter::expr` 到达文本，因此该臂覆盖调用与构造实参、lambda 体、数组下标、**字段 receiver** 等”）不是这一上下文的证据：单一打印臂只说明文本从哪里写出，不说明任何位置的分组被判过，也没有任何接收者位置的受控样本或执行对照。

## What Changes

- 接收者位置的分组：调用（以及字段读取、方法引用限定符等呈现实际写出的 receiver 位置）上的子表达式 MUST 以「文本按 Java 语法解析回同一棵树」为准打印；`return (a + b).substring(1);` 的呈现 MUST NOT 写成 `arg0 + arg1.substring(1);`。
- 分组规则按**位置**决定，而不只按父运算符：二元操作数、调用/构造实参、调用 receiver、字段 receiver、方法引用限定符、数组与下标、一元操作数、lambda 体各自判定；需要分组的位置才补括号，Java 语法已经界定该子表达式的位置 MUST NOT 增加无谓括号。
- 实施 MUST 先枚举这些位置并逐条说明「是否可能承载会被拼接重组的子式」与覆盖证据，MUST NOT 只改一处 Call 分支就声明整类关闭。
- 验收：受控 fixture 的两侧编译执行对照（含分歧输入 `("a", "bc")`）、精确文本断言、恢复缺陷的变异，以及正向对照（`arg0.foo()` 形态不加括号、嵌套调用逐字不变、同一优先级内部子式不新增括号）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：表达式分组按位置决定；接收者位置上的子表达式必须保持自己的分组。
- `recovery-validation`：接收者的分组以受控执行对照与精确文本验收，并明确「单一打印臂」或归档调用路径清单不构成位置覆盖证据。

## Impact

实现落在 `crates/jarde-java/src/emit.rs`（`Emitter::expr` 的 receiver 位置与既有 `binary_operand` 规则）；不改私有 AST，不改 `build.rs` 的值/求值点逻辑。本 change 与 [type-boolean-contexts](../type-boolean-contexts/proposal.md)、[spell-array-types](../spell-array-types/proposal.md) 都改 `crates/jarde-java`，三者 MUST 按 [路线](../../roadmap.md) 顺序**串行**实施（本 change 为第 1 个），彼此不依赖对方代码。

交付包含受控 fixture（源码、class 字节、来源 README、`tests/fixtures/README.md` 登记、fingerprint 再生成、reader fixture census 更新）、精确文本回归、执行对照、变异与正向对照。反例 MUST 在修正前先记录（命令、正文、报告平面、两侧执行值）。

非目标：不新增 crate、依赖或 verifier；不建全局类型/优先级框架；不改 `ExprKind`/私有 AST；不重做值、求值点或 origin 逻辑；不重开 R8/R9 与已归档的递归界、打印修正；不声称一般语义等价，也不声称覆盖整类表达式缺陷；不做性能工作（`optimize-demand-workloads` 保持 0/22）。

当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。
