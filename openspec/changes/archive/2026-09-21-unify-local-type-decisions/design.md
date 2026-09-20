## Context

动机与复现见 [proposal](proposal.md)。已知事实（来自 [完成复核](../../completion-review.md) 的 T3 一节，本 change 按当前代码位置核对过一次）：

- **就地声明路径**：`crates/jarde-java/src/build.rs::declare`（约 1618 起）用 `self.boolean_evidence(stored) || self.boolean_local(stored, at)`（约 1647）决定类型：descriptor 事实，或「stored 是一次对**本 body 已声明 boolean 的局部**的读取」。`boolean_local`（约 1512）只认一次 `Load`／该槽在 `at` 的 entry/phi，并刻意**不跟随值链**。
- **提升声明路径**：`declarations()`（约 203 起；类型决定约 276–290）只用 `boolean_proof`（约 3104）：`Z` 参数槽、返回 `Z` 的调用、descriptor 为 `Z` 的字段读取。它的注释自己写明它**不**带字面量，也不带「本 body 声明的 boolean 局部」——因为它在任何语句写出来之前运行；而**就地**路径在写出语句时读到的正是「已经写出的声明」。
- 因此同一个值（例如「一次对已声明 boolean 的局部的读取」）在两条路径上得到两个答案：就地 `boolean`，提升按帧的 `int`；结论因此取决于该变量是否被提升，也就是取决于**遍历顺序**。
- **写入路径** `write_statement`（约 1556 起）在变量已有声明时直接赋入渲染后的值，没有「值的类型与变量声明是否一致」的检查；于是 `int local3` 之后可以出现 `local3 = <boolean 值>`，产物在成员自己的签名下被 javac 拒绝（复核实测）。已有的一处例外是「写入本 run 已知 boolean 的变量」：`None if self.boolean_variable(variable)` 分支（约 1580–1594）已经会对无证据的值拒绝；本 change 把这条检查对齐到计划结果，并补上反方向（被本层以 boolean 呈现的值写进按声明是其它类型的变量）。
- **`declare()` 的返回值有歧义**：`Ok(None)` 同时表示「没有声明到期」（`build.rs:1621` 已声明/参数、`1628` 无名）与「声明已经失败、fallback 已写」（`1653`/`1670` 的两个 `self.fallback(...)?; return Ok(None);`）；`write_statement` 在 `None` 分支继续写赋值（`build.rs:1580`/`1598`）。两种含义 MUST 分开。
- 上一轮变更（[type-boolean-contexts](../archive/2026-09-20-type-boolean-contexts/proposal.md)）把「字面量不作声明证据」当作**实测结论**记录：把字面量加进声明决策会让 `p3-scope` 的 `scope(Z)I` 从 `Java`/`Structured` 降级为 `Mixed`/`Fallback`，并让 `intLocal(I)I` 同样退化；其 fixture README 也写明「提升路径只从 descriptor 证明」这一边界。本 change MUST NOT 悄悄重新接纳它。

## Goals / Non-Goals

**Goals:**

- 一个局部变量的类型只决定一次：在既有声明规划里产出一个按 `LocalVariable` 身份索引的类型结果，被提升声明、就地声明、赋值、条件与返回消费。
- 决定的证据有限且写清：descriptor 事实 + 「一次对本 run 已判为 boolean 的局部的读取」的传播；**每个写入**都按决定检查一致性；传播用有界工作队列并接线到既有预算/取消。
- 未知或冲突的类型终止该结构（既有 refusal 契约）；被本层以 boolean 呈现的值不会进入按声明是其它类型的变量。
- 两个形状与对照进入常备回归：修正前记录（文本与 javac 拒绝），修正后精确文本 + 编译 + 执行对照；变异让「提升路径只读 descriptor」或「只读第一次写入」时 MUST 变红。

**Non-Goals:**

- 不建类型系统、不做数据流推断、不引入 `Type` 新变体或新的拒绝词汇；拒绝走既有 fallback/refusal 契约。
- 不改 `value_type` 对其它类型的决定，不改 `emit.rs` 的打印规则，不改 `reuse::Plan` 的变量身份模型。
- 不重开 `type-boolean-contexts` 的其它结论（本 change 只把它的声明边界补齐并说清），不重开 R8/R9、递归界、concat、数组类型与 boolean 上下文修正。
- 不声称类型保真：literal-armed 形状的类型与源码不同这一事实 MUST 被记录为边界，MUST NOT 被表述为闭环。
- 不做性能工作；不修 body 解码重新解析类的债务。

## Decisions

### 1. 一个所有者：声明规划产出一个按变量身份索引的类型结果

类型结果在既有声明规划里计算（`declarations()` 是今天唯一在语句构建之前看到全部 `SlotUse` 的地方：它已经收集每个 `LocalVariable` 的每个读/写、每条的 region path 与 BCI），并按既有 `reuse::Plan` 的 `LocalVariable` 身份索引——不新建变量身份、不新建规划阶段。

消费点 MUST 只有一处决定、五处读：

| 消费点 | 今天的行为 | 本 change 后 |
| --- | --- | --- |
| 提升声明（`HoistedDeclaration`） | 只读 `boolean_proof`（descriptor 证据） | 读计划结果 |
| 就地声明（`declare()` 的布尔分支） | `boolean_evidence \|\| boolean_local`（只认识已写出的声明） | 读计划结果（同一份证据的闭合形式） |
| 赋值（`write_statement` 的 `None` 分支） | 直接赋入，无一致性检查 | 按计划结果检查兼容性；矛盾即终止 |
| 条件（`condition` 的真值测试分支） | `boolean_value`（含已写出的 boolean 局部） | 读计划结果（[decide-comparison-contexts](../decide-comparison-contexts/proposal.md) 已把判定顺序定在渲染之前） |
| 返回（`Z` 方法返回的拼写与拒绝） | 现有证据清单 | 读计划结果 |

### 2. 有限证据清单与「发起 / 适配」的区分

| 证据 | 是否发起 boolean 决定 | 说明 |
| --- | --- | --- |
| `Z` 参数槽的读取 | 是 | descriptor 事实（`boolean_parameter` 的既有读数） |
| 返回 descriptor 为 `Z` 的调用结果 | 是 | callee descriptor |
| descriptor 为 `Z` 的字段读取 | 是 | field claim |
| 一次对本 run **已判为 boolean 的局部**的读取 | 是（传播，一次读取一跳） | 与本 body 声明链同一读数；MUST NOT 跟随值链（一个 store 的 own value、phi 的多个 push 仍不证明） |
| `0`/`1` 字面量 | **否** | `int x = 0;` 与 `boolean c = true;` 是同一份字节；它的证据是帧的 `int` 形状 |
| 其它值 | 否 | 帧声明的类型（非 boolean）或未知 |

**适配规则**：决定为 boolean 之后，该变量的写入里 `0`/`1` 字面量按 `true`/`false` 拼写，其它被证明 boolean 的值照写；决定为 `int` 之后，`0`/`1` 保持 `1`/`0`。`0`/`1` MUST NOT 反向把变量定成 boolean。判定顺序：先由 descriptor 与已判定局部的传播决定 boolean；否则按帧的类型（`value_type`）决定非 boolean 类型；没有可用类型即「未知」。

### 3. 每个写入都检查，而不只是第一次写入

`declarations()` 今天只取首个写入（按 region path 与 BCI 排序的第一个 `written`）。本 change 沿用「首个写入说出类型」的既有读法（就地路径与提升路径今天都以此为据），但**决定一旦作出，该变量的每一个写入都 MUST 按它检查兼容性**：写入值必须能被拼成决定的类型（boolean 决定下：`0`/`1` 字面量、被证明 boolean 的值；`int` 决定下：`0`/`1` 字面量与其它非 boolean 的值），否则该结构 MUST 终止（决策 5）。这样 `if (n == 0) c = a; else c = b;` 的两个分支都被检查，而不是只检查第一个。

### 4. 传播用有界工作队列，接线到既有预算与取消

证据沿「局部读取局部」传播（`a` boolean → `c` boolean → `d` boolean），因此实现用一个工作队列迭代到稳定：

- 入队的是「其决定可能改变某个写入者或某个使用者」的变量（初值：全部有 `Z` descriptor 证据的变量）；
- 每个条目在处理前 `charge(budget, CountedBudgetDimension::IrItems, 1, Some(at))`，并在循环里 `poll(budget, Some(at))`（`crates/jarde-java/src/stop.rs` 的既有两个入口，`build.rs` 已在用）；`at` 取该变量自己的写入 BCI；
- 事实单调（一个变量从「未知」只可能变成某个具体类型），队列条目数由「变量 × 写入」的有限集合界定；预算或取消在规划期停止时返回既有 `StopReason`，由既有路径发布停止，MUST NOT 继续构建或提交部分产物。

不使用递归（递归深度与变量数无关），也不新增预算维度；`IrItems` 是既有维度，规划的工作量与它同量级。

### 5. 未知或冲突终止结构，`declare()` 的两种结果分开

- 「未知」（帧不给出可拼写的类型、或没有任何写入可决定类型）与「冲突」（某个写入的值与已决定的类型不一致、或其证据与另一写入矛盾）都 MUST 终止该结构：走既有 refusal/fallback 契约（保留 bytecode 与 origin、`representation=Mixed`、`quality=Fallback`、诊断点名 BCI 与物理方法），MUST NOT 发布猜测或类型矛盾的赋值。改类型不是选项：换一个更强的类型会让两条声明路径重新分叉。
- `declare()` 的结果 MUST 把「没有声明到期」与「声明失败（fallback 已写）」分成两个可区分的结果（例如三值枚举：`Declared(ty)` / `NotDue` / `Refused`）；`write_statement` 在 `Refused` 之后 MUST NOT 继续写那条赋值。这是「未知类型不落地」在调用点上的可观察形式：拒绝之后不再发射。

### 6. literal-armed 形状：记录边界，不重新接纳字面量

`boolean x; if (b) { x = true; } else { x = false; } if (x) …` 的现状是 `int local1; local1 = 1;`，本 change 后的决定是**保持 `int`** 并保证它自洽：

- 就地路径对**同一个值**（`0`/`1` 字面量）作同样决定（帧的 `int`），两条路径一致；重新接纳字面量是决策 2 排除、且上一轮有实测代价的做法。
- 文本在成员自己的签名下可编译，且两侧执行结果一致（`local1 = 1` / `local1 != 0`）；拒绝一个值正确的产物与「不用普遍拒绝换反例通过」冲突。
- 只有当该值在某个位置被**要求**是 boolean（例如 `Z` 方法的 `return`）时，证据不足才按既有规则拒绝；这条路径已经存在，本 change 不改它。
- 因此 verification MUST 记录：该形状的声明类型与源码不同、它是类型保真边界而不是语义闭环；两条路径在它上面给出的答案相同。若实施发现保持 `int` 会发布任何类型矛盾的赋值，那正是决策 5 的终止路径要处理的，MUST NOT 为了「保持现状」而放行。

### 7. 受控 fixture：新样本，形状与对照同处一份

三个形状都可以由 `javac` 产出，因此按仓库约定提交兄弟源码 + 真实编译产物：`tests/fixtures/p3-hoisted-boolean/`（`HoistedBoolean.java`、`v8/HoistedBoolean.class`、`README.md`、`Baseline.java`）。规划期候选成员（成员名、正文与精确文本由实施者按实测记录，下表只用于验收对照，不得据此反推）：

| 成员 | 形状 | 修正前 | 修正后 |
| --- | --- | --- | --- |
| `copied(boolean b, int n)I` | `boolean a = b; boolean c; if (n == 0) c = a; else c = b; if (c) …`（复核样本） | `int local3; … local3 = local2; … local3 = arg0; … if (local3 != 0)`，javac 拒绝 | `boolean local3; …`，javac 接受，两侧一致 |
| `relayed(boolean b)Z` | `boolean x; boolean y; if (b) { x = b; y = x; } return y;`（跨分支读取） | `int local2/3` 类，javac 拒绝 | `boolean …`，javac 接受 |
| `swapped(boolean b, int n)I` | 与 `copied` 同形状但**交换分支顺序** | 与 `copied` 同类 | 与 `copied` 得出同一类型与同一文本（除分支文本本身） |
| `literalArmed(boolean b)I` | `boolean x; if (b) x = true; else x = false; if (x) return 1; return 0;` | `int local1; local1 = 1;`（自洽） | 逐字不变（记录的边界） |
| `fromParameter(boolean b)Z` | `boolean c; if (…) c = b; else c = b; return c;`（descriptor 证据的既有路径） | `boolean` | 逐字不变（正向对照） |
| `intLocal(int n)I` | `int x = 0; … x = 1; …`（字面量不给类型） | `int` | 逐字不变（正向对照） |
| `unproven(boolean b)Z` | 提升变量的值在 `Z` 返回处无 boolean 证据 | 拒绝 | 拒绝（同一边界） |

回归落点：新建 `tests/p3_hoisted_boolean.rs`（精确文本与拒绝边界），样本登记进 `tests/p3_execution_comparison.rs` 的 `REQUIRED`；`p3-boolean-contexts` 与其测试保持原状并继续在门禁内运行（第二重对照）。

### 8. 复用与依赖

不需要新库：改动都在 `declarations()`／`declare()`／`write_statement` 的既有读法与 `stop.rs` 既有计费入口上；`javac` 仍是测试侧外部 oracle。新增 fixture 会改变 `tests/fixtures/corpus-fingerprint.json` 与 reader census 计数，两者按既有流程显式再生成/更新并记录实测值。

## Risks / Trade-offs

- **把「本 run 证明 boolean 的局部」按值链闭合** → 明确只闭合「一次读取 + 该变量自身的决定」，并在 verification 记录不跟随的边界（一个 store 的 own value、phi 合并值仍不证明）。
- **过度拒绝** → `intLocal`、`fromParameter`、`literalArmed`、`p3-boolean-contexts` 全样本继续通过编译执行对照；终止只发生在未知/冲突处。
- **规划变成新的无限工作量** → 决策 4 的队列条目由有限集合界定并逐条计费；预算停止走既有发布路径。
- **literal-armed 形状被当成已修好** → 决策 6 要求把它记录为类型保真边界，并在测试里断言它的文本与两侧执行一致，而不是声称类型正确。
- **`declare()` 的结果重构改变既有调用语义** → `NotDue` 的语义与今天一致（参数、已声明、无名）；只有 `Refused` 之后的赋值被去掉，且由回归与 reject 用例核对。
- **语料变更被静默接受** → fingerprint 再生成器与 reader census 都要求记录实测计数。

## Migration Plan

1. 在固定行为基线上先记录三个形状：恢复正文、报告平面与把正文套进正确签名后 javac 的拒绝信息（literal-armed 记录其自洽文本）。
2. 复核决策 7 表候选成员在当前代码上的类型决定与两条路径的证据入口，写出最终证据清单与每个形状的处置（呈现或拒绝）。
3. 实现规划里的类型结果、消费点接线、写入一致性检查与 `declare()` 的结果区分，保持就地路径的 `int` 对照不变。
4. 提交 fixture 与来源 README，执行 fingerprint 再生成与 census 更新，加入精确文本回归、javac 编译/执行对照、拒绝边界与变异。
5. 跑固定提交门禁并写 verification；同步 delta 与状态引用后归档。

与 `decide-comparison-contexts`、`re-express-string-concatenation` 串行实施（同一 crate，本 change 第 2 个）。回退按本 change 的独立提交进行；回退后必须恢复「跨分支复制的 boolean 局部被声明成 `int` 并赋入 boolean 值」的公开事实。

## Open Questions

无。规划里传播的具体实现形态（队列条目与闭合的写法）属于执行范围；`Z` 返回形状里哪些合并值能带 boolean 证据仍由上一轮记录的边界决定（不能证明即拒绝），本 change 不改。
