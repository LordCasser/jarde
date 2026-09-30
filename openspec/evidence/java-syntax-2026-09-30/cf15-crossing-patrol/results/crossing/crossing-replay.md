# `recover-named-row-crossing-locals` 实施复放记录（2026-09-30，`-crossing` 专名）

基线 = 主线 `66db80aa` 的恢复代码（本目录 `*.base.txt` / `*.base.json` 即该二进制的输出）；实施 = 本切片的门槛判别 + 写值扩展。所有恢复文本来自 `jarde-cli class-source --policy single-class --format text`，诊断来自同命令 `--format json`，运行对照为 `javac --release 8` 重编后 `java -Xverify:all`。fixture SHA 见 [crossing-fixture-sha256.txt](crossing-fixture-sha256.txt)，原四类 fixture SHA 复核与 [results/fixture-sha256.txt](../results/fixture-sha256.txt) 冻结值逐条一致。

## 1.1 基线复放

| 场景 | 基线行为（`*.base.txt`） | 与巡查 README 一致性 |
| --- | --- | --- |
| C1.five | 整方法回退：`local 0 crosses a quoted fallback region`，诊断 `jre_guard_handler`（BCI 18 handler 序列）+ `jre_region_uncovered_blocks` + `jre_concat_split` | 一致 |
| C4.constructNamed | 整方法回退（同 C1.five 家族，`jre_guard_handler`） | 一致 |
| C4.twrNamed | 整方法回退（`jre_guard_body`，CF-17 域） | 一致 |
| C3.five / C3.popstmt | 均完整恢复（popstmt 仅 concat 既有 interleaved 提示） | 一致 |
| N1 / P3StorePrefix（store 前置钉死负例） | `jre_guard_handler` + `jre_region_uncovered_blocks` 整方法回退 | 一致 |

## 1.2 六个 verifier 有效负例（`fixture/crossing/`，全部 `java -Xverify:all` 通过；X2–X5 为 javac 不写的字节码/异常表形状，ASM 手工构造）

| 负例 | 形状 | 基线（修复前） | 实施（修复后） |
| --- | --- | --- | --- |
| `X1Catchall` | **catch-all 行**（无类型）+ 完成 store 前置 | `jre_guard_handler` 回退 | 逐字节不变（`SAME`，`jre_guard_handler` 保持）——catch-all 降级一字未动 |
| `X2Split` | 具名行起点在构造 `dup` 处，**劈开构造表达式**（起点栈深非零，无语句边界） | 资源检查拒绝 | 逐字节不变（`jre_guard_resource_init`） |
| `X3Between` | 构造与 store 之间插入语句（`ldc; pop`）——store 的运行不可读 | 资源检查拒绝 | 逐字节不变（`jre_guard_resource_init`） |
| `X4DoubleUse` | 构造结果经第二个 `dup` 被 **两个 store 消费** | 资源检查拒绝 | 逐字节不变（`jre_guard_resource_init`） |
| `X5CrossBlock` | 分配在入口块、store 在合并块——**跨块构造** | 资源检查拒绝 | 逐字节不变（`jre_guard_resource_init`） |
| `X6Direct` | 构造被调用**直接消费，无 store** | 普通 try/catch 呈现（无跨区声明可发明） | 逐字节不变，构造内联在消费点 |

前后对照：`X*.before.txt`（基线二进制）/ `X*.after.txt`（实施二进制）逐字节一致，六条 SHA 对见上表同名列；负例的拒绝语义在实施前后均成立（该拒的拒）。

## 2.1/2.2 实施后的正例恢复

- `C1.five`：完整恢复——`StringBuilder local0` 声明 + `local0 = new java.lang.StringBuilder();` 引导语句 + 五个连续 `try { local0.append('x'); } catch (…)` + `return local0.toString();`（[C1.after.txt](C1.after.txt)）。
- `C4.constructNamed`：完整恢复，同上形态，catch 体 `return "caught"`（[C4.after.txt](C4.after.txt)）。
- N1 / P3StorePrefix（翻转）：`local1 = open();` 引导语句 + 唯一 `try/catch`，跨区局部在受保护体与 handler 中按源读取（[N1.after.txt](N1.after.txt) / [P3StorePrefix.after.txt](P3StorePrefix.after.txt)）。

## 3.1 翻转与边界保持

- N1/P3StorePrefix 由"store 前置降级"负例翻转为正例；`p3_preceded_catches` 的 `a_store_prefix_still_degrades_to_the_resource_refusal` 由 `a_store_prefix_before_a_named_catch_presents_the_catch` 取代（期望同步，翻转理由见 [preceded 验证记录第五节](../../../../changes/recover-preceded-statement-catches/verification/post-merge-regressions.md)）。
- 其余负边界逐项保持：catch-all 降级（X1，`jre_guard_handler` 一字不动）、行劈开语句（X2）、吞初始化、交叠呈现（全量套件既有钉死）；guard verdict 级新单测 `a_named_row_a_completed_store_precedes_is_not_examined_as_a_resource_header` / `the_crossing_answer_keeps_every_unreadable_or_unowned_store_refused` / `a_twr_lowering_row_a_completed_store_precedes_keeps_its_examination` 钉死判别的正反两侧。

## 3.2 回归

| 项 | 结果 |
| --- | --- |
| C3 对照组 | 逐字节不变（`C3.base.txt` vs after，`diff` 为空） |
| C4.twrNamed | 方法级与基线逐字节一致（保持拒绝，CF-17 另案） |
| FinallyOnce 全类 | 与冻结期望 `fo.exp.java` 逐字节一致（[FinallyOnce-crossing.java](FinallyOnce-crossing.java)，SHA `e3b2eb7fbffa442cab7f7cc4b781773c0bfbdbcef13278275a98efdfffc43d7c`） |
| Test2 全类 | SHA `59d5c8ca3f94714cd94e164b4200da220b2083a247ae099c7882324f09eeb67b` 与冻结值一致（[Test2-crossing.java](Test2-crossing.java)） |
| C2.alias | 与基线逐字节一致（Slice B 域不受影响，[C2.after.txt](C2.after.txt)） |
| Tf1–Tf4、dt14、M1/M2、P5TwoResources | 全量套件内钉死测试全绿（guard 单测 + `p3_preceded_catches` M1/M2 verbatim 常量） |

## 4.1 三方对照（`results/crossing/three-way/`；固定 JADX dev，SHA 逐路径一致）

原 class / 固定 JADX / Jarde class-source 三条腿各自 `javac --release 8` 重编后 `java -Xverify:all` 运行；注入异常路径经由受保护体内的条件抛出 helper（`C1x`/`C4x`/`N1x`，源码随目录），冻结 C1/C4/N1 走正常路径。`*.jarde.java` 为 Jarde 腿源码，`*.jadx.java` 为 JADX 腿源码，`*.out` 为各腿各路径 stdout。

| 类·路径 | 原 class 腿 | JADX 腿 | Jarde 腿 | stdout 内容 |
| --- | --- | --- | --- | --- |
| C1·normal | `a0a8bae2fc7a19d92e2f1c33f3e515cbef53fa010f63240c4853f9ad4113b0d7` | 同 | 同 | `ok` / `abcde` |
| C1x·normal | 同上 SHA | 同 | 同 | `ok` / `abcde` |
| C1x·boom | `bd4bbeeb1df5cb65e648998bbc113fc8fa9593d5055641fd895f76c6cda158fa` | 同 | 同 | `missed` / `bd` |
| C4x·normal | `87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7` | 同 | 同 | `a` |
| C4x·boom | `b0d97684b0dc50256a1e920238fa474bf9ff19c4c282f74e97e1f715d6d97065` | 同 | 同 | `caught` |
| N1·normal（翻转后正例） | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | 同 | 同 | `0` |
| N1x·normal | `7518621be3a5221ab9a492dd551de023d1774fd1ce5fb3cdee65413f4f368971` | 同 | 同 | `end:1` |
| N1x·boom | `f81a703667ed898236b036158c7766a1db08b8fc9b77a2d45d963d6d4efd4a20` | 同 | 同 | `0` / `end:0` |

全部腿运行无 `VerifyError`/异常栈（stderr 全净）。`C4x` 只含 `constructNamed` 形状——`twrNamed` 属 CF-17，实施二进制对其保持拒绝，全类三方对 C4 冻结类不成立（Jarde 腿文本含引述，不可编译），故 `constructNamed` 以独立伴随类对照。

## 4.2 门禁

- `cargo test --workspace --tests --locked --no-fail-fast`：**全绿**（263 个测试二进制 `test result: ok`、0 失败；一次 `frozen_identity_shapes_and_raw_control_project_together` 单发失败，三轮复跑与最终两轮全量均未复现，按任务书 flake 口径判定）。
- `cargo fmt --all -- --check`：通过。
- CI 同款 clippy（`ci.yml` 全量 `-A` 清单 + `-D warnings`）：零告警（新码零新增；本地 1.98 存量 23 站点未动）。
- `openspec validate --all --strict`：通过（223 项）。

## 判别与写值判据（落点摘要）

- `guard.rs::resources`：`before` 是 `Operation::Store` 且行具名（`catch_type_index.is_some()`）且 store 的运行 [`single_statement`] 可读且行尾无该槽 [`normal_close`]、handler 无 [`closes_something`] 且行起点 [`statement_boundary`] → `continue` 交回 catches。三个附加判据是固定负例与 TWR 零回退逼出的：javac 的 TWR 主行可携带 `Throwable` 类型且紧跟完成 store（P5TwoResources `[8,51)`），字面三条件会把它交给子句呈现；X3–X5 的 store 运行不可读，字面条件会把"插语句/双用途/跨块"变成错误程序。
- `build.rs::all_reads_reach_presented_writes`：写值接受集在 `presented_int_store_value` 之外新增 [`presented_reference_store_value`]——同块、`Value::Ref`、受保护 try 臂、[`single_use_at_with_budget`] 单用途、紧随 store；`<init>` 形状要求 [`Sites::site_producing`] 的已证站点表达式恰好覆盖 `[site.head, store)` 整段运行，零参静态调用要求单指令运行。类型闸 `cross_exception_store_type_is_proven` 放行已定 Reference 类型。声明经既有 Elevated 通道提升到语句词法父域，引导 store 以既有 new-site/调用表达式呈现。
