# 交接：`ba2076a` 轮 benchmark 结果 + 一个待你裁决的协议问题

写给 `/Users/lordcasser/workspace/projects/jarde` 那个会话（`01a0bfba-e6b9-7a72-bb8a-3ceb78010cac`）。
我尝试用 `ask_session` 直接投递三次（一次长、一次中、一次极短），全部返回
`audit_failure: sideband attempt selected Surface ids are not canonical or covered by input refs`
（`retryable: false`）。因此改为文件交接；用户已被告知这一点。

---

## 1. 我做了什么

按 `openspec/benchmark-protocol.md`（2026-09-21 复核版）对 **`ba2076a`** 跑了完整复测：

- 冻结 worktree `/tmp/jarde-bench-v5`，`--release --locked`，独立 `CARGO_TARGET_DIR`；
- `jarde-cli` sha256[:16] `7728e9766c88cc19`；`bulk_scope_sweep` / `demand_workloads` 同批构建；
- 语料 12/12 逐字节与冻结表一致（完整 SHA-256 在 `out5/corpus5.json`）；
- 机器独占：负载中位 4.63/12，峰值 24.56 出现在 8-worker 格子自身（`out5/load.log`）；
- 仓库我只做了 `git worktree add`，未改任何文件。
- 跑完后 HEAD 又进 `b3c65a8`、`235c257` 两个提交，但它们只动 `tests/` 与文档，`src/`、`crates/`、Cargo 文件未变——**我测的引擎与当前 HEAD 的引擎相同**。

## 2. 结果

**批量导出（协议 C/D，240/240 样本 `exit=0`、交付计数逐项一致；每格 10 次交错重复 + 逐配置预热）**

| corpus | sink | w1 | w2 | w4 | w8 | w8/w1 |
| --- | --- | --- | --- | --- | --- | --- |
| bcprov | discard | 3.44 s | 2.58 s | 2.23 s | 1.86 s | 1.85× |
| bcprov | encode | 3.78 s | 2.64 s | 2.29 s | 1.94 s | 1.95× |
| bcprov | write | 4.13 s | 2.69 s | 2.38 s | 2.02 s | 2.05× |
| s2-009 | discard | 11.62 s | 8.92 s | 7.48 s | 6.51 s | 1.79× |
| s2-009 | encode | 14.12 s | 9.35 s | 8.06 s | 7.41 s | 1.90× |
| s2-009 | write | 15.89 s | 10.13 s | 8.90 s | 8.50 s | 1.87× |

每格 min–max 见 `out5/bulk.jsonl`；w8 与 w1 的范围不重叠。

**同轮 jadx 1.5.6**：bcprov 2.72 s、s2-009 6.02 s（默认 6 线程）；`-j 1` 4.71 / 10.46 s；峰值 RSS 1.9–3.9 GB。
**jarde 峰值 RSS**：78–134 MB。
**交付物不同**：jadx 整类源码（bcprov 2,419 文件）；jarde 逐方法记录（bcprov 19,865 条 / 390.8 MB，s2-009 71,582 条 / 3,609.6 MB，含四平面与诊断）。
**CLI `export`**：jobs=auto 2.19 / 9.04 s，jobs=1 3.99 / 16.16 s，`exit=4`＝约定 INCOMPLETE。
**退化（1 MiB 输出额度）**：`exit=4`、写入 71 / 25 行、前缀可读。
**进程级单次**：`recover` 一个方法 11.22 ms / 14.2 MB 对 jadx `--single-class` 925 ms / 448 MB。

## 3. 正确性

600 抽样 → 可比 424 → **一致 421（99.3%）**；剩余 3 个已定位为我的探针把两个 `int` 参数槽接反（`comparePriorities`，三个 war 各一次），引擎文本与字节码一致。**本臂无引擎侧真实语义错误。**
`does_not_compile` 11/600（上一轮 103/1,157；样本来源不同，不作趋势结论）。
**定向复现**：复核里登记为开放的 T5（`append(int)` 消费 `char` 丢转换）在 `ba2076a` **已关闭并验证**——产物 `"" + (int) c + "!"`，贴回编译执行，`'A'`/`'0'` 逐值与原类相同。
**s2-009 的 `partial` 已定位**：298 个方法（28 类）报 `resolution_definition_unbound`，因为这些类**在两个容器里各有一份**（`commons-logging-1.1.1.jar` 与 `commons-logging-api-1.1.jar`；`ArrayUtils` 同时在 beanutils 与 commons-collections）。是设计上的歧义拒绝，不是恢复失败。

## 4. 未测

- page cache 初态未控制（只逐配置预热，故时间列是形状证据，不是稳态收益）；
- A/B 逐方法形状未重跑（其数字属 `8807fa5`，跨形状不比）；
- 零容量 store、超大类倾斜未跑（CLI 未暴露 store 配置）；
- 导出流的证据选择未单独测（CLI `export` 固定 `RecoveryEvidenceRequest::all()`，库侧 `with_evidence` 未跑对照臂）。

## 5. 待裁决

协议第 5–9 行写着"新正式候选须重新固定 SHA、构建配置、目标集合与 T5/CLI 文档预算等边界"。**要不要把正式候选更新为 `ba2076a`，并附上：二进制摘要 `7728e9766c88cc19`、负载区间、未测清单、以及"形状变了"的说明（旧 A/B 数字只作历史锚点）？**

请回：**① 由你写这次协议更新**（我不动仓库），**② 由我写、你 review**。
我倾向 ①：文档与 changes 归档的 owner 在你那边，我这轮只产出报告与原始样本。

如果你的 G0 基线（`tests/p5_optimize_workloads.rs` + `235c257`）在形状或口径上与我这轮冲突，请指出，我按你的口径补测，而不是两边各写一套数字。

## 6. 证据位置（都在本机）

- 报告：`/tmp/jarde-bench/jadx-vs-jarde-ba2076a.md`
- 图表（SVG + index.html）：`/tmp/jarde-bench/figures5/`
- 原始样本：`/tmp/jarde-bench/out5/`（`bulk.jsonl`、`cli_export.jsonl`、`jadx.jsonl`、`degenerate.jsonl`、`classification.json`、`unbound-sample.json`、`targeted.json`、`ondemand.json`、`load.log`、`corpus5.json`）
- 门禁：`/tmp/jarde-bench/correctness-head/`（`jarde_results_*.json`、`jadx_results_*.json`、`sample.json`）
- 生成脚本：`campaign5.py`、`analysis5.py`、`assemble5.py`、`charts5.py`、`gate5.py`、`targeted5.py`、`classification5.py`、`sample_unbound5.py`
