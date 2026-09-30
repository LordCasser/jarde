# CF-16 固定 Test2 验收（recover-void-loop-finally）

基线为 `b3a01e24c86755dfe9a356f4ffc72636194eda70`。固定输入是 `openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/fixed/TestTryCatchFinally2$TestCls.class`，SHA-256 `330ddd26a3bb313fde9be032e78c1be83e7b6acb2c1b86e53abc1ce604b89170`（major 55）。本目录的最终 Jarde 完整 class-source 是 [TestTryCatchFinally2$TestCls.java](TestTryCatchFinally2$TestCls.java)，SHA-256 `59d5c8ca3f94714cd94e164b4200da220b2083a247ae099c7882324f09eeb67b`（[摘要](recovery-sha256.txt)）。CLI 的报告为 `structured`、`fallbacks = []`（[摘要](recover-summary.txt)、[完整 JSON](recover-report.json)）。

## 基线重放（任务 1.1）

在基线提交上完整运行 `openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/replay.sh`（本机 JADX pin `2fb1b16386941660fda07e9017285aec40fcb37f`、JDK 23.0.1），退出码 0：上游 `TEST_INPUT_PLUGIN=dx` 测试通过、`check-bytecode-shape.py` 记录三份 Java 8 方法仅在 BCI 55 的 private `writeString` 调用上不同（`invokevirtual` vs `invokespecial`）、fixed/original/jadx 三方 `java -Xverify:all` 行为逐字节一致、专用 Cargo target 由脚本 trap 清理。基线 Jarde 对固定类安全拒绝：`jre_guard_handler`（BCI 160，rule `twr@1`）加 `jre_region_uncovered_blocks`（`[35, 160, 42, 64, 76, 83, 153, 115, 122, 147]`），与证据目录提交的 `results/jarde-test-evidence.json` 一致；证据目录按其冻结约定保持不变。

## Guard 证明（任务 2.1/2.2）

`crates/jarde-java/src/guard.rs` 新增 `prove_void_loop_finally`，注册在 `guarded()` 的 FINALLY 探测中，并给 `FinallyCompletion` 增加独立 `Void { binding_row, final_return }` 变体。有界物理证明逐项读取本 run 的事实：

- 恰好两行 catch-all、表序相邻：`[9,153)→160`、`[160,162)→160`；
- 89 个指令起点与 opcode 全表匹配，且每条指令的行覆盖为 `body 行 / 绑定行 / 空` 三者之一——由此绑定行只盖 `astore 12`（`span_end(160) == 162`），两份 `close()`（153/154 与 162/163）都在一切保护区间之外，自保护行不含 `close()`；
- canonical 边全集恰好等于 18 条预期边：三处正文回边 `42→35`、`122→115`、`147→76`，每个被覆盖块到唯一 handler 的异常边，以及 handler 自环 `160→160`；多出的任何出口、第二 handler 或子程序都拒绝；
- 三个头（35/76/115）在 normal-flow 视图下各两个后继、两个前驱、体内体都在，是普通入口可达循环，不是 Test11 的 handler-only SCC；
- `close` 目标两边同符号（`java/io/DataOutputStream.close:()V`，invokevirtual），接收者按 SSA 由 BCI 8 的同一定义到达两份清理，调用操作数就是各自 load 的栈值；
- handler 的 `astore 12 → aload 12 → athrow` 保存、重载、重抛同一异常值，且 `handler_binding` 证明该值来自行自身的异常交付；
- 完成是 void：normal 清理块的最后一条指令是无栈值的 `return`（169），视图下无后继；plan 不携带、也不伪造任何返回值。

旧证书未放宽：`SavedReturn`/`Joined`/`SharedFinally`/`NullableResource` 等全部匹配分支保持原判定；新证明是按 89 个 BCI 全表锁定的封闭证书。

## Region 与 Builder（任务 3.1/3.2）

`region.rs` 把 Void 计划路由进 `bounded_shared_finally_body`（与 Test5 同一条有界正文 walk）：期望块集为受保护正文 `[9,153)` 所在的 9 个物理块，walk 尾随至 `span.1`（153，normal 清理块起点）后必须无处可去；`shared_join_body_supported` 在同一形状门内允许循环体内再嵌循环（三个循环的正确嵌套），`visited` 回滚在失败时整体生效。`header_tested_loop` 的 finally 行所有权传播原本只对 3/4 行表开放（Test5/Test4 证书）；这里按 `void_loop_finally` 标记（仅由 Void 声明置位）对两行表开放同一传播，使循环体内的异常边按证书行记账——其他两行形状（Test16/Test9）不受影响。

`build.rs` 在既有 `Shape::Finally` 分支中：Void 完成不发明 saved return/catch pop；`nested_complete` 对 Void 直接成立（无常名 catch 需要嵌套呈现）；声明规划为 Void Guard lead 内的那一次资源存储提供窄作用域 hoist（与 `nullable_resource_lead` 同形），跨保护段的 local 2 由此声明一次、两份清理读同一名字。Builder 原子写出一个 `try/finally`、一次 `out.close()`，方法以 normal 清理后的 void 完成收尾（最后的 `return` 是被声明拥有的物理指令，不出语句）。

另有一处呈现类型修复支撑可编译输出：`array_of_value` 现在沿 `aaload`（`ArrayElementLoad`）的数组操作数推导元素类型（与 enhanced-for 的 `array_element` 同一读取），循环变量按 `ClspClass`/`ArgType` 声明而非 `Object`。全仓 484 个既有测试与 golden 输出不回退。

## 有效近邻（任务 1.2）

[neighbors/generate.py](neighbors/generate.py) 冻结六个 verifier 有效近邻，覆盖证书各边界：清理接收者（different-receiver，两份清理关闭第二引用）、清理目标（different-target，handler 侧重指 `java/io/OutputStream.close` 的追加 Methodref）、资源定义重写（resource-definition-rewritten，体内二次赋值）、自保护扩围（self-row-expanded，`[160,162)→[160,166)` 盖住 handler close）、Throwable 重抛改写（throwable-rewritten，`aload 12`→`aconst_null`）、循环额外出口（loop-extra-exit，内层循环 `break`）。`sh neighbors/verify.sh` 逐个以 `java -Xverify:all` 强制链接通过（[verify.txt](neighbors/verify.txt) 含各自 SHA-256）；基线与当前 Jarde 的 `recover` 输出逐个存于 `neighbors/baseline/`、`neighbors/recovery/`（`sh neighbors/run-recovery.sh <baseline-cli> <current-cli>` 可重放），六项两侧都保留 `@bytecode`、都不出现 `finally`，诊断与基线相同（`jre_guard_handler` + `jre_region_uncovered_blocks`）。

## 四方行为对照（任务 4.1）

`sh run-behavior.sh` 以 `javac --release 8`（fixed class 原样使用）编译固定类、独立转写、冻结 JADX Java-input 和本目录的 Jarde class-source，各自与支持类、探针 Runner 同目录编译后以 `java -Xverify:all` 跑全部六条路径（零类、两类三父、首写失败、父类型写失败、close 失败、写与 close 同时失败）。四份 [run 输出](fixed.run.txt)逐字节一致，verifier stderr 全空（[fixed](fixed.verify.stderr)/[original](original.verify.stderr)/[jadx](jadx.verify.stderr)/[jarde](jarde.verify.stderr)），并等于证据目录基线重放的 `fixed.behavior.txt`；输入与输出 SHA 记于 [behavior-sha256.txt](behavior-sha256.txt)。各路径 `closes=1`、close 失败覆盖待传播异常（`write-and-close-failure` 报 `IOException:close`），自保护行未造成重复关闭。

## 门禁（任务 4.2）

| 门禁 | 结果 | 记录 |
| --- | --- | --- |
| `openspec validate --all --strict` | 216 passed, 0 failed；本 change 单项 valid | [openspec-validate.log](openspec-validate.log) |
| `cargo test -p jarde-java --tests --locked` | 484 passed, 0 failed（含本目录新增 4 项回归与 Test3/4/5/11/13/16/17、空 finally 切片） | [jarde-java-tests.log](jarde-java-tests.log) |
| `cargo check --workspace --all-targets --all-features --locked` | 通过 | [workspace-check.log](workspace-check.log) |
| `cargo fmt --all -- --check` | 通过 | [fmt-check.log](fmt-check.log) |
| CI 同款 Clippy（`.github/workflows` 精确允许列表 + `-D warnings`） | 通过，无告警 | [clippy.log](clippy.log) |
| `git diff --check` | 干净 | [diff-check.log](diff-check.log) |

新回归在 `crates/jarde-java/tests/p3_void_loop_finally.rs`：正向证书（一个 `finally`、一次 `close()`、三个循环、89 个物理 BCI 全部有来源、owned 块集恰为 11 块）、六个近邻拒绝、预算与取消时正文与来源映射同时为空。replay 与本次验证的专用 Cargo target 均由脚本 trap 或临时目录清理，仓库内无构建产物。

## 合并后回归与修复（823b4bd1 引入，95470c38 的 CI 失败）

合并入主线后暴露两处根 crate 回归，均为本 change 改动的直接影响面，修复不回退任何恢复改进：

1. **枚举 map 后缀折叠器只识别索引循环呈现。** `array_of_value` 的 `aaload` 扩展让 dt14/CustomInit 的 clinit 从显式索引循环呈现为增强 for（`StmtKind::ForEach`，改进本身保留），而根 crate `src/enum_constants.rs::prove_map_initializer_suffix` 的 AST 形状匹配仍只接受旧的 11 条语句索引循环形状——字节码 27 指令匹配不变地通过，`steps.len() != 11` 起拒绝，证明失败后整类回退逐字段呈现，`custom_map_suffix_recompiles_and_preserves_singleton_identity` 与 `custom_map_suffix_output_stop_and_cancellation_leave_no_partial_projection` 失败（CI 36678484547）。修复：同一 27 指令证书下承认两种呈现——索引循环（11 条语句，原检查原样保留）与增强 for（8 条语句）；增强 for 分支要求节点 origin 折叠被证循环的全部机制指令（ iterable 锚 `aload_0`@suffix[6]、元素读 `aaload`@suffix[16]、单位步进 `iinc`@suffix[24]），循环变量名 `local3` 与被证 astore_3 槽一致，body 恰为被证 put 调用——增强 for 即该索引循环的呈现由 origin 归属钉死，无新机制。
2. **`aaload` 分支的 `(base, rank)` 维数算术对 n≥2 多算一维。** 分支把“元素的类型”（本身可能是数组类型）放进了 base 槽：`int[][]` 的 `aaload` 读出 `int[]` 却被记为 `(int[], 1)`（即 `int[][]`），p3_array_access `cell` 的 `iastore` 组件校验因此拒绝、方法回退。修复：直接返回数组自身的 base 与降一维的 rank（`[[I` 读出 `(int, 1)`）；rank-1 路径（`(base, 0)`，dt14 循环变量声明类型由此改进）逐位不变。

门禁（最终文件状态，本机 JDK 23.0.1 / clippy 1.98）：

| 门禁 | 结果 |
| --- | --- |
| `cargo test -p jarde --lib enum_constants` | 56 passed, 0 failed（54 既有 + 2 回归用例） |
| `cargo test -p jarde-java --tests --locked` | 497 passed, 0 failed（含 p3_void_loop_finally 4 项） |
| `cargo fmt --all -- --check`；CI 同款 Clippy + `-D warnings` | 均通过，零告警 |
| `openspec validate --all --strict` | 218 passed, 0 failed |
| Test2 固定类 class-source | 0 not-recovered，SHA `59d5c8ca…eeb67b` 与冻结值一致，`structured`/`fallbacks = []` |
| dt14/CustomInit（`javac --release 8` 重编 + class-source） | clinit 仍恢复为 `for (dt14.CustomInit local3 : local0)`，常量折叠为 `RED,`/`BLUE;` + 后缀 static{} |
| `cargo test --workspace --tests --locked` | 本 change 相关目标全绿；整仓仍有 **4 个与本 change 无关的既有失败**（见下） |

**遗留阻塞（非本 change 引入，待 preceded-statement-catches 责任方决策）**：`f9da424c`（"answer no for a resources row a non-store statement precedes"，经 a620adc1 合入，二分定位）把“非 store 前缀”的异常行一律跳过资源检查放行给普通 Catches 呈现，对 handler 行覆盖范围与被拒生产链交叠的形状产生了**同一 BCI 的重叠呈现**：`PostfixHandlerBoundary.update`（行 `[7,12)→13`，行起点落在 `a()[I→i()→dup2→iaload…` 后缀链内部）现同时把 BCI 0/3 呈现为语句 `a(); i();`、又把它们列进 try 体内 quote 的 `@bytecode` 清单。全部在 a620adc1 上即可复现、与本 change 的两处修复无关的失败共 8 个：`p3_content::explanation_only_means_no_emitted_statement`、`task_operations::{an_explanation_only_result_and_a_stop_are_presented_in_order, the_presentation_does_not_re_read_the_text}`、`p3_catch_after_field_assignment::a_range_beginning_inside_the_field_assignment_is_not_a_plain_catch`、`p3_guard::a_handler_range_that_swallows_the_initialisation_is_refused`、`p3_postfix_handler_boundary::exception_handler_boundary_inside_postfix_chain_is_refused_with_real_bcis`、`p5_bulk_corpus::{the_fixed_shape_bills_the_counts_the_corpus_pins, the_old_per_method_arms_keep_their_ledger_and_the_same_text}`。修复归属该切片的验收语义（收窄 skip、拒绝该形状或重定义 pin），本次未越权处理。另观察到一次 `bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too` 在 4-worker 投递顺序下的偶发失败（同一最终状态重跑 12/12 通过、干净基线 12/12 通过，断言预算按同一次完整运行的产出减半构造，与本次改动无因果），记录在案。

## 边界

本验收只声明固定 Test2 的 JVM classfile 切片与这六个有效近邻。DX/DEX 输入、其他编译器 lowering（含 `--release 8` 重编 fixed 源的 `invokespecial` 布局）、Test5 双返回、Test9 可空清理、任意循环数与 TWR 自动提取均不在证书内；这些输入继续按原判定拒绝。
