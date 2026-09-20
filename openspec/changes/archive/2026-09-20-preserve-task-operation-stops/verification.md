# 验证记录

实现提交见仓库历史（紧邻本文件归档提交之前）。反例来自独立复核 [completion-review](../../completion-review.md)；本文件记录修正后的库/CLI 证据。CI 结果随后回填。

## 两条反例的关闭

用复核脚本原样重跑（受控 JAR：有效 `NestedEval.class` + 损坏的 `x/NestedEval.class`；`bad-body` 为 fixture 第 311 字节 `0x1a`→`0xff`）。**左列为修正前，右列为修正后**，均为库层值（CLI 的 JSON 即库报告序列化）：

| 场景 | 修正前 | 修正后 |
| --- | --- | --- |
| 只有有效 fixture | `Performed`/`complete`，`method_bodies=1`，exit 0 | 不变（正向对照） |
| 有效 → 损坏同名候选 | `Performed`/`complete`，**执行了 body**，exit 0 | `Incomplete`/`failed{classfile_decode}`，candidates=1，`method_bodies=0`、IR 构造 0，exit **4** |
| 损坏 → 有效 | `Err(operation_target_not_found)`，exit 2 | `Incomplete`/`failed`，candidates=0，exit **4**（不再宣称不存在） |
| 同上、class-view | `Err(operation_target_not_found)`，exit 2 | `Incomplete`，candidates=0，exit **4** |
| 一个 body 在 BCI 0 解码失败（另有一个正常方法） | 顶层 **`complete`**（CLI 靠自己补算才 exit 4） | 顶层 **`partial{classfile_instruction_decode}`**，bodies `[partial, complete]`，exit 4 **由库报告决定** |

复核基线（修正前六行）在本次改动之前采集，见上文左列；每条新断言另行用变异证明可证伪（下述 M1–M4）。

## 实现

- `OperationOutcome<T>` 增加 **`Incomplete(Box<TargetCandidates>)`**（BREAKING，穷尽匹配一次性迁移；`TargetCandidates` 形状不变，预算的真实 usage 在 `execution.usage` 中）。完整性判断位于共享搜索/`find_targets` 之后、**候选数量判断之前**：`execution` 不是 `Complete` 即 `Incomplete`；只有在搜索完成之后，数量才决定 `Performed`/`Ambiguous`/未找到。内部 `ClassBinding`/`MethodBinding` 同步带该分支。
- **`class_view` 在库内汇总 body 停止**：`Read.execution` 与 `Refused.execution` 经既有 `merge_execution`/`stop_priority` 合入顶层（`NotDeclared` 不是失败），`usage` 由同一个 `Budget` 用 `with_usage` 收尾（不重新初始化、不重复计费）。每个 body 仍保留自己的 stages/coverage/stopped_at/execution；局部解码损坏被隔离且后续显式请求的 body 仍会执行；新增 `ends_the_request`（仅 Cancelled / `BudgetExceeded`）在新的预算/取消状态下不再启动后续 body。
- **结构 coverage 与 body 停止分离**：`structure_complete` 在任何 body 运行**之前**由 execution + `read.facts.stopped_at` 采样，喂给 `class_view_coverage` 的是它而非合并后的平面——因此坏 body 场景里成员表仍报 `complete_within_schema`（`class_fields 0..1`、`class_methods 0..5`、无 skipped），而 body 自己的平面是 `partial`（`method_code_bci` 扫过 `0..0`、跳过 `0..8`）。
- **CLI 只做适配**：`class_view_plane` 现在只读 `report.execution`（原先遍历 body 的第二次实现被删除——它正是与库的 `complete` 不一致的来源）；`Incomplete` 序列化库原值并退出 4；完整歧义仍 3、真实输入错误仍 2；JSON 与库逐字段一致，text 保留同一停止事实。

## 永久回归与变异证据

`tests/task_operations.rs`：`an_unfinished_name_search_is_neither_a_unique_nor_a_missing_target`（三种顺序 + 正向对照：完整唯一、完整缺失、真歧义、显式物理身份，含两次运行的确定性）、`a_stopped_selection_publishes_zero_or_more_candidates_and_runs_nothing`（0 候选的 `class_headers=1`、预取消、成员表截断在 1 与 0 个已确认候选下、类选择对照）、`a_view_with_one_stopped_body_is_not_complete_and_keeps_the_other_body`（复核的 byte-311 改动、顶层 `Partial`、正常 body 与成员表 coverage 保留、确定性）；`a_tight_override_stops_with_the_terminating_dimension` 加强为断言第二个 body 从未启动。`crates/jarde-cli/tests/task_cli.rs`：`an_unfinished_name_selection_exits_four_and_is_the_librarys_own_document`（4/3/2 三态、`assert_same_document`、text↔JSON 行对应）、`a_stopped_body_is_an_incomplete_view_from_the_librarys_own_plane`。

变异（每次施加后运行、以 `cmp` 确认文件字节复原）：**M1** 恢复「只看候选数」→ 顺序用例报 "executed against a target an unfinished search never bound"、选择用例报 `operation_target_not_found`、CLI 用例 `outcome == "performed"`；**M2** 去掉 `stopped_at` 吸收 → 成员选择用例执行；**M3** 去掉 body 合并 → 库顶层 `Complete`、CLI 退出 0 而非 4；**M4** 把合并后的平面喂给 `class_view_coverage` → 成员表状态变 `Partial` 而非 `CompleteWithinSchema`。

## 实现者自定取舍（已由我核对并接受）

- **成员表停止只吸收进「成员选择」**（新增 crate 私有 `SearchSubject::{Declaration, Member}`，由 `find_targets` 按 `query.member` 选择、`bind_class` 用 `Declaration`）。理由：delta 的场景说的是成员选择「不把部分重载集合宣称为唯一匹配」；类绑定的证据是已确认的 header，把类选择因成员表截断判为未完成会与已归档的 A13 契约（损坏成员不抹除类）冲突，并使 `a_damaged_member_record_isolates_to_its_own_member` 变红。类路径仍发布 `MemberTableStop`（顶层合并、诊断、partial coverage、`Refused` body），只是不否认类身份。**我的判断：与规格一致**——delta 要求的是「范围与**相关**成员表完整后才作唯一或缺失判断」，成员表对「这是哪个类」不是相关证据，而对「哪个重载」是。
- 成员停止诊断进入搜索时不计费（与 `stop_diagnostic` 的「控制元数据」规则、`list_members`/`class_view` 一致），而不是走按条目计费的 `navigation_path_name_mismatch`。
- `operation_target_not_found` 现在只可能来自**已完成**的搜索，其消息里的 "examined X of Y" 恒为 N of N，故未改写消息。

## 门禁

| 门禁 | 结果 |
| --- | --- |
| `cargo fmt --all -- --check` | 通过 |
| `cargo test --test task_operations --locked` | 32 passed / 0 failed |
| `cargo test -p jarde-cli --test task_cli --locked` | 11 passed / 0 failed |
| `cargo test --workspace --all-targets --all-features --locked --no-fail-fast` | **1254 passed / 0 failed / 6 ignored**，连跑两次一致（基线 1249，+5 新回归） |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | 通过 |
| `cargo test --test p3_execution_comparison --locked -- --ignored` | 2 passed / 0 failed（17.2 s） |
| `openspec validate --all --strict --no-interactive` | 20 passed / 0 failed（归档前） |
| 实现提交的 CI（`8586356`） | [run 35506187596](https://github.com/LordCasser/jarde/actions/runs/35506187596) **四 job success**：stable（fmt、clippy `-D warnings`、两轮固定 seed 全量测试、JDK 25 oracle、P3 编译执行对照、依赖边界、OpenSpec strict、`git diff --exit-code`）、MSRV 1.88.0、supply chain、fuzz smoke |

## 边界

- 本次只修选择与执行证据的传播：名称匹配、物理身份、runtime root 规则、恢复能力、container cache、分页与性能均未改；R8/R9 未重开。
- 既有弱点、本次未动：`method_code_facts` 会重新解析类，因此成员记录损坏的类里 body 仍是 `Refused{classfile_decode}`（独立债务，复核已单列）；`ClassViewBody::Read.diagnostics` 在本构建中恒为空；搜索仍无法区分「候选读取失败」与「候选从未到达」，只发布范围。均未重开。
- 性能专项 `optimize-demand-workloads` 保持 0/22；本次不是性能交付。
- benchmark 冻结基线随之更新为本次修正提交（`openspec/benchmark-protocol.md` 已同步），`85828c4` 降为历史比较臂。
