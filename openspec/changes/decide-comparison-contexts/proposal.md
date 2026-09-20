## Why

整数二元比较被错误地 boolean 化，产物是编译器拒绝的文本（[完成复核](../../completion-review.md) 的 T2 行与其「boolean 转换发生在确认比较上下文之前」一节；本 change 采用该行记录的数值，并已按当前代码位置复现）：

```java
public static int oneFirst(int n) { if (1 == n) return 3; return 4; }
public static int zeroFirst(int n) { if (0 < n) return 3; return 4; }
public static int oneLess(int n) { if (1 < n) return 3; return 4; }
```

恢复正文分别是 `if (true == arg0)`、`if (false < arg0)`、`if (true < arg0)`，报告为 Complete/Structured，把正文放进成员自己的签名后 javac 一律拒绝。

**这是一条本项目自己引入的回归：本 change 明确承认并闭环它，不把它当作新发现。** 复核用 `git archive fa6dc6e` 导出的独立源码与独立 target 构建了修正前的 CLI，这三个形状当时都保持正确的整数文本；回归由 `5a8c36a`（已归档的 [type-boolean-contexts](../archive/2026-09-20-type-boolean-contexts/proposal.md)）引入。

根因（复核的表述，本 change 已按当前源码核对）：`crates/jarde-java/src/build.rs::condition`（约 3250 起）先算 `let boolean = builder.boolean_value(operands[0].1, branch_bci)`——该谓词包含 `boolean_literal`（`build.rs:1495`）——并按它把左操作数 `boolean_spelling` 成 `true`/`false`，**之后**才在 `Test::Zero` 与 `Test::Pair` 之间取舍（`build.rs:3279`）；`Test::Pair` 直接复用已经拼过的 `left`（`build.rs:3348`）。`if_icmp*` 的两个 int 操作数没有 boolean 上下文，左侧的 `0`/`1` 不能据此变成 `false`/`true`。

缺陷的底层形态（复核的框架）：**同一位置的类型证据、目标位置的要求与字面量适配被混在一起**，于是「这个比较的结果是 boolean」被当成了「这个比较的操作数是 boolean」。本 change 把这一个决定交回唯一所有者：`condition` 在拼写任何操作数之前，先按比较的形态与目标位置判定「这个位置是否要求 boolean」，再按该要求与每个操作数各自的证据拼写它们。

## What Changes

- **判定的顺序**：某个位置是否要求 boolean MUST 在该位置的**全部**操作数被拼写之前判定；`0`/`1` 到 `false`/`true` 的映射只发生在目标位置已经要求 boolean 的那一处（`Z` 方法的 `return`、操作数被证明为 boolean 的零值测试、写入已证明为 boolean 的变量的值）。
- **两个操作数按同一规则**：整数二元比较（`if_icmp*`）的两个操作数 MUST 保持各自的整数文本（`1 == arg0`、`arg0 == 1`、`0 < arg0`、`arg0 > 0`），常量在左、常量在右、相等与大小比较一律相同；比较的结果是 boolean 这一事实 MUST NOT 被当作操作数的类型证据。操作数被**证明**为 boolean 的零值测试仍写成真值测试（`if (b)`、`if (!b)`）。
- **回归归属**：本 change 只纠正 [type-boolean-contexts](../archive/2026-09-20-type-boolean-contexts/proposal.md) 引入的这条回归，前一轮的 boolean 形状（`return true;`、`if (flag())`、`boolean local1 = …`）与 int 形态逐字保留，不重开该 change 的其它结论，也不重开任何其它已归档 change。
- **验收**：三个缺陷形状先作为**现在就会失败**的常备回归记录（精确文本 + 成员自己签名下的 javac 编译 + 两侧执行对照），修正在其后；常量在左/在右、相等与大小比较都被对照覆盖；变异把字面量映射放回测试形态判定之前必须让回归变红；正向对照覆盖前一轮的 boolean 形状与变量操作数的 int 比较，MUST NOT 只以 `n != 0` 证明「int 对照不变」。

## Capabilities

### New Capabilities

无。不新增 crate、公共模块、报告平面、诊断 code 或停止分支。

### Modified Capabilities

- `java8-recovery`：「Boolean contexts are presented as booleans」补上判定的**顺序与操作数范围**——映射只发生在目标位置要求 boolean 处，整数二元比较的两个操作数保持整数文本；同一字面量由它所在的上下文决定拼写；并写明本修正是对本项目上一轮 boolean 修正所引入回归的纠正，不是新发现。
- `recovery-validation`：「A boolean context's artifact compiles and agrees when executed」把这条回归纳入编译执行验收：常量在左/在右、相等与大小比较在派生声明下 MUST 被 javac 接受、两侧执行 MUST 一致，且变异必须恢复判定顺序错误。

## Impact

实现落在 `crates/jarde-java/src/build.rs::condition`（判定与拼写的顺序；`boolean_value`/`boolean_evidence`/`boolean_literal`/`boolean_local`、`Test`、`boolean_spelling`、`value_type` 与 `emit.rs` 的打印规则只读核对，不新增推断）。本 change 与 [unify-local-type-decisions](../unify-local-type-decisions/proposal.md)、[re-express-string-concatenation](../re-express-string-concatenation/proposal.md) 都改 `crates/jarde-java`（`build.rs`、`emit.rs`；本 change 落 `build.rs`），三者 MUST 按此顺序**串行**实施（本 change 为第 1 个），彼此不并行改动同一文件；后两个 change 消费本 change 定下的「值自身的证据 / 目标位置的要求 / 字面量适配」三分，但本 change 不替它们作决定。

交付包含受控 fixture（源码、class 字节、来源 README、`tests/fixtures/README.md` 登记、fingerprint 再生成、reader fixture census 更新）、精确文本回归、javac 编译与执行对照、变异与正向对照。反例 MUST 在修正前先记录（命令、正文、javac 的拒绝信息）。

非目标：不新增 crate、依赖或 verifier；不建通用类型系统或推断，不使用返回 descriptor、参数槽类型与既有 callee/field descriptor 之外的证据；不改 `value_type` 对其它类型的决定，不改 `emit.rs` 的打印规则；不 spawn 工作线程、不使用 `catch_unwind`；不新增拒绝词汇、报告平面或停止分支；不声称覆盖整类类型缺陷；不重开 R8/R9、已归档的递归界、concat、数组类型与停止传播修正；不做性能工作（`optimize-demand-workloads` 保持 0/22）。当前仅完成修正规划，实施任务全部待办；历史归档与既有验证记录保持原状。
