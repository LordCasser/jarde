# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。本 change 把"局部类型由遍历顺序决定"改为"生成语句前一次性决定"，并裁决了上一项遗留的比较边界。

## 反例与根因

受控 fixture（`javac --release 8 -g:none`）修正前：

| 成员 | 修正前文本 | javac |
| --- | --- | --- |
| `copied(ZI)I` | `int local3; boolean local2 = arg0; if (arg1 == 0) { local3 = local2; } else { local3 = arg0; } if (local3 != 0) …` | `boolean cannot be converted to int`（两处） |
| `swapped(ZI)I`（两个分支对调） | **`boolean local3; … if (local3)`** 且编译通过 | — |
| `relayed(Z)Z`（两跳复制） | `int local3; int local2; boolean local1 = arg0; …`，Mixed/Fallback，BCI 18 拒绝；包装后 3 个错误 | 2 × `boolean cannot be converted to int`、1 × `int cannot be converted to boolean` |
| `conflicted(ZI)I` | `int local2; … local2 = arg0; …` | `boolean cannot be converted to int` |

`copied` 与 `swapped` 是同一个程序的两个分支顺序，修正前却得到**不同结论**（`int` vs `boolean`）——这就是"类型取决于遍历顺序"的直接证据。

## 实现

- **决策的载体**：`Declarations` 新增 `decided: BTreeMap<LocalVariable, Decided>`，`Decided = Type(Type) | Unknown(NoType)`，`NoType::{NoFrameEntry, Unspellable(String)}`（两条拒绝消息与 `declare()` 原有措辞逐字相同）。
- **构建位置**：`declarations()`（在 `Builder` 存在之前运行，现为 fallible）先收集每个 `LocalVariable` 的读写（含 region path 与 BCI），再调用 `decide_types(...)`：类型取自该变量**方法序的首次写入**（两条路径今天读的都是它）；种子是 descriptor 为 `Z` 的参数变量与首次写入即 descriptor 已证明 boolean 的变量；边是"首次写入读取了另一个变量"（一次读取、一跳，不做链式追索）；有界 `VecDeque` 求不动点，**每条目按其自身首次写入 BCI 计一次 `IrItems` 并 poll**（用既有维度、不新增；条目数不超过有写入的变量数，各入队一次）。无证据证明 boolean 的变量走 `value_type`，得 `Type` / `Unknown(NoFrameEntry)` / `Unknown(Unspellable)`。**`0`/`1` 从不发起推断**，只在目标已确定为 boolean 处适配。
- **消费点**：提升声明（其 `ty` 即计划的决策）、就地声明（`declare`）、赋值（`write_statement` → `assignment`）、条件（`condition` 的真值测试分支）、返回（`return_expr`）。`Builder::boolean_local` 改为查询该计划，运行期的 `boolean_locals` 集合被删除。
- **`declare()` 的歧义消除**：`declare(variable, at) -> Result<Declaration, StopReason>`，`Declaration::{Declared(Type), NotDue, Refused}`；`Refused` 时 `write_statement` **不再写任何语句**（既不写声明也不写赋值）；每次写入在 `assignment` 里检查：目标已决定为 boolean ⇒ 值必须 `boolean_value`；已决定为其它类型 ⇒ 值不得是 `boolean_proven`（`0`/`1` 两可，仍可拼写）。冲突的 BCI 会出现在理由里。

## 上一项遗留边界的裁决：**拒绝**

`flag() == 1`（一个已证明 boolean 的操作数 对 int 字面量）由本 change 的"类型冲突"规则处置：`condition()` 在渲染任何操作数之前调用 `pair_comparison_refusal`——`Test::Pair` 中**恰好一侧**为 `boolean_proven`（或两侧皆 boolean 却用了排序运算符）在 Java 里无拼写，因此拒绝该区域，理由点名 BCI 与 javac 的判断。证据是实测文本 `if (flag() == 1)` 与 javac 的 `incomparable types: boolean and int`；方向与规则一致（未知或冲突即终止结构），且两侧操作数各自保留拼写（没有任何一侧被改写成 `true`）。**对照必须不动**：`1 == n`、`arg0 == 1`、`0 < n`、`arg0 > 0`、`arg0 != 0`（字面量不是 boolean 证据）——`p3-int-comparisons` 九个成员仍以相同 trace 执行。

上一项归档记录里"处置属于下一个 change"的句子已在本 change 的归档中补记关闭（追加，不改写原结论）。

## 修正后与验收要求逐条对应

| 要求 | 实测 |
| --- | --- |
| **分支顺序交换结论一致** | `copied` 与 `swapped` 的声明行与条件**同为** `boolean local3;` / `if (local3)`，只有两个 arm 的写入顺序互换（子串断言 + 抽取 arm 块比较写入顺序） |
| **复制链** | `relayed` 现在 Java/Structured：`boolean local3; boolean local2; boolean local1 = arg0; …`——第三个变量的证据隔两跳仍然成立 |
| **提升与就地一致** | 两条路径读同一个计划；`swapped` 与 `copied` 的一致即为证据 |
| **纯 int 对照** | `intLocal` 文本逐字节不变；`literalArmed`、`fromParameter` 文本不变 |
| **拒绝路径终止结构** | `conflicted` 的产物**含** `int local2;`、`local2 = 1;`、`// @bytecode 10` 的拒绝理由与本成员自己的 `if (local2 != 0)`；**不含** `local2 = arg0;`、`boolean local2` 或 `= true`；整体 Mixed/Fallback/NotJava。未知声明的一半钉在 `p3_array_types` 的 `elementless`：拒绝声明后不再发布赋值（该期望变更可追溯到本 change 的契约） |

## 变异与对照

- **(a) 提升路径只用 descriptor 证明**：`p3_hoisted_boolean` 5 passed / 3 failed（`copied` 退回 `int local3;`），执行对照在 `copied(ZI)I` 处以 javac 的 `boolean cannot be converted to int` 等 3 个错误失败。
- **(b) 一致性检查只看首次写入**：7 passed / 1 failed（`conflicted` 由 Mixed 退回 Java 并发布 `local2 = arg0;`），执行对照报 "keeps quoted bytecode, and this run states Java"。
- 两次变异后 `build.rs` 均以 `shasum -c` 验证**逐字节复原**（`91cd556e…66ad1`），无 `MUTATION`/`PROBE` 残留。
- **第三个实验钉住计费**：去掉队列的 `charge`+`poll` 后同一 `relayed` 请求计 378 而非 381、隔离上界的首个位置计费从 BCI 1 移到 BCI 9（已复原并校验哈希）——即本 change 引入的传播确实计费。
- **正向对照**：`fromParameter`（descriptor 定型的 boolean 局部）、`intLocal`（纯 int）、`p3_boolean_contexts`、`p3_int_comparisons` 成员、`p3_eval_context`、`p3_local_rewrite`、`p3_scope` 全绿。

## fixture 与语料

`tests/fixtures/p3-hoisted-boolean/`：`HoistedBoolean.java`（8 个成员）、`Baseline.java`（18 行断言）、`README.md`（编译器/命令/尺寸/digest、逐成员字节码与原始 code 字节、修正前文本与拒绝、证据表、两个拒绝产物、对照表），`v8/HoistedBoolean.class`（**743 字节**，SHA-256 `b9184230…d64d1640`，重编译逐字节相同）。reader census：`(48,213,44,105,8)` → **`(49,222,44,125,8)`**；fingerprint **120 → 123 files**。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1298 passed / 0 failed / 6 ignored**，复核者连跑两次一致（基线 1289，+9） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（36.1 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `openspec validate --all --strict --no-interactive` | 21 passed / 0 failed（归档前） |

## 边界与既有弱点

- **逐写入检查只覆盖 boolean 区分**，不是通用可赋值性：后续写入一个首次写入类型装不下的引用（`String local` 被 `Object` 值填充）仍如实发布——这需要本层刻意没有的子类型关系。`long`→`int` 一类不可达（verifier 的 slot 类别决定）。
- **字面量臂局部**（`boolean x; if (b) { x = true; } …`）仍是**记录在案的类型忠实度边界**：两条路径现在结论一致（都按 `int` 处理），但并非该输入的类型真相；编译与执行仍经过验证。
- **遍历顺序说明**：就地路径现在读"方法序的首次写入"（计划自身的规则），而不是 builder 首先到达的写入；builder 按 region path 顺序、块按 BCI 顺序遍历，语料中除预期变化外没有文本因此移动。
- `unproven` 一类仍按既有方式在 BCI 12 拒绝；比较结果作为 `Test::Pair` 操作数由渲染器拒绝，因此不存在可被渲染的混合 pair。
