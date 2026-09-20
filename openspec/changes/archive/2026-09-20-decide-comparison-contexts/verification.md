# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。本 change 是本项目**自己的修复造成的回归**的纠正。

## 反例、归属与根因

回归的两个反例（复核者与本会话各自复现）：

| 成员 | 修正前文本 | javac | 修正后（复核者独立复跑） |
| --- | --- | --- | --- |
| `oneFirst(I)I` / `Cmp.litEqual` | `if (true == arg0)` | `incomparable types: boolean and int` | `if (1 == arg0)` |
| `zeroFirst(I)I` / `Cmp.zeroLess` | `if (false < arg0)` | `bad operand types for binary operator '<'`（`first type: boolean` / `second type: int`） | `if (0 < arg0)` |
| `oneLess(I)I` | `if (true < arg0)` | 同 `zeroFirst` | `if (1 < arg0)` |
| `oneLast(I)I` / `zeroLast(I)I` / `nonzero(I)I` | — | — | `if (arg0 == 1)` / `if (arg0 > 0)` / `if (arg0 != 0)` |

修正前这些形状全部报 `Produced`/`Java`/`Structured`/`ContainsStatements`、无诊断——判据只能是编译与执行。

**归属**：这是回归，由本项目的 `5a8c36a`（boolean 上下文定型）引入；`fa6dc6e` 输出正确整数文本。实施者另行对两个版本的 `condition` 做了 diff 确认机制：`fa6dc6e` 按值渲染左操作数且 `Test::Pair` 复用该结果，`5a8c36a` 在 `Test` 选择**之前**就插入了 `boolean_value` → `boolean_spelling`。

## 实现

`crates/jarde-java/src/build.rs::condition` 是唯一同时看到比较形状与操作数证据的位置，现在**先决定位置要求、再渲染任何操作数**：

```rust
let (positive, negative) = match op { … };
let test = if taken { positive } else { negative };
// 只有位置要求本身可以读字面量证据
let boolean = matches!(test, Test::Zero(_)) && builder.boolean_value(operands[0].1, branch_bci);
let left = builder.render_value(operands[0].1, branch_bci, 0)?;
if let Test::Zero(op) = test && boolean {
    let left = boolean_spelling(left);        // 0/1 → false/true 只在这里发生
    return match op { … };
}
match test { /* Zero / Null / Pair：每个操作数按自身证据拼写 */ }
```

- `0`/`1` → `false`/`true` 现在**只**发生在真值测试分支（零测 + 已证明操作数）；`Z` 返回（`return_expr`）与存入已知 boolean 变量（`write_statement`）保留各自既有的适配，未变。
- `Test::Pair` 两侧都按值渲染；右操作数原本也是按值渲染，现在另有回归文本钉住。
- 字面量证据不再在比较形状确定之前被咨询。`boolean_value`/`boolean_evidence`/`boolean_local`/`value_type`/`ExprKind`/`emit.rs` 均未改动（无需打印路径变更）。实施者选的顺序是设计允许的两种之一：先决定 `test` + `boolean`，使 `boolean_spelling` **不可能**从 `Test::Pair` 路径到达。

## 执行对照

`tests/p3_execution_comparison.rs` 新增样本 `p3-int-comparisons/v8` 并登记进 `REQUIRED`：9/9 成员 `Java/Structured`/`contains_statements`、编译通过、`executed: traces identical`，**32 行 trace 两侧一致**、32 行基线逐字匹配（六个比较用输入集 `0,1,2,-1`）。修正前同一次运行在 `oneFirst(I)I` 处以 javac 的 `incomparable types` 失败。

## 变异与对照

- **变异**（恢复旧顺序：`Test` 选择之前先 `boolean_value` + `boolean_spelling`）：`p3_boolean_contexts` 失败于 `an_integer_comparison_keeps_its_integer_literals_on_either_side`（文本退回 `if (true == arg0)`，12 passed / 1 failed）；执行对照在 `oneFirst(I)I` 处失败（javac `incomparable types`）。恢复后 `build.rs` 哈希回到 `c15f603e…`，无 `probe`/`dbg!`/`println!` 残留。
- **对照**：`if (arg0)`（`count`）、`return true;`/`return false;`（`isZero`）、`boolean local1 = arg0;`（`throughLocal`）、变量操作数的 `if (arg0 != 0)`（`nonzero`）全部不变且执行通过；`nestedPlain`、`conditional` 文本不变（`p3_eval_context` 10、`p3_content` 4、`p3_local_rewrite` 7 全绿）；`p3_array_types` 9 绿；`p3-boolean-contexts` 执行行仍绿。

## 记录在案的边界（未声称关闭）

手建 class `Boundary`（`invokestatic flag()Z; iconst_1; if_icmpne`）恢复为 `if (flag() == 1) …`——**修正前后相同**，javac `--release 8` 拒绝（`incomparable types: boolean and int`）。已由测试钉住，并**禁止**把它粉饰成 `flag() == true` / `true == flag()`。它的处置（拒绝 vs 保留）属于下一个 change 的"类型冲突"规则，不属于本 change。

## fixture 与语料

`tests/fixtures/p3-int-comparisons/`：`IntComparisons.java`、`Baseline.java`、`README.md`（命令、尺寸、digest、逐成员字节码、修正前文本、javac 拒绝、边界、前后对照表），`v8/IntComparisons.class`（52.0，**647 字节**，SHA-256 `8b535ab8…583ff6`，用提交源码重编译逐字节相同）。reader census：`(47,203,44,97,8)` → **`(48,213,44,105,8)`**；fingerprint **117 → 120 files**。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1289 passed / 0 failed / 6 ignored**，复核者连跑两次一致（基线 1285，+4） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（35.8 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `openspec validate --all --strict --no-interactive` | 22 passed / 0 failed（归档前） |

## 假设、自定取舍与既有弱点

- **计划里的决定表有两条在字节码层不可表达**：javac 把 `n > 0` 折成 `iload_0; ifle`、`n != 0` 折成 `iload_0; ifeq`，即 `Test::Zero` 作用于**未证明**的操作数，而不是 `Test::Pair`；它们的文本（`if (arg0 > 0)`、`if (arg0 != 0)`）正是计划要求的，已按实际情况钉住并在 fixture README 里写明折叠事实。样本中唯一真正的右侧 `Test::Pair` 是 `n == 1`。实施者没有为凑表而手造 pair。
- **边界的方向由实施者定**：同一混合 pair 在 `if_icmpeq` 下会恢复成 `flag() != 1`（分支方向决定哪个块是 `if` 体），因此选用 `if_icmpne` 以匹配计划记录的 `flag() == 1`。
- **顺带修正的既有文档漂移**（因为正在编辑那些句子）：census 注释的标题计数过时（写着 45/177/44/86/8 而元组是 47/203/44/97/8）——现为 48/213/44/105/8；`tests/fixtures/README.md` 写"114 files"而清单为 117——现为 120。`docs/support-matrix.md` 的 P3 边界段补入整数比较规则（公开陈述被修改的 `java8-recovery` 要求）。
- **既有弱点、未触碰**：`Builder::boolean_value` 仍把 `boolean_literal` 折进一个名为"proven boolean"的共享谓词；本修正后三个调用点都安全，但未来若在**不要求** boolean 的位置调用它会重演本缺陷。三分（值的证据 / 位置要求 / 字面量适配）目前活在 `condition` 的顺序里，而不是谓词的 API 中——下一个 change（`unify-local-type-decisions`）要消费这个三分，故留给它。

## 边界处置已关闭（2026-09-21 补记）

本记录登记的 `flag() == 1` 边界已由下一项 [unify-local-type-decisions](../../2026-09-21-unify-local-type-decisions/verification.md)（或其归档位置）按"类型冲突"规则裁决为**拒绝**：`Test::Pair` 中恰有一侧为已证明 boolean 时在 Java 里无拼写，区域被拒绝并点名 BCI。原文保留，未改写。
