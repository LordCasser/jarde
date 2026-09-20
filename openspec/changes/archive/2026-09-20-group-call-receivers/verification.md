
## 复核者的独立复现（2026-09-20）

同一受控 fixture（`javac --release 8 -g:none`）在修正后由复核者独立复跑：

```text
recover --input Recv.class --policy single-class --class-name Recv \
  --method-name call --descriptor "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"
→ return (arg0 + arg1).substring(1);
```

修正前该输入得到 `return arg0 + arg1.substring(1);`，且报告仍为 `Java/Structured/ContainsStatements/Complete`——结构性平面不覆盖这一类，因此本项的验收只能由执行对照与精确文本断言承担。

## 位置枚举（本 change 的核心证据）

分组是**位置**的属性，不是父节点的属性——这一点是本项的存在理由（上一轮 `binary_operand` 只覆盖二元父节点，因此 receiver 位漏掉了）。实施者从 `Emitter::expr`/`stmt` 逐点枚举并给出可达性：

| 位置 | 需要分组 | 可达性 |
| --- | --- | --- |
| 调用 receiver | 是 | **可达**：拼接链结果做 receiver；本 fixture 的 `call`/`length`/`nested` + 执行对照 |
| 字段读/写 receiver | 是 | javac 不可达（字段 receiver 的类型必须命名 owner，而该子集唯一可构造的引用型二元式是 String 拼接）；printer 单测覆盖 `(arg0 + arg1).f`、`(arg0 + arg1).f = 1` |
| 方法引用限定符 | 是 | 拼接 capture 形态被既有规则拒绝（`jre_lambda_capture`），今天无 javac 可达形态；printer 单测覆盖 `(arg0 + arg1)::length` |
| 数组 indexee | 是 | 不可达（唯一 `Index` 来自 enumswitch 分派表，其 array 是 `getstatic`）；printer 单测覆盖 `(arg0 + arg1)[0]` |
| `!` 操作数 | 是 | 不可达（`Not` 只由 boolean 参数的零测构造，子式恒为 `Local`）；printer 单测覆盖 `!(arg0 + arg1)` |
| **lambda 作 receiver**（实施中发现，不在原清单） | 是 | **可达**：`((ToIntFunction<String>)(s -> s.length())).applyAsInt(x)` 形态；已并入统一规则 |
| 实参、`new` 实参、下标内、lambda 体、条件/selector/锁、return/初始化式 | 否 | 各自被 `,`/`)`/`[]`/`->`/括号/`;` 界定；printer 单测与 fixture 正面对照（`argument`= `wrap(arg0 + arg1)`、`same` 同优先级左结合）逐字不变 |
| 二元操作数 | 是（既有） | 既有优先级/结合性规则逐字段不变 |

实现把规则表述为「位置的 binding 下限」：`PRIMARY=6`（后随 `.`/`::`/`[`）、`UNARY=5`、二元按 Java 分组 4/3/2/1、lambda `0`；`operand(_, least)` 在子式 binding 低于下限时加括号。**括号写在子式自己的节点范围之外、父 span 之内**，单测断言子式 segment 不含括号（`text_of_bci(4) == ["arg0 + arg1", "(arg0 + arg1).substring(1)"]`）。

## 反例、执行对照、变异、正向对照

- **反例（修正前实测）**：`return arg0 + arg1.substring(1);`，平面 `java/structured/contains_statements/complete`、无拒绝；两侧执行：`("a","bc")` 原 `bc` / 恢复 `ac`；`("r","r")` 双方 `r`（**默认输入不暴露该缺陷**——这正是本项要求按成员显式声明输入的原因，已写进 fixture README）；`(null,"bc")` 原 `ullbc` / 恢复 `nullc`；`length` 的未分组文本被 javac 直接拒绝（`String cannot be converted to int`）。
- **执行对照（修正后）**：fixture 8 个成员全部 `Java/Structured`、wrapper 编译、`traces identical`，trace 15 行两侧一致；基线驱动打印 `call("a","bc")=bc`、`call(null,"bc")=ullbc`、`length("a","bc")=3`、`nested(...)=bcdef`、`same("a","b","c")=abc`、`argument("a","bc")=abc` 等。
- **变异**：去掉调用 receiver 分组 → 单测 `9 passed; 3 failed`（文本退回未分组）、`p3_eval_context` 1 failed、执行对照在 `length` 处 javac 报 `incompatible types`；手工对照复现 `ac`/`nullc` 分歧。恢复后 `emit.rs` 与变异前**逐字节一致**（`b063c62c…`），`grep MUTATION` 为 0。
- **正向对照（逐字不变）**：`arg0.foo()`、`arg0.trim().length()`、`arg0 + arg1 + arg2`、`arg0 + arg1 * arg2`、`!arg0 == arg1`、`wrap(arg0 + arg1)`，以及 `nestedPlain` = `arg0 + 1 + (arg0 + 2)`。

## fixture 与语料

`tests/fixtures/p3-receiver-grouping/`：`ReceiverGrouping.java`、`Baseline.java`、`v8/ReceiverGrouping.class`（52.0，1038 字节，SHA-256 `8e030b53…30cc8c00`）、README（命令、逐成员字节码、对照表、复现）。census **`(44,168,44,86,8)` → `(45,177,44,86,8)`**；fingerprint **108 → 111 files**；`tests/fixtures/README.md` 新增行并把过期计数修正为 111。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1264 passed / 0 failed / 6 ignored**，复核者连跑两次一致（基线 1258，+6） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（22.6 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `openspec validate --all --strict --no-interactive` | 22 passed / 0 failed（归档前） |

## 边界

- **没有形状需要走拒绝路径**：所有位置都能靠分组打印回同一棵树，计划里的拒绝分支未触发。
- **既有类型缺口（未触碰、非本 change）**：lambda 作 receiver 时文本现在**树忠实**但 Java 需要 target type 才能编译，而 AST 没有 cast/target-type 节点（约束禁止新增节点），拒绝它又要改 `build.rs`（本任务禁区）。修正前该文本既不忠实也不能编译；现在只是仍不能编译。如实登记为既有弱点。
- 未改 `README.md`/`docs/support-matrix.md`：两份文档没有把 receiver 分组写成"已覆盖"或"缺口"，不因此变旧。
