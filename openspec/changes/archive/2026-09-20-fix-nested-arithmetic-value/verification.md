# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。反例由 benchmark 战役首次发现，根因与四形状由本会话独立定位并实测。CI 结果随后回填。

## 根因（发射器，不是值分析）

`crates/jarde-java/src/emit.rs` 的二元表达式打印此前只做拼接，没有优先级判断、没有括号：**树是对的，文本是另一个程序**。修正后每个操作数经同一个 helper 处理：

```rust
ExprKind::Binary { op, left, right } => {
    emitter.binary_operand(left, *op, Side::Left)?;
    emitter.put(&format!(" {} ", op.spell()), at)?;
    emitter.binary_operand(right, *op, Side::Right)
}
```

规则是「helper + 小型优先级表」：`binding()` 给出 Java 自己的四组（`* / %` = 3、`+ -` = 2、关系 = 1、相等 = 0），全部左结合，因此左子式优先级更低才加括号、右子式更低**或相等**才加括号；非二元操作数（调用、字段/数组读、字面量、`!`）绑定更紧，从不加括号。私有 AST 未改，`build.rs` 的渲染/拒绝逻辑未改，节点 origin 未改。

覆盖面：所有表达式都经 `Emitter::expr` 到达文本，而 `op.spell()` 全仓只有一处调用点，因此该臂覆盖语句值、`if`/`while`/`do` 条件、`switch` selector、`synchronized` 锁、调用与构造实参、lambda 体、数组下标、字段 receiver，以及 `build.rs::concat_expr` 构造的 `Binary::Add` 链。

## 四形状：修正前 → 修正后（我独立复跑）

| 成员 | 修正前文本 | 修正后文本 |
| --- | --- | --- |
| `inverse32(I)I` | `local1 = local1 * 2 - arg0 * local1;` ×4 | `local1 = local1 * (2 - arg0 * local1);` ×4 |
| `scaledDifference(II)I` | `return arg0 * 2 - arg1 * arg0;` | `return arg0 * (2 - arg1 * arg0);` |
| `nestedDifference(III)I` | `return arg0 - arg1 - arg2;` | `return arg0 - (arg1 - arg2);` |
| `nestedQuotient(III)I` | `return arg0 / arg1 * arg2;` | `return arg0 / (arg1 * arg2);` |
| `differenceOfSum(II)I` | `return arg0 - arg1 + 1 * 2;` | `return arg0 - (arg1 + 1) * 2;` |
| `productOfSum(III)I` | `return arg0 + arg1 * arg2;` | `return (arg0 + arg1) * arg2;` |
| `sumOfProducts(III)I` / `leftNestedSum(III)I` | 对照 | 不变（同优先级左结合，无需括号） |

修正前这些形状全部报 `Java`/`Structured`/`ContainsStatements`/`Complete`/`Produced`，无拒绝、无错误诊断——正是「三个平面都为真而文本计算另一个值」的情形。

## 执行对照（我独立做的版本）

把 `inverse32` 的恢复体包成方法，与原 class 并排编译执行：

```text
d=-1 original=-1            recovered=-1            MATCH   ← 修正前此处为 -81
d=7  original=-1227133513   recovered=-1227133513   MATCH
d=0  original=0             recovered=0             MATCH
d=3  original=-1431655765   recovered=-1431655765   MATCH
d=-9 original=-954437177    recovered=-954437177    MATCH
```

ignored 对照（`p3_execution_comparison`，新增样本 `p3-nested-arithmetic/v8`）24 行 trace 两侧一致；提交的 baseline driver 打印 `inverse32(-1)=-1` 等十个值。

## 回归、变异与既有期望的移动

- **反例先行**：新用例 `the_printed_text_keeps_the_arithmetic_tree` 在修正前失败，逐形状报「文本必须恰好出现 `X`，实际 0 次」与「文本仍含 `Y`，它解析回另一棵树」；两条对照无告警。
- **变异**：把分组判定强制为 `false`（其余不动）→ 同一断言以同样六条失败，且执行对照在 `the two bodies' observable traces differ` 处失败（原 `-1227133513` 对 `4375`）；变异体实跑 `inverse32(-1)=-81`。`emit.rs` 变异前后哈希一致、`grep MUTATION` 为空。
- **三处既有精确文本期望按理由移动**：`nestedPlain` 是 `(x + 1) + (x + 2)`，其右操作数是**同优先级**的和式，要保树就必须写成 `return arg0 + 1 + (arg0 + 2);`（旧文本 `arg0 + 1 + arg0 + 2` 值等价但非树忠实）。更新位于 `tests/p3_eval_context.rs`、`tests/p3_content.rs`、`crates/jarde-cli/tests/json_cli.rs`；平面声明（Java/Structured、无 quote、`contains_statements`）与执行 trace 均未变。归档的 `close-recovery-correctness-gaps/verification.md` 仍引旧串，作为历史记录不改写。

## fixture 与语料

`tests/fixtures/p3-nested-arithmetic/`：`ModLike.java`（8 个静态方法：`inverse32` 为四轮 `i = i * (2 - d * i)`；四个缺陷形状；`productOfSum` 见证左子式分组；两个对照）、`Baseline.java`、`README.md`、`v8/ModLike.class`（javac 23.0.1，`--release 8 -g:none`，52.0，568 字节，SHA-256 `a5827ef4865e7d33aff6ea0c34c46194a69f2436e011f5ad5c848853d2891158`，我独立复核一致）。`inverse32` 的块字节为 `iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1`。

普查（`crates/jarde-reader/src/classfile.rs`，本 change 唯一触碰该 crate 的位置）：**(43, 159, 44, 86, 8) → (44, 168, 44, 86, 8)**（+1 class、+9 body）。指纹重生成：**105 → 108 files**（新增三个 fixture 文件），`tests/fixtures/README.md` 新增一行并把已过期的文件数 93 → 108 修正。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test p3_eval_context --locked` | 9 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1258 passed / 0 failed / 6 ignored**，连跑两次一致（基线 1256，+2 新用例） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（19.5 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| source-map 与 accessor 回归（`-p jarde-java --lib`、`p3_accessor_edges`） | 58 / 4 passed |
| `openspec validate --all --strict --no-interactive` | 21 passed / 0 failed（归档前） |

## 边界

- **拒绝分支未触发**：所有形状都可证明，分组即足够，没有形状需要走拒绝词汇。已检查本来可能需要它的形状：二元之下的比较（同一规则加括号）、负字面量操作数（`a - -1` 合法为两个 token）、`Not`（其子式在本子集里恒为 boolean 参数的 `Local`，故 `!` 今天不可能包住二元式）。拒绝路径本身未改，仍由既有 `nestedLocal`/`nestedCall` 用例覆盖。
- `concat` 链的打印路径没有「被追加操作数本身是二元式」的已提交样本，该组合由唯一打印臂推理而非实测——若要，需要一个新的受控样本。
- 顺带修正、本次带出的既有过期：`tests/p3_content.rs` 的普查注释（原 `(41, 153, …)`，实际断言为 `(43, 159, …)`）与 `tests/fixtures/README.md` 的文件数（93 vs 105）。
- 设计里带入的两条**测量侧**遗留属于测量方，不在本 change：其对照臂在 `comparePriorities(II)I` 上的参数顺序 bug（排除后应有 785/784），以及 103 条 `does_not_compile` 需在修复后重跑该桶。
