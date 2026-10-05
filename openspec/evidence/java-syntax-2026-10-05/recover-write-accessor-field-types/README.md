# recover-write-accessor-field-types —— 实现片工作记录（2026-10-05）

**状态：停手条件 (b) 触发，实现零改动，待 root 裁决**（详见 [FINDING](FINDING.md)）。
本目录目前只含**未受阻部分**的实证：任务 1.3 基线重验、任务 1.4 调用链确认、两条来源待鉴别的答复存档。

## 任务 1.3 基线重验（已完成，无主线漂移）

- worktree HEAD：`60072413`（与任务书基线一致，detached HEAD，干净树）。
- fixture 逐字节核验：`probe/WA.class`、`probe/WA$S.class` 与巡查冻结件 SHA256 **逐一相同**
  （`00576b25…` / `f981b690…`，对照 `results3/fixture-sha256.txt`）。
- 二进制：`target/debug/jarde-cli`（本 worktree 全新构建，`cargo build -p jarde-cli --locked`，exit 0）。
- 渲染：`jarde-cli class-source --input probe/wa.jar --class WA --format text`（**根类**，绝对路径）。
- 结果：**8/9 拒**（int/long/double/String/byte/short/char/float 全部
  `// jarde: not recovered: … produced no statement`，计 8 条）；boolean `access$002` 恢复为
  `arg0.flag = arg1;` + `return arg1;`。与巡查冻结测量（8/9 拒）**一致**。
- 零漂移对照：本渲染文本与冻结件 `results3/WA-rendered.txt` 的 diff 为
  `178a179,7699`——**前 178 行逐字节相同**，多出部分全部是 `--output` 落盘的记账面
  （`coverage.*`/`usage.*`/`methods.*.outcome…`），无任何文本面差异。
- 产物：[`baseline/WA-rendered-baseline.txt`](baseline/WA-rendered-baseline.txt)（含 jarde 自述头，
  已核对首行 `// jarde: presentation of` —— 非假零）。

## 任务 1.4 调用链确认（design Open Question 1，已完成，纯读码）

结论：**双槽（J/D）不需要 `LongAssignmentResult` 结构或调用链的槽宽区分**；预审计这一半成立：

- `LongAssignmentResult`（build.rs:8547-8553）只承载 `receiver_source`/`parameter_source`/
  `receiver_copy`/`field_copy`/`store_bci`/`return_bci`/`anchors` —— **无槽宽字段**。
- 消费方 `assignment_result_statement`（22517 起）按 SSA 值 `render_value`；槽宽语义由
  `field_value`（22644 起）按 `evidence.descriptor` 的描述符驱动分支承担（`Type::Boolean` 特判 /
  `Byte|Char|Short` 收窄 cast / 其余 `meeting_position`）。
- `fields.claim`（field.rs:133）是 `&self` 只读查找，prove 时的 claim 在 emission 时可复查，无一次性所有权问题。

但发现**预审计遗漏的一处类型事实**（停手条件 (b) 的裁决对象）：emission 门
`evidence.descriptor != if boolean_accessor { "Z" } else { "J" }`（build.rs:22538 附近）把消费侧
期望描述符硬编码为两值，9 类型中 7 个在 prove 通过后仍会被该门拒成空 stub —— 见 [FINDING](FINDING.md)。

## 本轮未做（全部因停手条件 (b) 阻塞）

tasks 2.1-2.4（泛化实现）、3.1-3.5（主锚/零回退/负例/corpus/门禁）——均依赖上述裁决。
**`crates/`、`tests/` 零字节改动**（`git status` 仅有本证据目录）。
