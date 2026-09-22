# G0 1.1：冻结基线、构建参数、fixture 摘要与 design §1 源码复核（2026-09-22）

本文件登记 `optimize-demand-workloads` 任务 1.1 的产物：一个**冻结 revision**、它的构建参数、
它读的字节摘要，以及 design §1 那张"已定位的工作形态"表在当前源码上的逐行复核。表里每条给出
**文件与符号**，并按"仍存在 / 已修复或已改变 / 未确认"三类分开列；不确定的项保持未确认，不用旧
数字或推断补全。

## 1. 冻结的基线

| 项 | 值 |
| --- | --- |
| baseline revision | `ba2076af2936ae9afe214899f40695dbdc503f4a`（worktree 基线 = 主工作区 HEAD） |
| 本专项自身的 diff | 仅 `tests/p5_optimize_workloads.rs`（新）与 `openspec/changes/optimize-demand-workloads/evidence/**`（新）；`git status --porcelain` 无产品代码改动，`git diff --stat` 为空 |
| 工具链 | `rustc 1.98.1 (48a229cea 2026-09-01) (Homebrew)`、`cargo 1.98.1 (797e8a9bc 2026-08-05)`；`edition 2024`，`rust-version 1.88` |
| 构建参数 | `cargo build --release --locked`；测量入口 `cargo test --release --test p5_optimize_workloads --features test-support`（观测口需要 `test-support`） |
| 工作负载 harness | `tests/p5_optimize_workloads.rs`（本专项新增，见 [g0-workloads.md](g0-workloads.md)） |
| 对照 fixture（仓库内、确定性） | 内存构造的 ZIP（`rawzip` dev-dep + 24 个已提交 `.class`），15824 字节，blake3 `145f7903f52e16d087584477c2d30c413023547c16e6d168bea3c558b9030854`，24 类 / 183 条方法记录 |
| fixture 的输入摘要在哪里 | `tests/fixtures/corpus-fingerprint.json`（53893 字节，sha256[:16] `3decfd213b77fbfbbd6343c82d9f2e80`，schema `jarde-corpus-fingerprint/1`）逐文件固定 blake3；本期 harness 引用的 24 个 `.class` 全部在其中（清单见 [g0-workloads.md](g0-workloads.md)） |
| 真实语料（次级证据，不在仓库内） | `bcprov-jdk15on-152.jar`，2903072 字节，sha256[:16] `5329ddefb3c92927eef5d5e28f955a96`，路径 `/Users/lordcasser/workspace/vulnerability/vulhub/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar`（字节数与摘要前缀与 `openspec/benchmark-protocol.md` 的表一致） |

本轮**未**使用 `S2-009.war`：它的环境声明是 54 条绑定了 snapshot 身份的 load root，
生成该声明属于 bulk 子项 campaign 自己的口径，本文不为它重建第二份环境声明。真实语料的正式候选
仍按 `benchmark-protocol.md` 的要求在独立 worktree 构建、独立 `CARGO_TARGET_DIR` 并记录二进制
摘要；本文件登记的是 G0 基线与可复现入口，不是该协议的完整执行。

## 2. design §1 源码复核（逐行，带符号）

按任务要求分三类落表：

- **仍存在**（9 行中的 6 行）：O1 的定向访问入口、多 consumer 的 unit 读取（改名后）、`class_view`
  的共享、prepared 的 `FactsKey/take/keep` 与容量、`open/root_bytes/read_verified`、CLI 的 JSON 长度
  固定点——这些观察在当前源码里都能指到具体文件与符号。
- **已修复或已改变**（2 行）：`providers.rs::tree_candidates`（整树路径）已不存在；query 的
  "全量收集后分页"已被 D3/D4 的可停止扫描取代。两者的行为都**需要在 2.1/2.3 用当前入口重测**，
  不能沿用 design 里那句描述。
- **未确认**（1 行 + 两项）：单方法恢复"仍有 driver/callee 重读"只确认到"普通入口已能走 prepared
  版本"，剩余重读量与本档案里 clone/toString 的 10/28 条历史记录一样**保持未确认**（§4）。

| design §1 的观察 | 当前符号与位置 | 判定 |
| --- | --- | --- |
| O1 已交付：指定 container 定向查名、有界 backing 复用 | `crates/jarde-reader/src/artifact.rs::container_candidates`（931 行）、`::container_record`（948 行） | **仍存在**。其验收归 `bound-container-lookup`（`b22ea04`，8/8），本专项只在 2.1 复核计数证据 |
| O1 的整树路径 `providers.rs::tree_candidates` | 全仓库无该符号；`crates/jarde-jvm/src/providers.rs` 现有 3722 行且无该入口 | **名称已失效**（`git log -S tree_candidates`：随 `b22ea04` 引入，此后被替换/改名）。行为未复测，2.1 必须用当前入口重新定位 |
| 多 consumer 分别 materialize/parse 同一 class | `crates/jarde-query/src/xref/mod.rs::ScanUnit::read_unit`（1208 行，`impl ScanUnit` 起于 932 行）、`xref/{code,metadata,bootstrap}.rs` | **符号改名后仍存在**（design 写的是 `ScanContext::read_unit`）。"同一 unit 被多个 consumer 重复 materialize/parse 的占比"**仍未测** |
| class_view 已共享同次 class 字节与成员列举 | `src/facade.rs::class_view`（851 行）、`::body_result`（5428 行） | **仍存在**；`class_view` 在 982–1010 行按次 `PreparedClass::prepare` 并复用同一 preparation（`crate::d0_counts::class_prepared`） |
| 单方法恢复仍有 driver/callee 重读 | `crates/jarde-jvm/src/callee.rs::read_callees`（318 行，自身读类）、`::read_prepared_callees`（377 行，不读类）；调用点 `src/facade.rs::read_named_callees`（3056 行）、`::read_prepared_named_callees`（3078 行） | **部分已修复**：普通入口已能走 prepared 版本（`93432c2`、`7314ff4`、`D2`）。哪条路径实际走哪个调用点、剩余重读多少属于 2.2 的**计测量**，本文不下结论 |
| classfile 方法定位 | `crates/jarde-reader/src/classfile.rs::method_code_facts`（4619 行） | **仍存在**；`K × M` 定位检查的实际占比未测 |
| prepared 路径可信 digest 与 CP/Header `Arc`（`642e49f`） | `crates/jarde-reader/src/facts_cache.rs::FactsKey`（292 行，`pub(crate)`）、`::take`（918 行）、`::keep`（1019 行）、`::FactsCapacity`（180 行）、`FactsReport`（456 行） | **仍存在**；容量 = entries + retained_bytes，满则拒绝（`refused_capacity` / `refused_capacity_bytes`），与 design 一致 |
| provider 全量收集后分页、续页重扫 boundary unit | `providers.rs` / `ProviderScan` / `scan_units` 均已不存在；当前为 `xref/mod.rs::scan_candidates`（420 行）、`UnitScan`（781 行）、`UnitStream`、`build_cursor`（1694 行） | **行为已改变**（`67d9f81` "let a query page stop the scan it pays for"、`db56958` "walk a scope by entry, so a page stops before the entries behind it"）：design 那句描述的是 D3/D4 之前的形状，其"收益与 coverage 变化"必须在 2.3 按当前代码复测 |
| open 完整读入后 hash、entry/root 拥有所有权的复制 | `crates/jarde-reader/src/artifact.rs::open`（248 行）、`::root_bytes`（283 行）、`::read_verified`（3245 行） | **仍存在** |
| JSON 长度固定点反复遍历 | `crates/jarde-cli/src/main.rs::write_success`（516 行）：`count_response_bytes` 与 `MAX_RESPONSE_LENGTH_ITERATIONS` 构成的固定点循环 | **仍存在** |

## 3. `85828c4`、`8586356` 的历史行为边界

| 提交 | 日期 | 内容 | 本轮登记 |
| --- | --- | --- | --- |
| `85828c4ae2b3ebf1e6d2dc9c822ee768bbf002a2` | 2026-09-20 | `feat: give the task chain a command line, and keep it thin` | 历史比较臂边界：它**存在** R1/R2 停止传播缺口（`bafdcec` 复核所见，与 `benchmark-protocol.md` 一致） |
| `858635670940153715c263aeec25193cfa42fd91` | 2026-09-20 | `fix: do not answer a question the search has not finished answering` | 该缺口**已关闭**（`preserve-task-operation-stops`，归档 8/8）；`8586356` 及其之后不再含该缺口 |

两者都不是本专项的候选：本专项的候选是**上一个已交付优化之后的剩余成本**，`85828c4`/`8586356`
只作历史锚点，且引用时必须连同第 5 节的后续语义提交一起说明。

## 4. 正确性修正与开放反例（按当前复测与独立归档）

- 第四轮复测（`openspec/completion-review.md`，复核 HEAD `ad0ffa6`）确认 T1–T4 原判据关闭：深链
  完成与清理（`5c35cbf`）、整数比较上下文（`f4d1044`）、局部 boolean 类型（`2895bc4`）、concat
  求值与转换顺序（`5c35cbf`）；树内另有 `66bd2d0`（数组类型拼写）、`5a8c36a`（boolean 上下文）、
  `c5f8185`（消费位置要求的转换）、`138524a`（descriptor 单次解析与数组槽宽）。
- **未确认（保持未确认）**：同一档案的 P1-3 写明"所举具体例子归因不成立，其余条目待核实"，并明确
  "不能据此确认所有 10 条 clone/28 条 toString 历史记录"。这 10/28 条在 G0 是**未确认项**：本专项
  不重测、不据此新增或撤销任何优化结论，也不把它填零。
- **开放的独立项（不归本专项实现）**：同一复测新增 T5 —— `append(int)` 消费 `char` 表达式时仍丢
  数值转换，由正确性修正类 change 负责。本专项基线沿用当前行为，性能结论不得把它的差异读成收益
  或回归。

## 5. 自 `85828c4` 以来的提交归类（"未混入其它 change 的行为差异"）

`git log --oneline 85828c4..HEAD` 共 **75** 个提交（自 `8586356` 起 **72** 个）。按"是否触及
`src/**` 或 `crates/*/src/**` 的 `.rs`"机械分类：**28 个触及产品代码**，其余只改文档、测试、示例或
任务表。触及产品代码者逐条归类（"行为" = 改变可观察结果；"工作量/结构" = 不改变结果但改变做多少
事；"计数/门禁" = 只加观测口或测试）：

| 提交 | 主题 | 归类 |
| --- | --- | --- |
| `8586356` | 停止传播修正 | 行为（缺口关闭，见 §3） |
| `4f62e26` | 区域遍历未完成时返回报告而非中止 | 行为 |
| `445a277` / `fa6dc6e` | 表达式与子表达式分组按位置 | 行为 |
| `5a8c36a` | boolean 上下文呈现 | 行为 |
| `66bd2d0` | 数组类型按 Java 拼写 | 行为 |
| `f4d1044` | 比较操作数上下文先行 | 行为 |
| `2895bc4` | 局部类型只决定一次 | 行为 |
| `5c35cbf` | concat 独立形式 | 行为 |
| `138524a` | descriptor 解析一次、数组一槽 | 行为（+工作量） |
| `93be08c` | 实例方法 receiver 写作 `this` | 行为 |
| `2ad5b9c` | 串行类槽、单一账本与结果权重 | 行为 + 工作量 |
| `c06fdee` | 在调用方已读的类上跑分析管线 | 工作量/结构 |
| `51f8c27` | 单类只准备一次、scope 增量遍历、单一总账 | 工作量/结构 |
| `2c77be3` | 一次有界 bulk 操作 | 工作量/结构（新能力） |
| `642e49f` | 消费者间交接已持有的 facts | 工作量/结构 |
| `93432c2` | 复用 binding 已读的类 | 工作量/结构 |
| `b0e7d23` | 类任务携带容器 | 工作量/结构 |
| `67d9f81` | 查询页可停止扫描 | 工作量/结构（+coverage 语义） |
| `809ca81` | 总账改按维度原子 | 工作量/结构 |
| `570dfc9` | 每类预留 + 共享池 | 工作量/结构 |
| `e83966d` | 请求选择要构造哪些可选证据 | 行为（新能力） |
| `7314ff4` | 只 materialize 被选择的证据并可停止 | 行为 + 工作量 |
| `d7843d1` | artifact binding | 行为（新能力） |
| `db56958` | 按 entry 走 scope，页在后续 entry 前停止 | 行为（coverage 范围） |
| `8e4522e` | D0 计数口与基线 | 计数/门禁 |
| `11f6d91` / `b37255b` / `5f618c6` / `557129b` | 语料、计费表与门禁 | 计数/门禁（测试） |

由此得到三条口径约束（写入 1.4 与后续调查）：

1. 本专项 baseline 是 `ba2076a` 的构建；把任何数字与 `8807fa5` / `85828c4` / `8586356` 比较前，
   必须先按上表对齐行为（尤其 `85828c4` 的停止语义缺口，以及 D1–D5 对 coverage（`db56958`）与
   诊断范围（`7314ff4`）的改变），否则差异会落在正确性修正上而不是性能上。
2. 上表任何"行为"行**不得**被读成本专项的收益或回归；本专项若准入实现，只能出现在独立实施
   change 里，并在其 §13 语义门禁下验收。
3. `clone`/`toString`（§4）与 T5 属未确认/他人负责项：G0 不为它们填零，也不把它们计入任何
   收益上界。
