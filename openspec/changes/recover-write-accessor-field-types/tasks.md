## 1. 取证与基线（root 已完成大半，见巡查 README）

- [x] 1.1 9 类型每型事实表已实测冻结（javap + jarde 现状 8/9 拒 + 编译对照 exit 1/8 错误），见 [results3](../../evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/results3/nine-type-measurement.txt)。（root 已完成）
- [x] 1.2 boolean/int 决定性对照（opcode 逐字相同、仅描述符差异）已冻结（results2）。（root 已完成）
- [x] 1.3 实现片开工时重验基线：以主线二进制渲染 `WA` 族确认 8/9 拒仍成立（防主线漂移；若 `d09f5dea` 后有相关改动，以重验为准并报告）。（实现片 2026-10-05 完成：HEAD `60072413` 全新构建二进制渲染，8/9 拒、boolean 恢复，文本面前 178 行与冻结件逐字节相同；fixture SHA256 与冻结件逐一相同。证据 `openspec/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types/baseline/`）
- [x] 1.4 确认 `LongAssignmentResult` 与调用链（`build.rs:7293`、`8221`）对双槽是否需要区分（design Open Question 1）。（实现片 2026-10-05 读码完成：双槽**不需要**槽宽区分——`LongAssignmentResult` 无槽宽字段、消费按 SSA 值、槽宽语义在 `field_value` 描述符分支；`fields.claim` 为只读查找。但发现消费侧 `assignment_result_statement` ~22538 的描述符常量门 `!= if boolean_accessor {"Z"} else {"J"}` 必须随表泛化，否则 7/9 类型 prove 通过后仍被拒成空 stub——root 预审计"消费链零改动"漏记该门；已按停手条件 (b) 上报待裁决，证据 `FINDING.md`）

## 2. 泛化实现

- [x] 2.1 按设计决策 1 的封闭每型表重构 `BooleanAccessorAssignment`（改名 `WriteAccessorAssignment` 或等价，boolean 成为一行），描述符/opcode/槽宽/stack-locals 下限全部查表。（实现 `ee68bfd8`：`write_accessor_shape`/`write_accessor_key` 封闭表，opcode 为仓库常量、stack/locals 下限取自冻结 patrol javap。root 2026-10-06 diff 审计确认）
- [x] 2.2 决策 2 的全部结构判据**逐字保留**（单块、无异常表、BCI 布局、Operation 序列、`fields.claim`、owner、预算）——diff 审查时逐条核对。（root 2026-10-06 逐条核对 `af6f65c4^1..af6f65c4` build.rs diff：单块/无 clone 块/无异常表/`store_bci:3`+`return_bci:6`+`anchors:[0,1,2]`/Operation 序列/claim/owner/`!is_static`/预算 poll 全在；唯一变化是 boolean 常量→表键，与 FINDING 提案逐字一致）
- [x] 2.3 `long`/`double`（`dup2_x1` + 双槽 + stack/locals 5/3）单独验证；引用型描述符一致性按 design Open Question 2 确认。（`two_slot` 分支 `dup2_x1`+slots 2+min 5/3；引用型 `Value::Ref(RefType::Named)` 名逐字等于表键描述符；九型 javap 体逐行核对：J/D `lload_1`/`dload_1`+`dup2_x1`+`lreturn`/`dreturn`，String `aload_1`+`areturn`，BCI [0,1,2,3,6] 九型一致——root 2026-10-06 对 `results3/WA-javap-code.txt` 实测）
- [x] 2.4 不得为避免改名而复制平行处理器；不得外推表外类型。（旧 `BooleanAccessorAssignment` 原地改名泛化，无平行副本；`_ => None` 表外描述符保持拒绝。root 2026-10-06 diff 审计确认）

## 3. 验证与验收

- [x] 3.1 主锚：`WA` 族 9 访问器全恢复，渲染源集 `javac --release 8` exit 0、运行输出与原 class 逐行一致（修复前 8 错误）；0 引注。（`tests/recover_write_accessor_field_types.rs::the_nine_write_accessor_types_all_recover` + `the_recovered_wa_text_recompiles_and_prints_the_original_line`；CI run 37402654588（success，含本代码）全量跑过。root 2026-10-06 据合并测试 + CI 核对；行为抽验见 3.6）
- [x] 3.2 零回退：`PrivateFieldFamily`（boolean 先例）渲染逐字节不变；`accessor.rs` 路径既有负例（无复制返回形）仍拒；`p3_accessor_edges` 家族断言全绿。（`the_boolean_precedent_renders_byte_identical_to_its_frozen_record` 断言逐字节；`p3_accessor_edges.rs` 在 CI 同跑。root 2026-10-06 核对）
- [x] 3.3 负例 (a) 表外描述符（合成探针）(b) static 形 (c) 结构判据破坏形——各保持拒绝。（八探针 `tests/fixtures/recover-write-accessor-field-types/probes/`（bcishift/dupbreak/handler/mismatch/nullrecv/unsynth/void/wrongowner）+ 测试断言拒绝；CI 37402654588 覆盖。root 2026-10-06 核对）
- [x] 3.4 corpus 双腿扫描：预期 diff 为空（语料 0 真实 `access$`）；出现任何差异停下报告。（合并含 `corpus-fingerprint.json` 再生 +75 条目；CI 37402654588 `p5_corpus_fingerprint` 绿。root 2026-10-06 核对）
- [x] 3.5 门禁全量：`cargo test --workspace --tests --locked --no-fail-fast`（基线 301/2970，flake 单测复跑两轮判定）、fmt、CI-exact clippy（ci.yml 46-76 逐字）、`openspec validate --all --strict`（275 项）、`git diff --check`、再生 corpus fingerprint。（CI run 37402654588 [success, head `5a6ec48e`] 四 job 全绿，其代码树含本片全部提交——test/fmt/clippy/openspec 与 CI 同口径；root 2026-10-06 以该 run 为门禁证据）
- [ ] 3.6 root 独立复核：每型表与 javap 实测一致、结构判据零放宽（diff 逐条）、主锚/零回退/负例实测、corpus 空 diff；更新 EM-15 账本（summary.md 从"待独立立项"改为指向本片并记录落地）。（留 root。**2026-10-06 进行中**：每型表 vs javap ✓（九型逐行）、diff 零放宽 ✓（逐条）；余：本地构建的 WA 锚/PrivateFieldFamily 行为抽验（构建窗口让道 temporal 切片后执行）+ summary.md 账本更新）
