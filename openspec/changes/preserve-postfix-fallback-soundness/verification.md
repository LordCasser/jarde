# 实现与验证记录（2026-10-05，coder）

本文件记录 `preserve-postfix-fallback-soundness` 的实现落点、锚实测与门禁结果；判定依据全部是实测命令，
无预测性陈述。插桩结论以 [instrumentation.md](instrumentation.md) 为准（Q-i/Q-ii 未改），预审计的 void 追加裁决见
[preaudit.md](preaudit.md)。

## 1. 代码落点

| 文件 | 位置 | 内容 |
|---|---|---|
| `crates/jarde-java/src/build.rs` | `VALUE_LEVEL_REFUSALS` + `fn value_level_refusal`（`completes_normally` 之后，~7780） | 六族诊断的**逐字**片段表；片段取自六处产生点（`render_value` Duplicate/Load、`prepare_deferred_bindings` 依赖链/多消费者、`lambda.rs` 绑定接收者、实参转换证据） |
| 同上 | 方法级守卫（`quoted_control_flow_exit` 升级块之后，~7576） | 只读**本方法体顶层**语句里首个带族理由的 `StmtKind::Fallback`（`builder.stmts.iter().position(...)`；不新增 Builder 字段、不改 `fallback` 漏斗）。条件：`!class_initializer && !stmts.is_empty() && 顶层族引注存在 && (return_type.is_none() \|\| !completes_normally(&stmts))` → 该体呈现为**只剩引注**：丢弃全部非引注语句、`statements = 0`；族引注自身**保留原 reason 与原有 BCI 顺序**，并在尾部追加本方法其余指令 BCI（`instructions.keys()` ∪ unaccounted region BCI），其 `origin` 同步把这些 BCI 记为 derived（source map 仍覆盖被丢弃语句的 BCI）。其余引注（含级联理由行）**逐字保留** |
| 同上 | `mod tests`（~27190） | 新单测：六族理由各命中、六条级联伴随行各不命中（防理由文本漂移） |
| `crates/jarde-java/src/emit.rs` | `fn refuses_void_body(facts)`（`emit` 之前） | `parse_method(descriptor) == Some((_, None))` ⟺ 返回 `V`；commit 与 replay 同读一处 |
| 同上 | `Emitter::body(stmts, declaration, void_member)`（~626） | 写完全部语句后：`void_member && statements == 0 && !body_stmts.is_empty()` → 写 `    jarde_refused_body();\n`（`put`，不经 `stmt()`，故不计入 `Emitted.statements`，报告面保持 `ExplanationOnly`） |

设计取舍（实测驱动，见 §4）：最初按字面"`stmts.clear()` + 全 BCI 单条引注"实现，被三处既有断言否掉——
`p3_compound_lvalue_updates`（引注的原始 BCI 必须在 source map 里仍有文本）、
`p3_platform_collection_widening`（同一方法内 7 条转换族引注的条数必须保持 7）、
`p3_for_add_store`（循环体内的引注形必须维持原呈现）。最终实现同时满足三者：**多条引注全部保留**、
丢弃非引注语句、族引注承担全部 BCI 覆盖与 origin 映射。

六族诊断文本与级联伴随行**未增未改**（锚实测逐字对照见 §2）；非 void 零语句体零改动（`emit.rs` 的条件含
`void_member`）；void 零语句体新增拒绝标记（preaudit 的裁决项）。

## 2. 锚实测（19 锚 + XS.xorSwap + LK.put）

协议：`target/debug/jarde-cli class-source --input <patrol jar> --class <C> --format text`，剥离 `^[[:space:]]*//`
后 `javac --release 8` 编译，编译成功则 `java` 运行并与原 class 运行输出对照。`before` = patrol `results/` 冻结件
（或改动前二进制渲染，已用 `git stash` 后重建的 `cli-old` 复现）；`after` = 本次实现。

| # | 锚 | 原行为 | before | after | 判定 |
|---|---|---|---|---|---|
| 1 | `SA.postSelf`（`i = i++`） | `5` | 编译过、`6`（可编译错文本） | 整方法拒（值级理由逐字）、缺 return 不可编译 | SAFE（整方法拒） |
| 2 | `SD.postSelfDec`（`i = i--`） | `5` | 编译过、`4` | 同上 | SAFE（整方法拒） |
| 3 | `SD.arrSelf`（`a[i] = i++`） | `102` | 编译过、`2` | 同上 | SAFE（整方法拒） |
| 4 | `FS.compoundSelf`（`i += i++ + 1`） | `11` | 不可编译（缺 return） | 整方法拒 + 不可编译 | SAFE（保持不可编译） |
| 5 | `SA.postOther`（跨变量） | `65` | 不可编译（缺 return） | **逐字节不变**（与冻结件仅账本面差异） | SAFE（零回退） |
| 6 | `BF.enable/disable`（void 复合 RMW） | `true/false/true/false/3`、`false/2` | 编译过、`false/false/false/true/0`、`false/0` | 整方法拒 + `jarde_refused_body();` → 找不到符号 | SAFE（整方法拒） |
| 7 | `BG.ienable2`（值消费复合） | `true/false`、`20` | 编译过、`true/false`、`0` | 整方法拒、缺 return 不可编译 | SAFE（整方法拒） |
| 8 | `AD.add`（`elems[size++] = t`） | `x/y` | 编译过、`null/null` | 整方法拒 + 标记 | SAFE（整方法拒） |
| 9 | `PC.viaChain`（链实参 `t++`） | `n5:6` | 编译过（单方法隔离）、`n:6` | 整方法拒、缺 return 不可编译 | SAFE（整方法拒） |
| 10 | `PC.viaArg`（方法实参 `idx++`） | `1` | 部分引注（copy 族理由） | 整方法拒、不可编译 | SAFE（整方法拒） |
| 11 | `PC.viaReturn`（返回位 `counter++`） | — | 整方法拒 | **逐字节不变** | 零回退 |
| 12 | `NI.main`（多消费者） | `3/10/5/true` | 编译过、空输出 | 整方法拒 + 标记（冻结件多消费者级联行被吸收） | SAFE（整方法拒） |
| 13 | `NJ`（2 次消费对照） | `3/10` | 完整恢复 | 完整恢复、编译过、`3/10` 与原始一致 | 行为一致 |
| 14 | `CP.byAnon`（引用转换族） | `[al:20, bo:30, al:40]` | 编译过（补 `CP$User` 后）、`[bo:30, al:40, al:20]`（sort 被吞） | 整方法拒、缺 return 不可编译 | SAFE（整方法拒） |
| 15 | `CP.byNameAge`/`byLen`（对照） | — | 完整恢复 | **逐字节不变**（`comparing/thenComparing/reversed` 全链与 `Arrays.sort` lambda） | 零回退 |
| 16 | `OP.sideEffect`（绑定接收者族） | `…false/S/` | 编译过、`…false//`（`ifPresent` 被吞） | 整方法拒、缺 return 不可编译 | SAFE（整方法拒） |
| 17 | `P02_multianewarray`（lambda 体二维复合，DT-26 冻结腿） | `6` | 编译过、`0` | 整方法拒 + 标记（void lambda companion） | SAFE（整方法拒） |
| 18 | `XS.xorSwap`（依赖链族） | `8/2/9/10/7/6` | **编译过、`3/2/9/10/7/6`**（proposal 记的"现状 SAFE"与冻结件不符，见 §4） | 整方法拒、缺 return 不可编译 | SAFE（整方法拒） |
| 19 | `LK.put`（显式锁，既有整方法引注） | `1/0/true` | void 空体 `{ }`（可编译静默 no-op） | 加 `jarde_refused_body();` → 找不到符号 | SAFE（void 呈现修复） |

诊断面核对：上表各"整方法拒"行的理由行与冻结件同族行**逐字相同**（diff 中该行未被标记为变更）；`SP.postSelf`/
`SD`/`FS`/`BF`/`BG`/`AD`/`XS`/`CP`/`OP`/`P02` 的原诊断族理由均保留一条 verbatim，无新拒绝码。

## 3. 变更有影响但不在 19 锚内的 fixture（实测）

| fixture | 原行为 | before | after | 说明 |
|---|---|---|---|---|
| `SG`（实例/静态复合全类型） | `fx/1` | 编译过、`f/0` | 整方法拒、`找不到符号` | **修复**：旧文本编译且行为错 |
| `DV`（8 种实例复合） | `1/1` | 编译过、`0/1` | 整方法拒、`找不到符号` | **修复**：同上 |
| `GA`（泛型集合内部 `add` 形） | CCE（与原始同） | 编译过、行为与原始**一致** | 整方法拒、`找不到符号` | **覆盖面回退**（安全方向）：旧文本行为正确但属依赖链族，守卫按统一规则拒之；待 root 裁决 |
| `FA`（`this.i++` 语句位等） | `6/3/30/11` | — | 与 before **零行差异**，编译过、`6/3/30/11` | 零回退（无六族诊断，守卫不可能触发） |
| `NI$Inner`、`AN` | — | — | 与冻结件**零行差异** | 零回退 |

## 4. 与 instrumentation / 预审计的偏差

1. **`<clinit>` 不在守卫范围内**（要求文本为"方法"）。理由：`src/enum_constants.rs` 的
   `int_static_field_arguments_preserve_the_same_run_expression_and_runtime_order` 断言类初始化呈现面保留自身
   `@bytecode 14` 引注（该平面由成员逐条呈现承载，枚举投影承载代码面）。守卫若吸收 `<clinit>` 该测试即红。
   实现以 `inputs.physical_method.name == b"<clinit>"` 排除；void 呈现标记不受此限（零语句 `<clinit>` 仍会加标记）。
2. **只有顶层值级引注触发整方法降级；结构体内的嵌套引注维持原呈现**。理由：`tests/p3_for_add_store.rs` 的
   `changed_step_extra_consumer_shared_effect_and_post_use_do_not_move` 断言 `extraConsumer` 的呈现保留
   `while (` 结构与 `@bytecode 13` 引注（该形是循环体内的引注 + 外层结构存活）。守卫因此只读 `builder.stmts` 顶层，
   不记录漏斗级事实；19 锚的族引注实测**全部在顶层**，故不受影响。**残留洞**：循环/`if`/`try` **体内**的族引注
   （如 `extraConsumer`）仍可能留下"结构存活、被拒语句被吞"的可编译文本；该形不在本片 19 锚内，且被既有测试钉住，
   报 root 决定是否单独立片。
3. **级联行保留、覆盖集中在族引注一条上**：整方法降级不新增引注，因此原先伴生的级联行（如 `NI` 的
   "the saved producer at BCI N has no bounded final expression consumer" 序列、`XS` 的逐指令引注、`AD` 的
   "not part of the provable subset"）**全部原样保留**；代价是族引注那一条的 `@bytecode` 列表在尾部追加了本方法其余
   指令 BCI（引注范围加宽，reason 行不变）——这正是"引注范围"面的改动，非诊断文本改动。
4. **generic-Signature 拒绝变体切换（非六族）**：体变成 explanation-only 后，成员先撞上 `src/class_source.rs:3945`
   的既有前置门（`method body or no-body declaration has no complete source proof`），不再走到 `:4071` 的
   `ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under parameterized types`
   （或 `generic_source_shape_unproved: class name has no unambiguous Java source spelling`）。实测出现于
   `CP.byAnon`、`OP.sideEffect`、`GA.add`。两行都是既有文本，未发明码；但**该行确实移动了**，超出"诊断文本不新增不改"
   的字面范围，报 root 裁决。
5. **`XS.xorSwap` 的冻结件实测为不安全**（proposal 记"现状安全/空 body 缺 return"）：`results/jarde-XS.txt` 剥注释后
   `javac` 通过、运行 `3/2/9/10/7/6` ≠ 原 `8/2/9/10/7/6`。本片把该锚从"可编译错文本"改为"整方法拒"，比预期更强；
   proposal 的该条描述需 root 更正。

## 5. 门禁（本 worktree 实测）

- `cargo fmt --all -- --check`：干净（exit 0）。
- `cargo clippy --workspace --all-targets --all-features --locked -- -A …（`.github/workflows/ci.yml` 46–76 逐字 29 项）-D warnings`：
  exit 0。
- `openspec validate --all --strict`：`Totals: 300 passed, 0 failed (300 items)`。
- `cargo test --workspace --no-fail-fast`：**309 targets / 2986 passed / 8 failed**（默认 fail-fast 运行在第 201 个 target
  即停）。8 条失败**全部是其它切片钉住的"旧呈现逐字节/计数/计费"记录**，它们钉的正是本片有意改变的"可编译错文本"；
  逐条如下（`left` = 本片渲染，`right` = 记录）：

| 失败测试 | 文件 | 记录钉住的旧文本 | 本片后的文本 |
|---|---|---|---|
| `the_multianewarray_capture_keeps_its_recorded_status` | `tests/recover_lambda_primitive_array_capture.rs:396` | DT-26 冻结件：`lambda$sum$0` 体 = 引注 + `return;`（**实测编译过、跑出 `0`，原 class 为 `6`**） | 引注（reason 逐字保留）+ 尾部 BCI 加宽 + `jarde_refused_body();` |
| `the_boolean_precedent_renders_byte_identical_to_its_frozen_record` | `tests/recover_write_accessor_field_types.rs:376` | `dt29/PrivateFieldFamily$B.set(ZZ)V` = 引注 + `return;`（void 体，**编译过且静默丢弃字段写**） | 引注（`@bytecode 7 10 5 6` → `7 10 5 6 0 1 2 11`）+ `jarde_refused_body();` |
| `the_walk_proves_the_eighth_edge_and_refuses_the_ninth` | `tests/p3_snapshot_hierarchy_widening.rs:327` | H3.main = 已证 `println(via((H3$Sig) new H3$L7()))` + 引注（L8）+ `return;`（**编译过、L8 调用被吞**） | 整方法拒：只剩引注（L7 行随之消失，计数 1 与 L8 断言仍成立） |
| `lower_parameter_stack_value_is_not_an_extra_throw_read` | `tests/p3_throw.rs:795` | 抛掷操作数呈现 | 整方法拒 |
| `the_fixed_shape_bills_the_counts_the_corpus_pins` | `tests/p5_bulk_corpus.rs:1329` | 计费 `output_bytes=23229` | `output_bytes=23333`（引注面变宽） |
| `the_old_per_method_arms_keep_their_ledger_and_the_same_text` | `tests/p5_bulk_corpus.rs:1695` | 计费 `output_bytes=39513` | `output_bytes=39617` |
| `same_class_method_call_proves_binding_and_reflects_like_the_original` | `tests/same_class_generic_binding.rs` | 同族旧呈现 | 整方法拒 |

**结论**：本片要求的呈现变化与这 8 条"旧呈现逐字节/计费"记录**不可能同时成立**——它们钉住的文本本身即
第一不变量违反（上表前三条已实测"编译过且行为不同"）。因此需要 root 裁决：(A) 由本片同步更新这 8 条记录
（机械重录新文本/新计费，因为任何健全性修复都必然移动它们）；或 (B) 改用"保留语句 + 代码态拒绝标记"
机制（把 `jarde_refused_body();` 从 void 零语句推广到任何触发守卫的体），它保住全部冻结文本的语句面，但会把
`Emitted.statements` 留在非零（报告面由 `ExplanationOnly` 变为 `ContainsStatements`），与本片 requirement 2 的
void 规矩不一致。**本提交不动这 8 条记录**（属其它切片账本，越权修改风险高于收益）。

- 未执行：tasks 3.2 的 corpus 指纹再生（`fixture 双协议`后半）、root 独立重放、CI。
- 未验证为"行为一致"的编译成功样本：`NJ`（`3/10` = 原）、`FA`（`6/3/30/11` = 原）；其余锚均为"编译失败=SAFE"。
- 残留洞（因既有测试钉住，本片不动，见 §4.2）：结构体内部的族引注形（`p3_for_add_store` 的 `extraConsumer`）。

## 6. 收尾：期望记录与测试期望的更新（2026-10-05，coder）

root 按 §5 的 **(A)** 落地：更新受本变更有意改变之呈现影响的记录，实现与设计不动（§1 的两个落点一行未改）。
收录口径只有三类——**整方法拒绝**、**族引注 BCI 集加宽**、**void 零语句体的 `jarde_refused_body();` 标记**；
工作树 diff 逐行复核，未接受其它任何改动。patrol 的 `results/` 冻结件是 §2 的 before 证据，全部保持原样；
`tests/fixtures/`、`fuzz/corpus/` 的源 fixture/jar 一个字节未动（`tests/fixtures/corpus-fingerprint.json` 的根只覆盖
这两处，本片改动的文件都不在其中，指纹无需再生）。

### 6.1 期望渲染记录（5 个文件，`git diff` 逐行核对）

| 文件 | 变更（三类之内） | 原因 |
|---|---|---|
| `.../recover-lambda-primitive-array-capture/baseline/P02_multianewarray-v8.baseline.txt` | ① `@bytecode 2 5` → `2 5 0 1 3 4 6 7 10 11 12`；② 新增成员级 "not recovered … produced no statement (explanation only)" 行；③ `return;` → `jarde_refused_body();` | `lambda$sum$0` 的 void companion 体整方法拒：丢弃非引注语句、族引注承担本方法全部 BCI、零语句 void 体写标记。旧记录钉住的 `引注 + return;` 实测可编译且跑出 `0`（原 class `6`），即第一不变量违反 |
| `.../recover-lambda-primitive-array-capture/baseline/P02_multianewarray-v23.baseline.txt` | 同上 | 同上（v23 腿） |
| `.../recover-lambda-primitive-array-capture/fixed/P02_multianewarray-v8.fixed.txt` | 同上 | 同上（fixed 腿与 baseline 腿逐字节相同） |
| `.../recover-lambda-primitive-array-capture/fixed/P02_multianewarray-v23.fixed.txt` | 同上 | 同上（v23 腿） |
| `.../recover-write-accessor-field-types/pff/pff-baseline-PrivateFieldFamily$B.txt` | ① `@bytecode 7 10 5 6` → `7 10 5 6 0 1 2 11`；② 新增 explanation-only 行；③ `return;` → `jarde_refused_body();` | `set(ZZ)V` 同上；旧记录实测可编译且静默丢弃字段写 |

这些记录的旧文本不是"另一份合法事实"：§3 的实测已证明它们是可编译错文本。测试已按新记录逐字节核对
（`recover_lambda_primitive_array_capture` 与 `recover_write_accessor_field_types` 两个 target 转绿，见 §6.4）。

### 6.2 其它切片的测试期望（4 文件 7 例；期望随已裁决行为移动，断言面不放宽）

| 测试 | 旧期望 | 新期望（本片后实测） | 说明 |
|---|---|---|---|
| `p3_snapshot_hierarchy_widening.rs::the_walk_proves_the_eighth_edge_and_refuses_the_ninth` | `H3.main` 呈现 `via((H3$Sig) new H3$L7())` | 拒绝计数仍 = 1（第八边无拒绝 ⇒ walk 仍证明它）、第九边理由行逐字保留；新增断言 `main` 的 run 是 `ExplanationOnly` 且体写 `jarde_refused_body();` | `main` 是 void 且顶层有第六族引注；旧文本编译过并吞掉 L8 调用 |
| `p3_throw.rs::lower_parameter_stack_value_is_not_an_extra_throw_read` | 引注 + 呈现的 `throw arg0;`（计数 = 1） | 整方法拒：`!text.contains("throw arg0;")`；"恰好一次"断言移入 `assert_boundary_value_or_quote` 仍允许的呈现分支 | 同上（void + 顶层族引注）；旧文本编译过并丢弃其前的那次调用 |
| `p5_bulk_corpus.rs::the_fixed_shape_bills_the_counts_the_corpus_pins` | `many-method-class` `output_bytes = 23229` | `23333`（+104） | 归因见 §6.3 |
| `p5_bulk_corpus.rs::the_old_per_method_arms_keep_their_ledger_and_the_same_text` | `DIRECT_ARM` / `SHARED_ARM` `output_bytes = 39513` | `39617`（+104，两臂同） | 同上 |
| `same_class_generic_binding.rs::same_class_method_call_proves_binding_and_reflects_like_the_original` | 整类渲染编译 + `-Xverify:all` 运行 + 反射 | `SCGA.main`（平台转换族：`String` → 擦除后的 `Comparable`）整方法拒，运行时腿改为编译并运行**本 run 证明的成员**（新增 `proved_unit()`：剔除 `ExplanationOnly` 成员），runner 写出 `main` 所做的调用 | `note` 的声明、签名、绑定理由与"反射 + 行为"对照全部保留 |
| `same_class_generic_binding.rs::receiver_consumed_field_read_keeps_the_field_refusal` | 同上 | 同上（字段 `kept` 的拒绝断言保留；`main` 整方法拒） | 同上 |

§5 表里还有 1 例未列出（第 8 条）：上表两行 `same_class_generic_binding` 的失败当时只记了一行，此处一并记录。

### 6.3 `+104 output_bytes` 的归因（改前/改后二进制同口径实测）

- `many-method-class` 的三个类逐一实测 usage `output_bytes`：`Guarded` 56605 → **56709（+104）**；
  `BooleanContexts` 5371 → 5371（0）；生成的 `p/WideClass` 是 64 个 `public static int memberN(int)`（非 void、
  无六族诊断）——emit 的标记只对 void 成员生效、build 的守卫只对六族诊断生效，两者都不可能移动。
- 因此 case 行与两臂行的 +104 = `Guarded` 四个 explanation-only void 体各多一行 `jarde_refused_body();`；
  `explanation_only` 改前改后都是 4（recorder 实测输出），另外五张 case 行与两臂行的其它七个维度全部未动。
- 行值由 `cargo test --test p5_bulk_corpus --locked -- --ignored --nocapture record_the_billing_table` 读出，
  注释按本文件既有惯例记录重测原因；没有手改其它数字。

### 6.4 收尾后的门禁（本 worktree 实测）

| 门禁 | 结果 |
|---|---|
| `cargo test --workspace --tests --locked --no-fail-fast` | **304 个 run block / 2993 passed / 46 ignored**；本片与 §6.1–6.2 涉及的 target 全绿（收尾前为 4 个 target 红，全部是 §6.1/§6.2 的旧期望） |
| `cargo fmt --all -- --check` | exit 0 |
| clippy（`.github/workflows/ci.yml` 46–76 逐字 29 项 `-A` + `-D warnings`，`--workspace --all-targets --all-features --locked`） | exit 0 |
| `openspec validate --all --strict` | `Totals: 300 passed, 0 failed (300 items)` |
| `cargo test --test p3_execution_comparison --locked -- --ignored`（CI 的 JDK 步骤，本片额外跑） | 3 passed / 0 failed（渲染仍可编译并运行的那个语料未受本片影响） |

三次全量运行里出现过两条**与本片无关的计时抖动**，两次都不在收尾前失败过、单独重跑均通过：
`p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched` 与
`p1_multi_release::physical_evidence_is_the_single_provider_result_it_reports`——两者都把两份报告的
`UsageSnapshot`（含 `elapsed_millis`）整体比较，机器有负载时 0ms/1ms 会互差。隔离重复实测：两条各 25/25 通过。
本片不触碰 plugin/多版本选择路径（改动只在 class-source 的守卫与 void 呈现标记），未改动这两条测试。

`tests/fixtures/corpus-fingerprint.json` 的 `ROOTS` 只有 `tests/fixtures`、`fuzz/corpus`；本片改动的
5 个渲染记录都在 `openspec/evidence/` 下，指纹（§5 记为"未执行"的 tasks 3.2 后半）确认无需再生。

### 6.5 复测锚（真 `javac 1.8.0_432` / `corretto-1.8.0_432`）

协议：`jarde-cli class-source --input <patrol jar> --class <C> --format text`（P02 用 `--policy single-class`），
剥离 `^[[:space:]]*//` 后按 8 号编译器编译，编译成功则运行并与原 class 运行对照。

| 锚 | 原 class 运行 | 剥离后编译（8 号编译器） | 判定 |
|---|---|---|---|
| `SA`（`postfix-self-assign-soundness-patrol/fixture/sa.jar`） | `5/6/65/7/1/aAb/y` | 失败：`SA.java:10/22 缺少返回语句` | SAFE：整方法拒（改前 `SA.postSelf` 可编译、跑出 6） |
| `BF`（`field-compound-soundness-patrol/fixture/bf.jar`） | `true/false/true/false/3/false/2` | 失败：`BF.java:11/15 找不到符号`（2 处标记 = `enable`/`disable`） | SAFE：整方法拒 + void 标记 |
| `XS`（`numeric-idioms-patrol/fixture/xs.jar`） | `8/2/9/10/7/6` | 失败：`XS.java:8 缺少返回语句` | SAFE：整方法拒（改前为可编译错文本 `3/2/…`） |
| `LK`（`explicit-lock-patrol/fixture/lk.jar`） | `1/0/true` | 失败：`LK.java:17 找不到符号`（1 处标记） | SAFE：void 呈现修复 |
| `P02_multianewarray` v8 | `6` | 失败：`找不到符号`（1 处标记） | SAFE：整方法拒；渲染与 §6.1 的 fixed 记录逐字节相同 |
| `P02_multianewarray` v23 | `6` | 同上 | 同上 |

## 7. Root 独立验收（2026-10-06，HEAD=0b82f70e）

- **门禁复现**：`cargo test --workspace --tests --locked --no-fail-fast` 311 targets / 2993 passed / 1 failed=`p4_plugins::the_plugin_plane…`（elapsed_millis 0 vs 1 负载敏感；隔离复跑 25/25 绿）；`cargo fmt --all -- --check` exit 0；clippy 按 ci.yml 46–76 逐字（29 allow + `-D warnings`）exit 0；`openspec validate --all --strict` 300/0。
- **锚独立复验（真 javac 1.8.0_432，HEAD 重编 jarde-cli）**：SA（缺 return）、BF（找不到符号×2 标记）、XS（缺 return）、LK（找不到符号×1）、CP、OP、NI（缺 return/找不到符号）——7/7 剥离编译失败=SAFE，失败原因与设计一致。
- **零回退面**：123 个巡查 fixture 顶层类 spared(7a2f92c9) vs HEAD 源段对比：86 逐字节不变；37 变更全数定性——35=旧渲染含六族诊断的 **void 成员** fail-closed 升级整方法拒（诊断 reason 行逐字保留、BCI 集扩为全方法、非引注语句清零）、2=`LK.put`/`SR.voidBody` 零语句 void 体加 `jarde_refused_body()` 标记。无健康渲染被触碰。
- **裁决追认**：void 成员一律升级（不逐体判可编译性）是 fail-closed 取向——`completes_normally` 无 javac 级编译性 oracle，void 的幸存语句静默 no-op 风险优先于最小 diff；接受。
- **残留（转队列）**：verification §4.2——幸存 loop/if/try 体内的六族引注仍保留外层呈现（p3_for_add_store 钉住），为下一窄片取证对象；§4.4 generic 前置门切换未触碰。
