# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。反例由 benchmark campaign 报告、复核者独立复现；本 change 是当前路线三项中的最后一项。

## 反例与根因

受控 fixture（`javac --release 8 -g:none`）修正前把数组 descriptor 直接写进类型位置：

| 成员 | 修正前 | javac | 修正后（复核者独立复跑） |
| --- | --- | --- | --- |
| `echoed([B)[B` | `[B local1 = copy(arg0);` | `illegal start of expression` at `[B` | `byte[] local1 = copy(arg0);` |
| `named([Ljava/lang/String;)[Ljava/lang/String;` | `[Ljava.lang.String; local1 = arg0;` | 同上 + `not a statement` | `java.lang.String[] local1 = arg0;` |
| `grid([[I)[[I` | `[[I local1 = arg0;` | 同上 | `int[][] local1 = arg0;` |
| `table([[Ljava/lang/String;)[[Ljava/lang/String;` | `[[Ljava.lang.String; local1 = arg0;` | 同上 | `java.lang.String[][] local1 = arg0;` |
| `copy([B)[B`、`text(Ljava/lang/String;)…`（对照） | 正常 | 编译通过 | **逐字节不变** |

`byte[]` 那条修正前报 `Structured`/`ContainsStatements`——与前一族同样的"结构性平面看不到"。根因：`build.rs::spell_reference` 只做对象 descriptor 脱壳与 `/`→`.`，数组原样通过。

## 类型位置枚举（本 change 的核心证据）

| 位置 | 类型来源 | 可带数组 descriptor | 覆盖 |
| --- | --- | --- | --- |
| **局部声明**（`declare`、提升的 `declarations()`/`declare_at`，经 `value_type` → `spell_reference`） | 帧的 `RefType::Named` 名（字段 descriptor，`[` 分支返回整个 descriptor） | **是——缺陷所在** | fixture 精确文本 + 编译执行对照 + `tests/p3_array_types.rs` |
| TWR header（`resource_declaration`） | 同一 `value_type` 事实 | 原则上可以；编译器不会产出（resource 必须是 `AutoCloseable`） | 同一入口；拼不出时拒绝该 header 并带自己的 BCI |
| 构造的类（`new_expr`、lambda 的 `Reach::Constructor`） | 池里 `<init>`/handle owner 的 `CONSTANT_Class` 名 | **手工字节可达（实测 `new int[]()`）**，不可拼时拒绝（`unnamedNew`） | `tests/p3_array_types.rs` |
| 静态成员 owner（字段读/写、lambda 限定符） | `Fieldref`/handle owner 的池类名 | **手工字节可达（实测 `int[].value`）**，不可拼时拒绝（`unnamedOwner`） | 同上 |
| lambda 参数（`lambda::parse_type` → `LambdaParam.ty`） | 实现 handle 的方法 descriptor | 是，且**原本就正确** | `lambda.rs` 既有测试 + 新增一致性测试 |
| `facts.rs::parameter_types` 的 `[` 分支 → `Object` | boolean 使用事实，从不写成类型 | 否，不是位置 | 按要求未触碰 |

另实测：`checkcast` 到数组类型**不会**走到拼写——cast 规则先拒绝该形态（`String[] local = (String[]) v;` → Mixed/Fallback，quote BCI 1/4，无声明）。

## 实现

`spell_reference` 改为可失败且数组感知，**数组分支直接复用 `lambda::parse_type`**（后者改为 `pub(crate)`，成为 crate 内唯一的 descriptor→Java 拼写），`build.rs` 只补失败规则：

```rust
b'[' => match crate::lambda::parse_type(descriptor.as_bytes(), 0) {
    Some((Type::Reference(spelled), end)) if end == descriptor.len() => Some(spelled),
    _ => None,
},
b'(' => None,
b'L' => /* L…; 脱壳，空名或内含 ';' 则 None */,
_ => Some(descriptor.replace('/', ".")),
```

`lambda.rs` 另加一处共享契约需要的守卫（`L;` → `None`，而不是 `Reference("")`）。新增一致性测试把两处拼写钉在一起（`[B`/…/`[[[[Ljava/lang/Object;` 双方一致；`[`、`[V`、`[Lfoo`、`[L;`、`[[`、`[[L;` 双方拒绝）。拒绝经既有词汇与 BCI 位置传递，未新增诊断码。

## 拒绝范围（保持窄）

只对**说不出 Java 类型**的输入拒绝：空串、`(` 方法 descriptor、`L;`、内含 `;` 的 `L…;`，以及元素不是完整字段类型的数组（`[`、`[V`、`[Lfoo`、`[[`）。裸名字从不被重新解释为 descriptor（类可以就叫 `Lfoo`），`spell_reference` **没有**变成 descriptor 校验器，`facts.rs` 的 `Object` 决策未动。

## 变异与对照

- **变异**（数组分支退回原样通过）：9 个用例中 7 个变红（四个精确文本 + 泄漏检查 + 边界 `return [I.value;` + 拒绝用例退回 `Java`），编译对照在 `echoed` 处 javac 报 `illegal start of expression`。恢复后 `build.rs` 与变异前逐字节一致（`a8d57f11…`），无 `println!`/`dbg!`。
- **对照**：对象/基本类型拼写逐字节不变（`text`、`copy`）；`lambda.rs` 既有数组拼写测试全绿（`jarde-java --lib` 67 passed，含 2 个新增）；`p3_scope` 9、`p3_eval_context` 10、`p3_content` 4、`p3_local_rewrite` 7 全绿。

## 编译执行对照

`p3-array-types/v8` 六个成员全部 `Java/Structured`、wrapper `compiles`、`traces identical`（trace 7 行），派生声明为 `public static byte[] copy(byte[] arg0)`、`byte[] echoed(byte[] arg0)`、`java.lang.String[] named(java.lang.String[] arg0)`、`int[][] grid(int[][] arg0)`、`java.lang.String[][] table(java.lang.String[][] arg0)`、`java.lang.String text(java.lang.String arg0)`；提交的 `Baseline.java` 打印 7 行。

## fixture 与语料

`tests/fixtures/p3-array-types/`：`ArrayTypes.java`、`Baseline.java`、`README.md`（命令、版本、尺寸、SHA-256、七个成员的字节码与原始 code 字节、修正前文本与 javac 拒绝、双向文本表、位置表、对照结果），`v8/ArrayTypes.class`（52.0，**511 字节**，SHA-256 `84cea788…a3ac59`，重编译逐字节相同）。census `(46,196,44,97,8)` → **`(47,203,44,97,8)`**；fingerprint **114 → 117 files**。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1285 passed / 0 failed / 6 ignored**，复核者连跑两次一致（基线 1273，+12） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（29.3 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `openspec validate --all --strict --no-interactive` | 20 passed / 0 failed（归档前） |

| 该次 Push 的 CI（实现提交 `66bd2d0` 随归档提交 `8807fa5` 一起推送） | [run 35516767173](https://github.com/LordCasser/jarde/actions/runs/35516767173) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke。run 挂在归档提交上（GitHub 只为 push 的 head 建 run），其树包含 `66bd2d0`。 |

## 偏差与记录在案的边界

1. **计划 2.2 的前提在两个位置上不成立**：`ExprKind::Path` 与 `ExprKind::New` **可以**从手工字节收到数组 descriptor（`getstatic` 的 class 为 `[I` → `return int[].value;`；`new [#]/<init>` → `return new int[]();`），两者都报 `Java/Structured` 而 javac 拒绝（`class expected`、`array dimension missing`）。**这是位置的既有边界，不是本 change 引入**：修正前这两个位置的文本（`[I.value`、`new [I()`）同样非法；JVMS 4.4.2 下没有合法 class 文件能表达它（数组不声明字段、也不声明 `<init>`）。按计划保持"这些位置照旧拼名字、不新增拒绝规则"，并把边界**永久记录**在 `tests/p3_array_types.rs`（带"记录边界、非验收"注释）与 fixture README 的位置表里。若复核者要更强的规则（"数组类型不得占据这些位置 → 拒绝"），那是一条小且独立的后续，本 change 未取。
2. **对照 harness 的假设少了一条断言**：`java_type` 能正确读数组，但 `run_sample` 会把每个**基本类型元素**的参数的拼写与 `MethodFacts::parameter_types` 比较，而后者的 `[` 分支刻意是保守的 `Object`（boolean 使用事实，不是名字）——于是 `[B` 会用 `byte[]` 对 `Object`。已在这条断言上做最小修正（跳过数组参数）并写明原因；wrapper 自身的数组拼写由派生声明见证与 `tests/p3_array_types.rs` 覆盖。
3. `spell_reference` 的裸名字分支仍会打印任何非 descriptor 字符串（如池名 `foo;bar`）——这是"不变成校验器"的代价，已知即可。
4. 未触碰、已注意：超过 255 维的数组会被拼出而不是拒绝（javac 拒绝该文本；JVMS 4.3.2 限 255 维，只有手工字节能到达）；lambda 不可重放捕获的类型缺失沿用 `lambda::plan` 自己的措辞，而不是拼写专用措辞。
