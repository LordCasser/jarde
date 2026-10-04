## 1. 取证与基线（root 已完成大半，见巡查 README）

- [x] 1.1 9 类型每型事实表已实测冻结（javap + jarde 现状 8/9 拒 + 编译对照 exit 1/8 错误），见 [results3](../../evidence/java-syntax-2026-10-04/value-returning-write-accessor-patrol/results3/nine-type-measurement.txt)。（root 已完成）
- [x] 1.2 boolean/int 决定性对照（opcode 逐字相同、仅描述符差异）已冻结（results2）。（root 已完成）
- [x] 1.3 实现片开工时重验基线：以主线二进制渲染 `WA` 族确认 8/9 拒仍成立（防主线漂移；若 `d09f5dea` 后有相关改动，以重验为准并报告）。（实现片 2026-10-05 完成：HEAD `60072413` 全新构建二进制渲染，8/9 拒、boolean 恢复，文本面前 178 行与冻结件逐字节相同；fixture SHA256 与冻结件逐一相同。证据 `openspec/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types/baseline/`）
- [x] 1.4 确认 `LongAssignmentResult` 与调用链（`build.rs:7293`、`8221`）对双槽是否需要区分（design Open Question 1）。（实现片 2026-10-05 读码完成：双槽**不需要**槽宽区分——`LongAssignmentResult` 无槽宽字段、消费按 SSA 值、槽宽语义在 `field_value` 描述符分支；`fields.claim` 为只读查找。但发现消费侧 `assignment_result_statement` ~22538 的描述符常量门 `!= if boolean_accessor {"Z"} else {"J"}` 必须随表泛化，否则 7/9 类型 prove 通过后仍被拒成空 stub——root 预审计"消费链零改动"漏记该门；已按停手条件 (b) 上报待裁决，证据 `FINDING.md`）

## 2. 泛化实现

- [ ] 2.1 按设计决策 1 的封闭每型表重构 `BooleanAccessorAssignment`（改名 `WriteAccessorAssignment` 或等价，boolean 成为一行），描述符/opcode/槽宽/stack-locals 下限全部查表。
- [ ] 2.2 决策 2 的全部结构判据**逐字保留**（单块、无异常表、BCI 布局、Operation 序列、`fields.claim`、owner、预算）——diff 审查时逐条核对。
- [ ] 2.3 `long`/`double`（`dup2_x1` + 双槽 + stack/locals 5/3）单独验证；引用型描述符一致性按 design Open Question 2 确认。
- [ ] 2.4 不得为避免改名而复制平行处理器；不得外推表外类型。

## 3. 验证与验收

- [ ] 3.1 主锚：`WA` 族 9 访问器全恢复，渲染源集 `javac --release 8` exit 0、运行输出与原 class 逐行一致（修复前 8 错误）；0 引注。
- [ ] 3.2 零回退：`PrivateFieldFamily`（boolean 先例）渲染逐字节不变；`accessor.rs` 路径既有负例（无复制返回形）仍拒；`p3_accessor_edges` 家族断言全绿。
- [ ] 3.3 负例 (a) 表外描述符（合成探针）(b) static 形 (c) 结构判据破坏形——各保持拒绝。
- [ ] 3.4 corpus 双腿扫描：预期 diff 为空（语料 0 真实 `access$`）；出现任何差异停下报告。
- [ ] 3.5 门禁全量：`cargo test --workspace --tests --locked --no-fail-fast`（基线 301/2970，flake 单测复跑两轮判定）、fmt、CI-exact clippy（ci.yml 46-76 逐字）、`openspec validate --all --strict`（275 项）、`git diff --check`、再生 corpus fingerprint。
- [ ] 3.6 root 独立复核：每型表与 javap 实测一致、结构判据零放宽（diff 逐条）、主锚/零回退/负例实测、corpus 空 diff；更新 EM-15 账本（summary.md 从"待独立立项"改为指向本片并记录落地）。（留 root）
