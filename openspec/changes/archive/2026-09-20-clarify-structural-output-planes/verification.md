# 验证记录

本 change 是**纯文档/契约**修正：无 Rust 改动，无行为变更（下述门禁用于**证明**这一点）。实现提交见仓库历史（紧邻本文件归档提交之前）。

## 契约陈述（任务组 1）

四份 delta 均已在规划阶段写好，本次逐条核对确认满足任务要求：

| 任务 | delta | 核对结果 |
| --- | --- | --- |
| 1.1 | `recovery-validation` MODIFIED `Separate representation and validation statuses` | 写入「每个平面只回答自己的结构性问题；任意组合——包括 `Structured`+`contains_statements`+`complete`+`CompleteWithinSchema`+无诊断同时成立——MUST NOT 构成语义等价或值正确声明」，并给出调用方动作（受控执行对照，或把产物当作假设）。主规格原有 **6 个 scenario 逐字节保留**，新增 2 个 |
| 1.2 | `java8-recovery` MODIFIED content requirement | 写入 `Produced` 与 `content` 必须分开统计（含 182,883 中 30,452 的口径证据）、两种分类口径分别命名。原有 **4 个 scenario 逐字节保留**，新增 2 个 |
| 1.3 | `analysis-contracts` ADDED `Diagnostic codes are version-scoped identities` | 含声明码含义变化证据（`jre_declaration_class_not_in_run` 在单 artifact 上 10,720 次 → 语料全局 0 次；`jre_declaration` 出现在每个 artifact）与「比较须同时声明两个引擎版本与各自 code 词汇」 |
| 1.4 | `measured-execution` ADDED `Counting dimensions are read within one request shape` | 含 `archive_entries` 的形状证据（28/29、约 6,009、500–1,296、3,702，container tree root 会走子容器）与「比较行须声明请求形状、roots/profile 与引擎版本」 |

## 文档落点（任务组 2）

**措辞与 owner delta 逐字节相同**（脚本按 `str.count` 比对，非目视）；四处落点互不一致的可能性因此为零：

| 文件与位置 | 单元 |
| --- | --- |
| `README.md:21`（恢复产物说明之后） | 平面读法 + `Produced`/`content` 分离 |
| `README.md:251`（任务链 `recover` 示例之后） | 同上（同一契约适用于该示例的报告） |
| `docs/support-matrix.md:50`（`decompile-quality` 条目末） | 平面读法 |
| `docs/support-matrix.md:51`（`output-level` 条目末） | `Produced`/`content` 分离 |
| `docs/support-matrix.md:142`（P5「局部 vs 全范围」段末） | 计数按请求形状读 |
| `openspec/benchmark-protocol.md:9`（冻结基线末） | 诊断 code 词汇随引擎版本登记 |
| `openspec/benchmark-protocol.md:82`（记录列之后） | 计数与比较行的声明要求（形状、roots/profile、引擎版本、scope 说明） |

**只增不改**：逐行 diff 显示原有每一行都作为新行的前缀存活（README 272→276、支持矩阵 215→215 两处追加、协议 102→105）；没有改写任何既有状态词、语料行、历史数字或规则（`Partial` 一处未动）。

## 机械锚点与编辑性条目（任务 2.4）

| 条目 | 机械锚点 | 分类 |
| --- | --- | --- |
| 平面是结构性的 | 本 change 内无独立断言。其真实证据是 `tests/p3_execution_comparison.rs`——**2 个测试均为 `#[ignore]`**（需 JDK），不在默认 workspace 门禁内，由 CI 的 JDK job 以 `--ignored` 运行 | **编辑性**（本 change 只写契约） |
| `Produced` ≠ content | `tests/p3_content.rs::explanation_only_means_no_emitted_statement`、`::a_stopped_run_reports_not_produced`（均在通过的 4 项内） | 机械半 + 编辑性 |
| 诊断 code 词汇 | `tests/p3_declaration_handoff.rs` 的「交接后不再出现 `jre_declaration_class_not_in_run`」「缺事实时的拒绝」「`jre_declaration` 在存活集合内」三处断言 | 机械半 + 编辑性 |
| 计数按形状 | `tests/p2_entry_counts.rs`、`tests/p1_artifact_tree.rs`、`tests/p5_container_lookup.rs`、`tests/p5_benchmark.rs` 中形状/局部性/预算对照用例 | 机械半 + 编辑性 |

按组 2 的要求，**未新增断言散文的测试**：四处一致性与「文档不能移动引擎」的核验是一次性脚本检查，不落为测试。

## 门禁（本机）

| 门禁 | 结果 |
| --- | --- |
| `cargo test --test p3_content --locked` | 4 passed / 0 failed |
| `cargo test --test p3_declaration_handoff --locked` | 8 passed / 0 failed |
| `cargo test --test p5_corpus_fingerprint --locked` | 5 passed / 1 ignored |
| `cargo fmt --all -- --check` | 通过 |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1258 passed / 0 failed / 6 ignored**，与基线一致（证明文档变更未移动引擎） |
| `openspec validate --all --strict --no-interactive` | 20 passed / 0 failed（归档前） |

反向核验：没有任何测试或源码读取 `README.md`（根）、`docs/support-matrix.md` 或 `openspec/benchmark-protocol.md`（只有 fixture 目录下的 `README.md` 是语料输入），因此文档不可能影响引擎行为。

## 边界与未处理项（如实登记）

- **计数器的 scope/形状不是报告字段**：`Limits`/`UsageSnapshot`/`RecoveryReport`/`MethodRecoveryReport` 都没有 `shape` 字段，形状只隐含在调用方的 snapshot/root 声明里。因此「声明形状」是**测量记录的职责**，不是引擎输出，也无法用测试断言。
- **引擎版本与 code 词汇没有机器发布的载体**：`src/`、`crates/` 中没有 `CARGO_PKG_VERSION`/`engine_version` 之类的可读版本，诊断码是普通字符串字面量，没有词汇表或摘要。「code 词汇随引擎版本登记」同属**外部记录职责**。
- **条目 1 的机械证据不在默认门禁内**（见上表），依赖 CI 的 JDK job。
- 既有文档张力，**已报告未静默修改**（超出「只增不改」范围）：`benchmark-protocol.md` 的「预期变化」仍含跨形状的量级期望与 `jre_declaration_class_not_in_run` 大幅减少的预登记（后者已自带免责，二者现由新要求治理）；`README.md:19` 与支持矩阵既有同义改述保留，故那两段同时含改述与契约原文；`README.md:3` 的状态行仍写 `8586356` 的 1254/0/6（带 SHA 的历史标注，非平面冲突）。
- 交叉引用修复：本 change 与两个已归档兄弟 change 之间的 `../<name>/proposal.md` 链接已改为指向归档位置（见归档提交）。
