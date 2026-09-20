# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。反例由 benchmark campaign 首次报告并经复核者独立复现；第三站点由实施者在本 change 内发现并实现。

## 反例与根因

受控 fixture（`javac --release 8 -g:none`）复现的三个位置：

| 成员 | 修正前文本 | javac | 修正后 |
| --- | --- | --- | --- |
| `isSCSV(I)Z` | `return 1;` / `return 0;` | `int cannot be converted to boolean` | `return true;` / `return false;` |
| `flag()Z` | `return 1;` | 同上 | `return true;` |
| `condFromCall()I`（条件为 boolean 调用结果） | `if (flag() != 0)` | `incomparable types: boolean and int` | `if (flag())` |
| `staticFlagCount()I`（条件为 `Z` 字段读取） | `if (BooleanContexts.staticFlag != 0)` | 同上 | `if (BooleanContexts.staticFlag)` |
| **`throughLocal(Z)Z`**（第三站点） | `int local1 = arg0; return local1;`（后来变为 BCI 3 拒绝） | `boolean cannot be converted to int` / `int cannot be converted to boolean` | `boolean local1 = arg0; return local1;` |
| `localFromCall()Z` | `int local0 = flag(); return local0;` | 同上 | `boolean local0 = flag(); return local0;` |
| `pick(IZZ)Z`、`fromLocal(Z)Z` | 声明为 `int`（提升路径同样） | 同上 | `boolean local3` / `boolean local1; boolean local2 = local1;` |
| `intLocal(I)I`（对照） | — | — | `int local1; local1 = 0; …`（不变） |
| `overwrite(Z)Z`（手建） | `int local1 = arg0; local1 = 2;` | 无效 | `boolean local1 = arg0;` + BCI 3 拒绝 |

修正前这些都报 `Java`/`Structured`/`contains_statements`/`complete`、无拒绝诊断——结构性平面看不到这一类，因此验收只能靠**编译与执行对照**。

根因（复核已定位）：`build.rs` 的 `Return` 分支按值渲染而未按返回 descriptor 定型；`condition` 的 boolean 特判只识别 boolean **参数**；第三站点是 `Builder::declare` 读 **store** 的 SSA 值（而谓词要求 `load`），因此没有任何局部能被声明为 `boolean`。

## 实现

- **返回类型入线**：`report.rs` 从 descriptor 读出 `returns_boolean`（`(…)Z`）并经 `build::Inputs` 交给构建层，写语句的层不再自行读 descriptor。
- **`Return`**：`Z` 方法里，带 boolean 证据的值写成 `true`/`false`；无证据时**拒绝**（保留值自身的更具体拒绝理由）。
- **`condition`**：一次计算 boolean 证明，同时用于拼写与既有的零测分支（原先只看参数）。
- **声明/赋值（第三站点）**：`declare` 改读**被存入的值**（store 的操作数），按 `boolean_evidence || boolean_local` 定型；两条 store 路径共用 `write_statement`；**把无 boolean 证据的值存进已知 boolean 变量同样拒绝**并点名变量。`declarations()`（提升路径）与就地路径共用同一个 `boolean_proof` 读法；`Builder::boolean_parameter` 在合并后成为死代码，已删除，`parameter_boolean` 是「`Z` 参数的 load」的唯一读者。
- **无 AST/发射器改动**：`ast.rs` 早有 `Type::Boolean` 且 `spell()` 返回 `boolean`，`emit.rs` 本就打印 `ty.spell()`——本 change 只是让 `build.rs` **产出**它，因此"AST 改动超出拼写"的分支没有触发。

## 证据来源清单（实施者定稿，经复核）

| 证据 | 采用 | 说明 |
| --- | --- | --- |
| `Z` 参数的 load | 是 | P3-R5 既有事实，单一读者 |
| callee descriptor 返回 `Z` 的调用 | 是 | 与 `typed_arguments` 同一读法 |
| **被认领字段读取的 descriptor 为 `Z`** | 是（**计划外，复核接受**） | 不加它 `staticFlagCount` 仍输出 javac 拒绝的文本，而今天能编译的 `fieldFlag()Z` 会变成新拒绝——修一个缺陷换来一个回归；该 descriptor 来自主张自身的证据，与 callee 同类，不是推断 |
| `0`/`1` 字面量 | 仅在有上下文时（返回/条件/存入已知 boolean 变量） | **不用于声明决策**：实测若用于声明，`p3-scope` 的 `scope(Z)I` 会退化为 Mixed（`int x = 0;` 与 `boolean c = true;` 字节相同且新局部无上下文） |
| 本 body 已声明 boolean 的局部的读取 | 是（**比"不做 SSA 追索"宽一跳**） | 不加它 `fromLocal` 会印 `int local2 = local1;`（boolean→int，javac 拒绝）——同一无效文本缺陷。边界仍守住：不做定义链行走（`replaced_by`、store 自身值、phi 操作数、`!` 合并） |
| `!` 作为输入证据 | 否 | 取反是分支的**语义方向**，不是被读取的值；`negated(Z)Z` → `if (!arg0)` 仍产出 |

## 变异与正向对照

- **变异 A**（回退返回站点）→ `p3_boolean_contexts` 2 failed（`return 1;`/`return 0;`、`intReturn` 由拒绝退回 Java），执行对照在 `isZero` 处 javac 拒绝。
- **变异 B**（条件只认参数）→ 3 failed（`if (flag() != 0)`、`if (1 != 0)`、`if (staticFlag != 0)`），执行对照在 `parity` 处 javac 拒绝。
- **变异 D**（去掉声明定型）→ 2 failed（`throughLocal` 由 Java 退回 Mixed 并印 `int local1 = arg0;`；`overwrite` 把无证据的 store **发布**出来），执行对照红。
- **变异 D2**（只去掉"已声明 boolean 局部"这一项）→ `fromLocal` 印出 `boolean local1 = arg0; int local2 = local1;`，javac 拒绝——这正是保留该项的依据。
- 恢复：`build.rs`/`report.rs` 哈希与变异前逐字节一致，`grep MUTATION|dbg!` 为空。
- **正向对照**：`intLocal(I)I` 仍 `int local1;`、`nonzero(I)I` 仍 `if (arg0 != 0)`、`answer()I` 仍 `return 1;`、`count(Z)I` 仍 `if (arg0)`；上一轮 11 个成员中**只有 `throughLocal` 文本改变**（其余逐字节相同，已 diff 验证）；`p3_scope` 9、`p3_eval_context` 10、`p3_local_rewrite` 7、`p3_content` 4、`p2_golden` 9、`p3_declaration_handoff` 8 全绿。

## 执行对照

`tests/p3_execution_comparison.rs` 的 `p3-boolean-contexts/v8` 样本：**16 个成员 `Executed`**、1 个拒绝（`negated`），全部 wrapper 按运行自身的事实派生声明并编译，**trace 29 行两侧一致**，基线驱动 28 行逐字断言（含 `localFromCall()=true`、`pick(7,true,true)=true`、`pick(0,false,false)=false`、`fromLocal(true)=true`、`throughLocal(false)=false`、`intLocal(0)=0`）。

## fixture 与语料

`tests/fixtures/p3-boolean-contexts/`：`BooleanContexts.java`（18 个成员 + `<clinit>`）、`Baseline.java`、`README.md`（命令、逐成员字节码含 `static {}`，并与类字节机器比对一致、修正前 javac 拒绝、前后文本、拒绝、基线行、复现），`v8/BooleanContexts.class`（52.0，**1114 字节**，SHA-256 `e6a85f9a…6bfb2bf0d`；用提交的源码重编译得到逐字节相同的类）。reader census：`(46,191,44,93,8)` → **`(46,196,44,97,8)`**；fingerprint 仍 114 个文件（三处内容更新，总字节 627,249 → 629,243）。

## 本 change 内一并修正的陈述

- `crates/jarde-java/src/ast.rs` 的 `Type::Boolean` 文档此前称"帧读取从不产出这四个，只有 lambda 的 SAM 描述符读取会"——现已过时：`build.rs` 也会从 descriptor（`Z` 返回/参数/callee/字段）与已声明 boolean 的局部产出它。已改写（实施者的文件集外，由复核者补）。
- `docs/support-matrix.md:54` 的"参数按描述符定型（`boolean` 参数写 `if (b)`）"已进一步不完整，改为说明四个位置（返回/条件/字段/局部）都按上下文与证据定型、证据只来自 descriptor 与本次已声明的 boolean 局部、无证据时拒绝。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1273 passed / 0 failed / 6 ignored**，复核者连跑两次一致（基线 1264，+9） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（27.2 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `openspec validate --all --strict --no-interactive` | 21 passed / 0 failed（归档前） |

## 边界

- **仍然不能表达**（如实登记）：`boolean c = true; return c;` —— 字面量定型的局部无证据，现在**拒绝**而不是发布 `int c = 1; return c;`（能力收窄，但输出不再非法）；比较结果赋值（`c = (n != 0);`）今天不可渲染；**语句级**拒绝仍只存在于产物（`// @bytecode N` + 理由 + anchor + `report.method`），不进 `report.diagnostics`。
- **未触碰的既有弱点**：反向情形"已证明 boolean 的值存入被定型为 `int` 的变量"未拒绝（javac 不会产生该形态，加它属于未被要求的拒绝）；`emit::expr` 对拼接左折叠的递归深度（属发射器形状工作）；深表达式递归除已测反例外的未知面（`bound-recovery-recursion` 已登记）。
- 本 change 不改变 representation/quality/coverage/execution 的语义：拒绝时按既有 refusal 契约降级为 Mixed/Fallback 并保留 bytecode 与 anchor。
