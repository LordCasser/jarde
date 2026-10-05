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
