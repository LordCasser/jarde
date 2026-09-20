## Context

动机与实测见 [proposal](proposal.md)。已知事实（来自 [完成复核](../../completion-review.md) 的“新确认”一节，不构成新的根因结论）：

- 受控源码 `return (a + b).substring(1);` 经 `javac --release 8 -g:none` 编译后恢复出的正文是 `return arg0 + arg1.substring(1);`；
- 以 `("a", "bc")` 执行，原 class 返回 `"bc"`，恢复正文返回 `"ac"`；
- 同一次运行报告 `Complete`/`Structured`/`ContainsStatements`，没有拒绝，恢复侧的 syntax/compile/semantic/verification 仍是 Unchecked/NotAttempted/Unproven/NotPerformed；
- 另一个样本 `(a + b).length()` 被写成 `a + b.length()`，连返回类型也不成立。

`build.rs` 的拼接规则把 `String` 拼接链构造成 `ExprKind::Binary { op: Add, .. }`，再把它作为调用 receiver 交给 `Emitter::expr`；`Emitter::expr` 的 Call 分支打印 receiver 后直接追加 `.`，而括号只由 `binary_operand` 在 **Binary 臂**内按优先级/结合性补出。因此这一缺陷不是“值错了”，而是同一棵树被打印成另一个程序：**分组只按父运算符判断，没有按子表达式所在的位置判断**。

## Goals / Non-Goals

**Goals:**

- 子表达式写在接收者位置（至少调用，以及呈现实际写出的其它 receiver 形态）时，文本 MUST 按 Java 语法解析回同一棵树。
- 分组判定以**位置**为单位：为每个位置说明是否需要分组、该位置能否承载会被拼接重组的子式，以及覆盖它的证据。
- 以受控 fixture 的两侧编译执行对照证明「呈现文本与原 class 在分歧输入上同值」，并把精确文本、变异与正向对照永久化。

**Non-Goals:**

- 不新增/升级依赖或 crate；不引入 verifier；不新增公共类型、报告平面或停止分支。
- 不建全局类型或优先级框架，不改 `ExprKind`/私有 AST，不改 `build.rs` 的值、求值点、拒绝与 origin 逻辑。
- 不重开 R8/R9、`bound-recovery-recursion` 的递归界、`fix-nested-arithmetic-value` 的优先级/结合性修正。
- 不声称一般语义等价，也不声称覆盖整类表达式缺陷（本 change 只闭环受控到达的 receiver 位置）。
- 不做性能工作（`optimize-demand-workloads` 保持 0/22）；不修 body 解码重新解析类的债务。

## Decisions

### 1. 分组按位置判定

文本由 Java 自己的语法消化，所以判据只能是「这段文本解析回同一棵树吗」。同一个子表达式在不同位置答案不同：`f(a + b)`、`new X(a + b)`、`xs[a + b]`、`x -> a + b` 里的加法已经被逗号、括号、方括号或 lambda 体界定，加括号是装饰；而 `(a + b).substring(1)`、`(a + b).field`、`(a + b)::length`、`!(a + b)` 里的加法没有界定，`.`、`::` 与 `!` 绑定更紧，直接拼接就换成另一棵树。

实施者 MUST 从当前 `Emitter::expr` 的代码**枚举**所有写入子表达式的位置，并把结论按位置记进 verification。规划期的候选清单（MUST 复核，不得直接照抄）：

| 位置（写入点） | 需要分组 | 本次覆盖方式 |
| --- | --- | --- |
| 二元操作数（左/右） | 由既有优先级/结合性规则决定 | 既有 `binary_operand` 与 `the_printed_text_keeps_the_arithmetic_tree` 必须逐字段不变 |
| 调用/构造实参 | 否（`,` 与 `)` 已界定） | 正向对照：文本不新增括号 |
| 调用 receiver | 是（**本 change 的缺陷点**） | 受控 fixture + 精确文本 + 执行对照 |
| 字段读取 receiver | 是 | 复核可达性：可达则进 fixture 与对照；不可达则给出代码证据说明为什么没有可到达的形态 |
| 方法引用限定符 | 是 | 同上（按可达性判定，不得只凭直觉关闭） |
| 数组位置（`array[index]` 的 array） | 是 | 同上 |
| 下标位置 | 否（`[ ]` 已界定） | 枚举说明 |
| 一元 `!` 的操作数 | 是 | 复核可达性（既有归档记录称本子集里 `Not` 的子式恒为 boolean 参数的 `Local`，本 change 按当前代码复核并记录） |
| lambda 体 | 否 | 枚举说明 |

「需要分组」不等于「一定出现括号」：接收者的子式若本身绑定更紧（`arg0.foo()` 的 `Local`、嵌套调用），MUST 保持原文本。规则是「位置 + 子式形态」共同决定，无法用单一 `if` 覆盖。

### 2. 最小实现边界

修正落在 `Emitter::expr`（及同文件内的私有 helper），不触碰 `build.rs`：组合规则与括号写在哪里是打印问题，不是值分析问题。括号 MUST 写在子式自己的节点上，保持既有段表语义（`fix-nested-arithmetic-value` 已确立：括号写进父节点 span、不移动任何 anchor），不新增 AST 变体、不复制子表达式。

### 3. 「单一打印臂」不是位置证据

`binary_operand` 只从 Binary 臂调用，所以它只覆盖**二元父**；归档 verification 的覆盖面清单（“调用与构造实参、lambda 体、数组下标、字段 receiver 等”都经 `Emitter::expr`）说明的是文本从哪里写出，不是每个位置的分组被验收过。本 change 的验收 MUST 是「受控 fixture 的编译/执行对照」加「精确文本断言」，或一条点名代码位置的可达性论证；引用打印路径或历史声明不构成证据。

### 4. 受控 fixture 提交真实字节，对照执行原 class

fixture 放在 `tests/fixtures/p3-receiver-grouping/`：

- `ReceiverGrouping.java`：受控源码，含缺陷形状与正向对照，按现有 fixture 的写法保持与反例一致的形状；
- `v8/ReceiverGrouping.class`：`javac --release 8 -g:none -d v8 ReceiverGrouping.java` 的真实输出；
- `README.md`：编译器版本与确切命令、字节数、SHA-256、逐成员字节码，沿用 `p3-nested-arithmetic` 的记录格式；
- `Baseline.java`：原 class 的驱动，打印输入集合的原值（对照的基准侧）。

规划期候选成员（成员名与正文由实施者按实测确认，不得按本表反推）：

| 成员 | 形状 | 期望文本 |
| --- | --- | --- |
| `tail(String, String)String` | `return (a + b).substring(1);` | `return (arg0 + arg1).substring(1);`（缺陷期为 `arg0 + arg1.substring(1)`） |
| `size(String, String)I` | `return (a + b).length();` | `return (arg0 + arg1).length();` |
| `nested(String, String, String)I` | `return (a + b + c).length();` | 只加外层一对括号（同优先级内部左结合子式不新增括号），精确文本由实测确认 |
| `len(String)I` | `return a.length();` | `return arg0.length();`（正向对照：不加括号） |
| `chain(String, String)String` | `return a.substring(1).concat(b);` | 逐字不变（正向对照：嵌套调用不加括号） |

输入集合 MUST 含一个让两侧可观察结果不同的输入：已实测 `("a", "bc")` 时原 class 返回 `"bc"`、缺陷正文返回 `"ac"`（`("r", null)` 形态也可分歧——缺陷正文在那里抛 `NullPointerException`——但不能只靠这种行）。既有 `sample_values` 对 `String` 固定给 `"r"`/`null`，若它不能覆盖分歧输入，实施者在同一对照文件内为该样本扩展输入集合并记录该扩展只影响本样本。只用一个输入、或只比较「两侧都没抛异常」的行不算覆盖。对照沿用 `tests/p3_execution_comparison.rs` 的既有流程（新增 `Sample`、登记进 `REQUIRED`、逐成员 `Member`）：从本次事实派生声明、`javac --release 8` 编译呈现文本、两侧 trace 逐行比较、`#[ignore]` 常备并由 CI 显式运行。

### 5. 拒绝路径的验收

若某个位置的子表达式无法被安全分组（例如呈现形态本身无法表达所需分组），该区域 MUST 走既有 refusal：引用 bytecode、保留 origin、`quality=Fallback`、`representation=Mixed`、诊断点名相关 BCI 与物理方法。MUST NOT 用「没有生成 Java」当作通过；也 MUST NOT 为了不拒绝而写出解析回另一棵树的文本。

### 6. 复用与依赖

不需要新库或新抽象：`Expr`/`ExprKind` 与既有段表机制足够表达位置分组。`javac` 只是测试侧的外部 oracle（固定版本、非构建依赖、缺席如实报告），生产引擎保持离线、不执行目标代码。新增 fixture 会改变 `tests/fixtures/corpus-fingerprint.json` 与 `repository_class_fixtures_validate_without_false_target_rejections` 的计数，两者都按既有流程显式再生成/更新并记录实测值。

## Risks / Trade-offs

- **只修一个位置就宣布整类关闭** → 位置枚举是独立交付物；未做 fixture 的位置必须有代码级可达性论证。
- **顺手加括号改变既有精确文本** → 既有精确文本回归（`tests/p3_eval_context.rs`、`tests/p3_content.rs`、`crates/jarde-cli/tests/json_cli.rs` 的既有期望）MUST 逐条核对；确需移动的期望必须能追溯到本次有意改变的行为并留下理由。
- **对照两侧共享同一错误** → 基准侧运行原 class 自己的 driver，不复算呈现侧。
- **过度保守（一律加括号）** → 正向对照必须逐字不变：`arg0.length()`、嵌套调用、同优先级内部子式只保留必要的一层括号。
- **语料变更被静默接受** → fingerprint 再生成器与 reader census 都要求记录实测计数，改语料必须显式可见。

## Migration Plan

1. 在固定行为基线上先记录反例：CLI 命令、恢复正文、报告平面、两侧执行值与分歧输入（修正前完成）。
2. 枚举位置并逐条给出分组判定、可达性与覆盖证据。
3. 实施最小修正，保持既有优先级/结合性行为不变。
4. 提交 fixture 与来源 README，执行 fingerprint 再生成与 census 更新，加入精确文本回归、执行对照、拒绝边界与变异。
5. 跑固定提交门禁并写 verification；同步 delta 与状态引用后归档。

与 `type-boolean-contexts`、`spell-array-types` 串行实施（同一 crate，本 change 第 1 个）。回退按本 change 的独立提交进行；回退后必须恢复「该产物算错值」的公开事实，不能保留完成声明。

## Open Questions

无。位置清单的可达性由实施者按当前代码查证并记录，这不是需要上游裁决的设计歧义。
