# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。本 change 用表示修正同时关闭 T1（深拼接 abort）与 T4（转换算错），两者**分别验收**。

## 表示

`crates/jarde-java/src/ast.rs` 新增专用片段形式：

```rust
ExprKind::Concat { parts: Vec<ConcatPart> }        // 每个 append 一个片段，按调用顺序
pub struct ConcatPart { pub parameter: Type, pub value: Expr }
```

- **片段携带**：值自身的表达式（`render_value` 的结果——其 `OriginSet` 把值的生产者作为 *direct* anchor，并经 `Expr::derived_from(append_bci)` 把执行转换的那个 `append` 作为 *presented* 证据；一个片段 ⇔ 一个 `append` BCI），以及 **append 的参数类型**（AST `Type`）。`concat::Chain::appends` 改为携带解析后的 `Type`，`record_of` 用 `Type::spell()` 拼写，因此 `report.concats[].appends[].parameter` 与之前逐字节相同。
- **旧折叠被移除**：`build.rs::concat_expr` 现在推入片段而不是折成 `Binary`；树、打印器、`Clone` 与 `Drop` 全部迭代 `Vec`。节点自身 origin 未变（primary 为 `Origin::direct(chain.tail)`，其余 owned BCI 为 derived）。
- **打印器**：`Concat` 臂按序迭代，每段用 `binary_operand(.., Add, Left/Right)` 写出（绑定与 `+` 同级的片段在右侧保留分组），并在 `parts[0].is_a_string()` 为假时先写 `""`。`expression_binding(Concat) = binary_binding(Add)`，因此拼接出现在 receiver 位仍保持 `(a + b).substring(1)`。
- **转换**：构建层对参数为 `Z` 的片段套用 `boolean_spelling`（与实参/返回/存储路径同一实现），并对**无 boolean 证据**的 `Z` 片段拒绝；其余被接受的 overload 不需要额外拼写，因为 `"" + value` 就是 `String.valueOf`。

## T4 逐形状（我独立复跑）

| 成员 | 修正前 | 现在 |
| --- | --- | --- |
| `twoIntsThenString(II)` | `return arg0 + arg1 + "!";`（`(1,2)` → `"3!"`，类为 `"12!"`） | `return "" + arg0 + arg1 + "!";` |
| `onePartIsASum(II)` | 同上文本 | `return "" + (arg0 + arg1) + "!";` |
| `booleanLiteral()` | `return 1 + "!";`（`"1!"` vs 类 `"true!"`） | `return "" + true + "!";` |
| `booleanParameter(Z)`、`nullPart()`、`objectPart(Object)` | — | `"" + …`（覆盖） |
| `marked()`、`failing(I)` | `markA() + markB() + "!"` | `"" + markA() + markB() + "!"` |
| `numericLast`、`allStrings`（对照） | — | **不变** |

`twoIntsThenString` 与 `onePartIsASum` 修正前是**同一段文本却是两个不同的程序**——现在分别是 `"" + a + b + "!"` 与 `"" + (a + b) + "!"`，即"不得平衡/合并片段"的可证伪对。

**执行对照**：新样本 `13 requested = 13 produced`，全部成员 `executed: traces identical`，**25 行 trace 两侧一致**，`Baseline.java` 打印类自身的答案（`twoIntsThenString(1, 2)=12!`、`(7,0)=70!`、`(0,-1)=0-1!`、`booleanLiteral()=true!`、`objectPart(null)=null!`）。

**副作用与异常顺序**（执行、两侧一致）：`marked()=12! calls 0->2`（两段各求值一次、按字节码顺序——交换得 `"21!"`、合并得 `"3!"`、重复则计数不同）；`failing(0)` 抛出 `ArithmeticException: / by zero` 且 `calls 3->4`（抛异常之前的那段**确实可观察地跑了**，之后的段没有跑）；`failing(2)=150! calls 2->3`。

## T1 证据

- **生成器**：`deep_concat_fixture(appends)`，两处逐字节相同的副本（`tests/p3_concat_conversion.rs`、`crates/jarde-cli/tests/task_cli.rs`）：类 `p/DeepConcat`、`public static method(Ljava/lang/String;)Ljava/lang/String;`、直线 `new StringBuilder; dup; invokespecial <init>; N × (aload_0; invokevirtual append(Ljava/lang/String;)); invokevirtual toString; areturn`、version 52、无 `StackMapTable`（无分支）、固定 21 项池。`Code = 11 + 4N`，类 = `312 + 11 + 4N`；钉住的 `N = 2048` → **8515 字节，SHA-256 `00ef5b95…622450e`**，测试内断言该 digest；确定性由两次调用一致与"Python 生成器产出字节逐字节相同"证明。**用生成字节而非编译**：链长是检验的参数，而 javac 需要 `-J-Xss64m` 才能编译这些输入。
- **修正前 abort（记录一次）**：debug CLI、默认栈、同一批字节——`N=1024` exit 0（2,214,580 字节报告）；`N=1536`/`2048` **exit 134**（`Abort trap: 6`），stdout **0 字节**，stderr `thread 'main' has overflowed its stack` / `fatal runtime error: stack overflow, aborting`；放大预算后相同（与预算无关）。`lldb` 停在 `Emitter::operand`（`emit.rs:505`），每层重复组为 `Emitter::expr::{closure#0}`（**emit.rs:476**）→ `binary_operand`（`emit.rs:544`）→ `operand`（`emit.rs:511`）→ `expr`（`emit.rs:397`）——即打印器对折叠树的走查，不是 `Drop`。`RUST_MIN_STACK` 在本环境对该二进制无效（实测 `RUST_MIN_STACK=16384` 时 `N=1024` 仍完成）。
- **修正后（我独立复跑）**：用 javac 构建的**合法**类（`javac -J-Xss64m --release 8 -g:none`）——`S1536` → **exit 0**，3,130,892 字节报告，`contains_statements`，1535 个 `+`；`S2048` → **exit 0**，4,172,812 字节，`contains_statements`，2047 个片段；stderr 空。修正前这两个输入是 exit 134。
- **子进程级检查**（debug、默认门禁、无 `RUST_MIN_STACK`、无大栈线程）：默认预算 → exit 0、JSON 报告、`content=contains_statements`、每 append 一个片段、无信号；`--budget output_bytes=17000` → exit 4、JSON 报告、`content=not_produced`、`text=""`、`source_map.segments=[]`、`execution.status=partial`、`outcome.stopped`——无信号，也没有"空 stdout"。
- **release 门禁（可逐字重跑）**：`cargo build --release -p jarde-cli --locked` + `cargo test -p jarde-cli --test task_cli --locked -- --ignored the_deep_chain_answers_in_the_optimized_build` → **1 passed**（对 `target/release/jarde-cli` 施加同一批断言；我本次也复跑过）。

## 变异与对照

- **变异 A（恢复左深折叠）**：debug CLI 于 `N=2048` **恢复 abort**（exit 134、stdout 0 字节、栈溢出），子进程测试 FAILED（"exited with a status rather than a signal"，SIGABRT），库侧深链测试被 `signal: 6` 杀死，T4 的 5 个断言红（`arg0 + arg1 + "!"` vs `"" + …`）。一个变异同时让两项要求变红。
- **变异 B（去掉字符串上下文）**：T4 文本全红（`arg0 + arg1 + "!"`、`true + "!"`、`null + "!"`）、打印器单测红，执行对照以原反例失败（`original "12!"` vs `generated "3!"`）。
- **变异 C（额外，拒绝的证伪）**：关掉 boolean 证据检查后，本应是 `append(boolean)` 的输入被发布为 `return "" + 5 + "!";`（`Java/Structured`），拒绝用例红。
- 恢复：`build.rs`/`ast.rs`/`concat.rs` 与变异前 SHA-256 一致，`emit.rs` 的差异仅为变异之后新增的打印器测试；`grep` 无 `MUTATION`/`string_context = false`/`if false &&` 残留。
- **对照**：`allStrings`、`numericLast` 文本逐字节不变；内存样本 `"x" + 5 + f()` 不变（`p3_patterns` 42 passed，含四条 concat 拒绝与接受集单元测试）；`p3_eval_context` 10、`p3_content` 4、`p3_local_rewrite` 7、`p3_scope` 9、`p3_boolean_contexts` 13、`p3_hoisted_boolean` 9、`p3_array_types` 9 全绿。

## fixture 与语料

`tests/fixtures/p3-concat-conversion/`：`ConcatConversion.java`（13 个成员）、`v8/ConcatConversion.class`（52.0，**1461 字节**，SHA-256 `0e999005…803b932`）、`Baseline.java`、`README.md`（命令、尺寸、digest、修正前文本与值、**每个成员**的原始 code 字节与 javap 列表——两者程序化比对过、基线行、复现步骤）。fingerprint **123 → 126 files**；reader census `(49,222,44,125,8)` → **`(50,236,44,125,8)`**。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1310 passed / 0 failed / 7 ignored**，复核者连跑两次一致（基线 1298/0/6；+12 passed、+1 ignored 为 release 门禁） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（40.3 s） |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `cargo test -p jarde-cli --test task_cli --locked -- --ignored the_deep_chain_answers_in_the_optimized_build` | 1 passed（release） |
| `cargo test --test p3_concat_conversion --locked` | 10 passed |
| `openspec validate --all --strict --no-interactive` | 21 passed / 0 failed（归档前） |

## 实现者自定取舍（已核对）

- **空串写在链节点上而不是伪片段**：片段与 append 保持 1:1，节点自身 anchor（primary = 被执行到的 `toString`）承载它，因此没有 anchor 声称为未执行的指令。
- **统一的字符串上下文规则**（首个片段参数不是 `java.lang.String` 就插 `""`，含 "参数是 `Object` 但值已证明是 `String`" 的情形，其 `""` 是恒等）：后果是两个既有内存 fixture（`p3_patterns` 的 `concat_buffer_class`、`accessor_class` 的 `combined`）得到 `"" +`，对应断言已按"有意变更"更新，值未变。
- `Chain::appends` 携带解析后的 `Type`，使接受集判定留在原处，发布的 `parameter` 字符串不变。
- 额外给了两个证伪（变异 C 与"合并/交换会改变结果"的执行对照）。

| 该次 Push 的 CI（实现提交 `5c35cbf` 随归档提交 `c2aa793` 一起推送） | [run 35524042352](https://github.com/LordCasser/jarde/actions/runs/35524042352) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke。run 挂在归档提交上（GitHub 只为 push 的 head 建 run），其树包含 `5c35cbf`。 |

## 边界

- **CLI 无法报告"发射中途的 `output_bytes` 停止"**：同一上限同时资助请求工作与渲染出的文档。实测 `N=2048` 时 19,000–31,000 落在发射中（`consumed == limit`），适配器随后以 `budget_exceeded` 拒绝（exit 2、stderr、无 stdout）；17,000 则在足够早处停止、报告放得下（exit 4）。因此**发射中途的清理**（2048 段全部构建后再释放）断言在**库入口**（`output_bytes = 12_288` → `Stopped(Budget { OutputBytes, written 3770, at BCI 2043 })`、`content=not_produced`、空 text/segment、两次运行一致），CLI 侧覆盖的是"可交付的停止"形状。
- `RUST_MIN_STACK` 在本环境对该二进制无效，因此计划里排除的"放大栈"对照两种方向都做不出来；验收在默认栈上进行。
- 既有弱点、未触碰：`source_map.rs::OriginSet::mentions` 会把 *derived* primary 报成 `Direct`（该文件不在允许改动范围；本表示不产生这样的 anchor）；CLI 的 `output_bytes` 双重职责；默认输出上限会限制**很长链的文档**大小（`N ≥ 8192`，属文档成本而非栈成本，本 change 未改）。
