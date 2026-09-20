## Context

动机与复现见 [proposal](proposal.md)。已知事实（来自 [完成复核](../../completion-review.md) 的 T2 一节，本 change 按当前代码位置核对过一次）：

- `crates/jarde-java/src/build.rs::condition`（当前树约 3250 起）先 `let boolean = builder.boolean_value(operands[0].1, branch_bci)`，再渲染左操作数并对它无条件 `boolean_spelling`（约 3274–3278），**之后**才在 `Test::Zero`／`Test::Pair` 之间取舍（约 3279–3319），且 `Test::Pair` 分支直接复用已经拼过的 `left`（约 3348–3351）。
- `boolean_value`（`build.rs:1469`）由三部分证据构成：类文件 descriptor（`boolean_evidence`，`build.rs:1482`：`Z` 参数槽、返回 `Z` 的调用、descriptor 为 `Z` 的字段）、`0`/`1` 字面量（`boolean_literal`，`build.rs:1495`）、本 body 已声明 boolean 的局部（`boolean_local`，`build.rs:1512`）。其中字面量一项的注释写明它**只在已经要求 boolean 的位置**才成立——零值测试与二元比较的区别恰恰是「该位置是否要求 boolean」。
- 复核的对照证据：`fa6dc6e`（boolean 修正之前）对 `1 == n`、`0 < n`、`1 < n` 输出正确的整数文本，`5a8c36a` 之后三条都变成 boolean 文本。回归属于本项目自己的上一轮修正。
- 前一轮的形状与对照都在受控语料里：`p3-boolean-contexts` 的 `count(Z)I`（`if (arg0)`）、`isZero(I)Z`（`return true;`/`return false;`）、`localFromCall()Z`/`throughLocal(Z)Z`（`boolean local1 = …`）与 `nonzero(I)I`、`answer()I`（int 对照），已登记进 `tests/p3_execution_comparison.rs` 的 `REQUIRED`。

## Goals / Non-Goals

**Goals:**

- 整数二元比较保持两个操作数各自的整数文本；`0`/`1` 到 boolean 的映射只发生在真正要求 boolean 的位置。
- 上下文判定先于转换，且覆盖比较的**两个**操作数、常量在左与在右、相等与大小两个方向。
- 把这条回归变成**修正前就红**的常备回归，并用变异证明它在保护该顺序；前一轮的 boolean 形状与 int 对照逐字保留。

**Non-Goals:**

- 不建通用类型系统、不做数据流推断；不使用返回 descriptor、参数槽类型、callee/field descriptor 与既有局部声明记录之外的新证据。
- 不改 `value_type` 对其它类型的决定，不改 `emit.rs` 的打印规则，不新增 `ExprKind`/`Type` 变体、报告平面、诊断 code 或停止分支。
- 不重开 R8/R9、已归档的递归界与 concat 修正；不重开 `type-boolean-contexts` 的其它结论（本 change 只纠正它引入的这条回归）。
- 不声称一般语义等价或覆盖整类类型缺陷；不做性能工作；不修 body 解码重新解析类的债务。

## Decisions

### 1. 三件被混在一起的事分开，判定交给 `condition` 一处

| 问题 | 所有者与事实 | 本 change 的处置 |
| --- | --- | --- |
| 值自身的类型证据（这个值是什么） | descriptor 事实：`Z` 参数槽的读取、返回 `Z` 的调用、descriptor 为 `Z` 的字段读取、本 run 已证明 boolean 的局部；其余值（含 int 字面量）保持自己的整数形状 | 只读，不在 `condition` 里扩展 |
| 目标位置的要求（这个位置要不要 boolean） | 位置本身：`Z` 方法的 `return`、零值测试且操作数被证明 boolean 的真值测试位置、写入已证明 boolean 的变量的值；整数二元比较**没有**这一要求 | `condition` 在渲染操作数之前判定 |
| 字面量适配（`0`/`1` 怎么拼） | 上一条的**结果**：只有要求 boolean 的位置才把 `0`/`1` 拼成 `false`/`true` | 仅在判定为要求 boolean 的那一支发生 |

`condition` 是**唯一**同时看见比较形态（`Test::Zero`/`Test::Null`/`Test::Pair`）与操作数证据的地方，因此「这个位置是否要求 boolean」的判定只能在这里作出，且 MUST 在渲染任一操作数之前作出。实现可以按「先定 `Test` 与是否 boolean 上下文，再渲染操作数」，或按「先渲染、再只对 `Test::Zero` + 证明 boolean 的分支做映射」；唯一 MUST NOT 的是**先**按 `boolean_value`（含字面量）转换、**再**按测试形态取舍——那正是 `5a8c36a` 引入的形态。

### 2. 两个操作数、常量在左与在右、两种比较方向按同一规则

判定与拼写规则对每个操作数独立成立：

| 形态 | `0`/`1` 操作数 | 变量/调用操作数 | 结果 |
| --- | --- | --- | --- |
| `Test::Pair`（`if_icmp*`，整数二元比较） | 保持 `0`/`1` | 按值渲染 | `1 == arg0`、`arg0 == 1`、`0 < arg0`、`arg0 > 0` |
| `Test::Zero`，操作数被证明 boolean | 保持按 boolean 拼写 | 真值测试 | `if (b)`、`if (!b)`、`if (flag())` |
| `Test::Zero`，操作数未被证明 boolean | 保持整数比较 | 整数比较 | `arg0 != 0`（这不是缺陷） |
| `Z` 方法的 `return` | `true`/`false` | 按 boolean 拼写 | `return true;` |
| 写入已证明 boolean 的变量的值 | `true`/`false` | 按 boolean 拼写 | `local1 = true;` |

`if_icmp*` 是一对 int 操作数上的比较，它的**结果**是 boolean，这与两个操作数的类型无关；把该结果的类型回灌给操作数，就是本回归。验收因此不能只用 `n != 0` 证明「int 对照不变」：常量在左与在右、相等与大小比较都要有对照，否则「把左操作数无条件按值转换」的实现仍能通过。右操作数今天的渲染路径（按值）已正确，本 change 只把**规则**在两侧同时说清并纳入回归，不假设它不会被将来的改动破坏。

### 3. 已知但不在本 change 闭环的边界

「一个被证明 boolean 的操作数与一个 int 字面量组成的 `Test::Pair` 比较」（例如手写字节 `invokestatic flag()Z; iconst_1; if_icmpeq`；javac 会把源码 `flag() == true` 折成 `flag()`，本层读不到该源码形态）今天的文本是 `flag() == 1`，javac 会拒绝。本 change 的规则（Pair 比较的两个操作数保持各自的证据拼写）**不改变**它：它既不是本 change 引入的，也不在本 change 的证据清单里勾出的三个形状内。实施 MUST 在 verification 里记录该形态的现状（一个手工字节样本的文本与 javac 的处置）作为边界，MUST NOT 声称它已被闭环，也 MUST NOT 为它引入通用推断；若实施发现本 change 的改动**反而**扩大了该形态（例如新产生拒绝文本），那属于本 change 的回归，按既有 refusal 契约处理。

### 4. 受控 fixture：新样本，自带前一轮形状的对照

`javac --release 8` 能产出这三个形状（`if (1 == n)`、`if (0 < n)`、`if (1 < n)` 都不满足 javac 的常规归一化，复核已实测常量在左保留），因此按仓库约定提交**兄弟源码 + 真实编译产物**：`tests/fixtures/p3-int-comparisons/`（`IntComparisons.java`、`v8/IntComparisons.class`、`README.md`、`Baseline.java`）。

样本同时自带：三个缺陷形状；常量在右与另一比较方向的 int 对照（`n == 1`、`n > 0`、`n != 0`）；前一轮的 `Z` 返回、`if (arg0)`、`boolean local1 = …` 与变量操作数比较。这样本 change 的变异与正向对照在一个样本内可读；`p3-boolean-contexts` 与 `tests/p3_boolean_contexts.rs` 保持原状并继续在门禁内运行（前一轮形状的第二重对照）。

精确文本回归落在 `tests/p3_boolean_contexts.rs`（该目标已经拥有这条规则的精确文本与拒绝边界），样本登记进 `tests/p3_execution_comparison.rs` 的 `REQUIRED`，由该文件从本次运行的事实派生声明、以 `javac --release 8` 编译并两侧执行。

### 5. 复用与依赖

不需要新库：两个改动点都在 `condition` 的既有判定与渲染顺序上；`boolean_value`、`boolean_spelling`、`Test`、`render_value` 都已存在；`javac` 仍是测试侧外部 oracle（可以缺席，`p3_execution_comparison` 已是 `#[ignore]`d 的显式门禁）。新增 fixture 会改变 `tests/fixtures/corpus-fingerprint.json` 与 reader census 计数，两者按既有流程显式再生成/更新并记录实测值。

## Risks / Trade-offs

- **只移除左侧转换，漏掉常量在右或另一比较方向** → 对照必须覆盖常量在左/在右与相等/大小两个方向，否则实现仍可能通过。
- **把「未证明 boolean 的零值测试」也改成真值测试** → `nonzero(I)I`、`answer()I` 一类 int 对照 MUST 逐字不变；`condition` 对未证明操作数保持整数比较。
- **顺手扩大拒绝面** → 本 change 不新增拒绝；`flag() == 1` 一类形态按决策 3 记录边界。
- **对照两侧共享同一错误** → 基准侧运行原 class 自己的 driver（`Baseline.java`），不复算呈现侧。
- **语料变更被静默接受** → fingerprint 再生成器与 reader census 都要求记录实测计数。
- **把回归说成新发现** → proposal 与 verification 的措辞 MUST 点名 `5a8c36a` 与 `fa6dc6e` 的对照，不得写成新缺陷。

## Migration Plan

1. 在固定行为基线上先记录三个反例：恢复正文、报告平面、把正文套进正确签名后 javac 的拒绝信息（修正前完成，作为前置证据）。
2. 实施 `condition` 的判定先于转换，保持前一轮 boolean 形状与 int 形态不变。
3. 提交 fixture 与来源 README，执行 fingerprint 再生成与 census 更新，加入精确文本回归、javac 编译/执行对照与边界记录。
4. 跑固定提交门禁并写 verification；同步 delta 与状态引用后归档。

与 `unify-local-type-decisions`、`re-express-string-concatenation` 串行实施（同一 crate，本 change 第 1 个）。回退按本 change 的独立提交进行；回退后必须恢复「整数比较被写成编译器拒绝的 boolean 文本」的公开事实。

## Open Questions

无。`if_icmp*` 之外是否还有别的位置把字面量误判为 boolean，由实施者按决策 1 的清单在当前代码上逐条核对并记录；这不构成需要上游裁决的设计歧义。
