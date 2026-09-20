## Context

动机与两个复现见 [proposal](proposal.md)。已知事实（来自 [完成复核](../../completion-review.md) 的 T1/T4 两节，本 change 按当前代码位置核对过一次）：

- `crates/jarde-java/src/build.rs::concat_expr`（约 2561 起）先为每个 `append` 调用 `render_value(value, at, depth + 1)`（约 2580），再从第一个片段起左折叠 `ExprKind::Binary { op: Add, .. }`（约 2586–2595）：树的深度等于 append 数。`MAX_VALUE_DEPTH = 24`（`build.rs:73`）在 `render_value` 入口检查，它约束的是**值**的嵌套，不是这棵树的形状。
- 这棵树的每个生命周期都递归：构造（折叠循环建出 `Box<Expr>` 链）、打印（`crates/jarde-java/src/emit.rs:475–479` 的二元臂对左右子树递归，复核记 `emit.rs:476` 为左操作数打印点，LLDB 的重复帧在那里）、释放（`Box` 链的 `Drop`），以及任何对 `Expr`/`ExprKind` 的克隆（`ast.rs:114` 派生 `Clone`，`Box<Expr>` 的克隆同样沿链递归）。因此「界内输入」不等于「四件事都有界」。
- 复核的两个构建边界：debug 在 N = 1536/2048 以 signal 结束（1024 完成，exit 134、空 stdout、`stack overflow, aborting`），optimized release 主线程对 2048–15000 完成。两者都不是界的依据，也不能相互替代。
- `crates/jarde-java/src/concat.rs` 的 `Chain`（`head`/`tail`/`class`/`appends: Vec<(u32, String)>`/`owned`）已经逐 `append` 记下 BCI 与**参数类型**（`concat.rs:75–87`）；`keeps_its_conversion`（`concat.rs:654`）逐 overload 接受 `int/long/float/double/boolean/String/Object`，`char`/`char[]`/`CharSequence` 等一律让整条链拒绝。那条判断只在**字符串上下文**里成立：`String.valueOf(int)` 与 `append(int)` 相同，`arg0 + arg1` 不做任何转换。
- 既有受控形状没有覆盖 T4：`crates/jarde-java/tests/p3_patterns.rs` 的 `concat_order_class` 第一个操作数是 `String` 字面量，`concat_buffer_class` 的第一个 `+` 里已有一个返回 `String` 的调用——两者的第一个 `+` 都在字符串上下文里。
- 上一轮归档的 `bound-recovery-recursion` 把这条债务明确留在这里（其 verification 的「边界与残留」：`emit::expr` 对 `concat_expr` 左折叠树的递归，深度等于一条拼接链的 append 数）。同一次归档给出了两条本 change 沿用的规则：进入前检查的显式界，以及「界不得由构建配置的完成深度替代」。
- 内存 fixture 的既有先例是 `tests/p3_eval_context.rs::recursion_fixture` 与 `crates/jarde-cli/tests/task_cli.rs::recursion_fixture`（两份逐字节相同的生成器；其注释记录「预修行为只写一次、常备断言的对象是界的行为」与 `StackMapTable` 的必要性）。

## Goals / Non-Goals

**Goals:**

- 拼接链的呈现逐片段保留 `append` 自己的转换，转换发生在片段自己的位置，并从字符串上下文开始；文本计算与链相同的值。
- 链长只增长片段序列，MUST NOT 增长本层在构造、发射、克隆或释放路径上的递归深度；片段内部真正嵌套的子表达式仍按既有 `MAX_VALUE_DEPTH` 受限。
- 公开入口与 CLI 对深链在 debug 与 release 两个构建上都返回报告：正常完成（能呈现时）或既有已发布停止（预算/取消时），MUST NOT abort。
- 以受控执行对照把转换语义永久化，并以子进程级检查把「无 signal」永久化；两类变异（恢复左深折叠、恢复丢失转换）都必须让检查变红。

**Non-Goals:**

- 不改拼接规则的识别，也不扩大 `keeps_its_conversion` 的接受集合（`char`、`char[]`、`CharSequence`、`StringBuffer` 之外、以及一切未知 overload 继续拒绝）。
- 不新增 crate/依赖、报告平面、停止分支或预算维度；不 spawn 工作线程、不使用 `catch_unwind`、不放大线程栈、不引入增长栈的库。
- 不做一般表达式类型推断；不改 `emit.rs` 的其余分组规则，不改 `decide-comparison-contexts` 与 `unify-local-type-decisions` 的决定。
- 不重开 R8/R9 与已归档的递归界、分组、boolean、数组类型修正；不声称任意链长安全或覆盖整类表达式缺陷；不做性能工作。

## Decisions

### 1. 一个所有者：片段表示拥有整条链的呈现

选定的机制是一份**片段序列**（design 层的候选名 `Concat { parts }`，形状由实施者按 `ast.rs` 的既有风格定），每个片段带着：

- 自己的表达式（就是今天的 `render_value` 结果，内部嵌套仍受 `MAX_VALUE_DEPTH` 约束）；
- 该 `append` 的**参数类型**（`Chain::appends` 的既有 `String` 描述符/类型）；
- 自己的 origin/BCI 锚点（片段自己的 append BCI，以及片段表达式的直接锚点）。

构造：按 `chain.appends` 迭代，逐个取片段（今天已是循环，改成把片段放进序列而不是折进 `Box` 链）。发射：打印机按序列顺序迭代，每个片段按自己的上下文与嵌套打印。克隆/释放：序列的 `Vec<Part>` 派生 `Clone`/`Drop` 是**迭代**，每个片段自己的子表达式仍受既有值深度界约束。

被否决的候选：

- **只加打印守卫**（superseded 方案）：树仍被构造、仍要在同一深度释放，也要在打印到一半时撤回已写出的产物；而且它把「界」留在一个与输入长度同阶的形状上。
- **构造前的深度界**（superseded 方案的第二形态）：把合法输入拒掉换取不 abort，把「2048 个片段必须停止」变成产品契约；复核明确要求深链可以正常完成。
- **迭代化通用的打印/释放**：能继续处理更深的输入，但要把打印与 `Drop` 都改写、把深度换成显式工作栈，超出本 change 需要的边界；片段表示已经让这一层的深度与链长解耦。
- **放大线程栈 / 增长栈的库**：把 abort 推迟到更深输入，不构成界，也不是「增加一个片段不增长这一层深度」的性质；复核明确不接受。

### 2. 链从字符串上下文开始，判据只用链自己的事实

呈现的第一个 `+` MUST 已经是字符串拼接，使每个片段的转换落在它自己的位置：

| 事实 | 结论 |
| --- | --- |
| 第一个 `append` 的参数类型是 `java.lang.String` | 首个片段已是 String，无需改变 |
| 第一个 `append` 的参数类型是 `int/long/float/double/boolean/Object` 之一 | 需要让 `+` 从字符串上下文开始（在序列前加一个空字符串片段，或用同等语义的 `String.valueOf` 形态） |
| 首个 `append` 的参数是 `Object`、而该值的本次运行类型被证明是 `java.lang.String` | 文本已是 String；插入与否语义相同，按「是否证明为 String」选定并记录 |

选择空字符串片段而不是逐片段 `String.valueOf(x)`：前者是既有的 `ExprKind::Str("")` 字面量（原子、无副作用、无调用），语义与 javac 对 `"" + a` 的 lowering 一致；后者要在文本里引入字节码没有的调用，并为每个片段再判断用哪个重载。插入的空字符串片段**没有对应指令**，它的 origin MUST 由链的既有 BCI 派生（`Origin::derived`，与折叠今天对 `chain.owned` 的用法同一模式），段表 MUST NOT 出现一个「读了某条指令」而实际没有读的锚。

`MUST NOT` 用括号或右折叠修补：`(arg0 + arg1) + "!"` 只是把数值加法说得更清楚，值仍然错；右折叠 `arg0 + (arg1 + "!")` 改变求值/转换顺序。

### 3. 片段的转换按它的 `append` 参数类型拼写，boolean 由既有类型决定给出

`keeps_its_conversion` 接受的每个 overload 在字符串上下文里都有与 `append` 相同的文本：`int/long/float/double` → `String.valueOf`；`String` → 本身；`Object` → `String.valueOf(Object)`；`null` → `null` 字面量。**boolean 是唯一需要显式按 descriptor 拼写的一项**：`append(true)` 的 `iconst_1` 在文本里 MUST 是 `true` 而不是 `1`（`"" + 1` 得到 `"1"`，与 `append(true)` 的 `"true"` 不是同一个值）。拼写规则是：片段的目标位置由 `append` 的参数 descriptor 给出（`Z` 即要求 boolean），值自身的证据决定它是不是能拼成 boolean——这与 [decide-comparison-contexts](../decide-comparison-contexts/proposal.md) 的三分一致。当片段的值是一次对局部变量的读取时，「这个变量是不是 boolean」由 [unify-local-type-decisions](../unify-local-type-decisions/proposal.md) 的局部类型决定给出，本 change MUST NOT 在拼接路径里再作一次局部类型判定。

接受集合与拒绝路径保持不变：集合内每个 overload 的转换都必须能拼出同一文本；某个片段拼不出时走既有 refusal 契约（保留 bytecode 与 origin、Mixed/Fallback、诊断点名 BCI），MUST NOT 通过扩大接受范围、猜测类型或改写转换绕过。

### 4. 顺序、副作用与锚点：不允许平衡、合并或重排

- 片段序列 MUST 就是 `chain.appends` 的顺序；每个片段的求值次数与左到右次序不变（不重复调用、不移动可能抛异常的操作、不改变异常次序）。
- MUST NOT 对片段做任意平衡、合并或重排：把两个相邻数值片段合并成一次加法、把片段提到括号外、或为了「少一层」改变某个片段的求值位置，都会改变转换与副作用，属被禁止的实现。两个可观察形态必须同时成立：两个片段 `append(a).append(b)` 是 `"" + arg0 + arg1`（`(1, 2)` → `"12!"`），**一个**片段 `append(a + b)` 是 `"" + (arg0 + arg1)`（`(1, 2)` → `"3!"`）；实现 MUST NOT 把前者合并、也 MUST NOT 把后者拆开。
- 每个片段与它自己的 append BCI 一一对应；插入的空字符串片段取派生锚点（决策 2）。
- 「片段内部真正嵌套的子表达式」仍按今天的 `MAX_VALUE_DEPTH` 受限：本 change 只让**链长**不再贡献深度，不取消值嵌套的界，也不把它换成「更大的界」。

### 5. 深链的可观察结果与不变量

- 输入侧：链长达到复核记录的区间与更大规模时，两个构建的公开入口与 CLI 都 MUST 返回报告；能呈现时 MUST 正常完成（MUST NOT 把「2048 个片段必须停止」写成契约），不能呈现时 MUST 结束为既有已发布停止。
- 停止侧：预算或取消在深链的构建/发射过程中生效时，MUST 发布既有停止形态（`outcome = Stopped`、非 `Complete` 的 `execution`、点名原因与位置的诊断、`content = not_produced`），且 MUST NOT 因释放这棵结构而 abort（`build/emit/drop` 三条路径都在证据面内）。
- 默认线程栈是验收条件：检查 MUST 在默认栈上运行，MUST NOT 通过 `RUST_MIN_STACK`、大栈线程或任何等价手段取得结论。

### 6. 两类受控 fixture：转换用真实编译产物，深链用生成字节

- **转换形状**（javac 可产出）：`tests/fixtures/p3-concat-conversion/`（`ConcatConversion.java`、`v8/ConcatConversion.class`、`README.md`、`Baseline.java`），成员覆盖 `append(int).append(int).append(String)`（`(1, 2)` → `"12!"`）、需要转换的片段在最后、全字符串、`append(boolean)`（含 `iconst_1` 的布尔片段）、`null`、对象片段、以及一个可观察/可能抛异常的片段（求值次序）。规划期候选成员（成员名、正文与精确文本由实施者按实测记录）。
- **深链**（选择**生成字节**，而不是 `javac -J-Xss64m` 产出的 class）：`javac -J-Xss64m` 只是复核为了产出字节而用的手段——编译器自身的递归需要放大栈，测试 MUST NOT 依赖它；链长是本检查的**参数**（复核区间与更大规模都要跑），生成器才能确定地给出它；链形是直线 `new`/`dup`/`invokespecial` + N×(`aload`/`invokevirtual append`) + `toString`/`areturn`，常量池小而固定，可以逐项断言。生成器沿用 `recursion_fixture` 的两份逐字节相同副本模式（`tests/p3_eval_context.rs` 的库入口与 `crates/jarde-cli/tests/task_cli.rs` 的子进程入口），并在注释里点名它镜像的复核样本（同一 `StringBuilder` 链、同一 overload、同一直线、同一区间）与忠实性判据；若 frame pass 需要属性，按 `recursion_fixture` 的方式补最小 `StackMapTable` 并写明原因。
- 修正前的行为只写一次：复核的 debug signal（1536/2048，含 exit 134、空 stdout、stderr 行）与 release 完成范围写进 verification；常备断言的对象是「返回报告」而不是崩溃本身。

### 7. 复用与依赖

不需要新库：改动都在 `concat_expr` 的构造与 `emit.rs` 的打印上，必要时新增一个纯增量的 `ExprKind` 变体（不改变既有节点的含义）；`Concat` 的识别、`Chain::appends` 的参数类型、`Origin`/`OriginSet`、`stop.rs` 的发布路径都已存在。`javac` 仍是测试侧外部 oracle。

## Risks / Trade-offs

- **插入位置错（放在第一个字符串片段之后）** → 以多顺序对照证伪：数值在前、数值在最后、全字符串、对象、boolean、`null` 各有成员，`(1, 2)` 的值必须逐项相同。
- **多插一个空字符串改变既有文本** → 「数值在最后」与「全字符串」两条对照 MUST 逐字不变。
- **布尔片段被写成 `1`** → 对照必须包含一个 `append(boolean)` 成员，值 `true` 与文本 `1` 的差异必须被执行对照捕获。
- **片段表示仍保留左深结构**（例如把序列又折回 `Binary`） → 变异任务要求恢复左折叠后深链检查变红；检查覆盖构造/发射/释放三条路径。
- **深链被「界」拒绝而当成修好** → spec 明确深链可以正常完成；正向对照要求能呈现的深链完成并给出正确文本。
- **只覆盖打印，留下释放或克隆** → 证据面要求 build/emit/drop 都记录；实现若改变节点形状，克隆路径同样要在证据里。
- **把 release 的完成当界** → 两个构建边界分别记录；默认栈是验收条件。
- **语料变更被静默接受** → fingerprint 再生成器与 reader census 都要求记录实测计数。

## Migration Plan

1. 在固定基线上重放两类反例：T1（受控深链在 debug 的 signal 与其 native 栈证据，确认与默认/放大预算无关；release 的完成范围单独记录）与 T4（`(1, 2)` 的两侧值、两侧都能编译）。
2. 实现片段表示、字符串上下文的起始、按参数类型的片段拼写（含 boolean/`null`/对象）与锚点处理；核对接受集合与拒绝路径不变。
3. 提交转换样本与生成器，执行 fingerprint 再生成与 census 更新，加入精确文本回归、执行对照、子进程级深链检查（debug；release 由显式门禁运行并记录）与变异。
4. 跑固定提交门禁并写 verification（两个构建的边界、build/emit/drop 证据、多顺序对照、变异、门禁结果、两个拒绝边界）。

与 `decide-comparison-contexts`、`unify-local-type-decisions` 串行实施（同一 crate，本 change 第 3 个，且与第 1、2 个共享 `build.rs`）。回退按本 change 的独立提交进行；回退后必须恢复「深链在 debug 构建下 abort、数值前缀先做数值加法」的公开事实。

## Open Questions

无。片段表示的确切字段与命名、`emit.rs` 迭代的落点、深链检查的 N 取值由实施者按测量与既有风格选定；spec 只约束可观察结果（文本的值、求值次序、报告与无 signal），不约束 AST 形状。若实施发现必须以「放大栈」或「普遍拒绝深链」才能通过检查，那属于本 change 的设计失败，必须回到片段表示而非改验收。
