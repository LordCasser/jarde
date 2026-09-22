# §4 G3：实施跟踪、验收与回退（主 Agent 记录）

## 4.1 已交付子项的跟踪

| 子项 | 状态 | 固定候选产物 |
| --- | --- | --- |
| `bound-container-lookup`（O1） | 归档 8/8（`b22ea04`） | 本轮**重跑其门禁**（`p5_container_lookup` 29 passed / 1 ignored），未重复实现 |
| `add-parallel-bulk-recovery`（O2 的类形状、O4、O7 的 W6a、O5/O8 的必要面） | ✓ Complete 29/29 | `openspec/evidence/benchmark-ba2076a/`（正式候选 `ba2076a` 的报告、原始样本、门禁抽样、负载记录、harness 差异） |
| `add-demand-driven-core-results`（O3 的底座、普通 prepared 交接、证据产品） | ✓ Complete 32/32 | `openspec/changes/add-demand-driven-core-results/evidence/measurement-7-3.md` 与 `raw-7-3-*.jsonl` |
| `reuse-selected-class-read`（O2 的准入项） | ✓ Complete 12/12 | 其 `evidence/implementation.md`、`p5_definition_read_reuse` 11 passed、语料门禁 168/187 命中 |
| `make-required-conversions-explicit`（T5） | ✓ Complete 8/8 | 其 verification（执行对照） |

未完成项保持待办、不被本专项勾选：`add-parallel-bulk-recovery` 的 6.3 已关闭，其余 change 无开放任务。

## 4.2 足额跨路径语义对照与紧预算/取消

| 实验 | 结果 | 证据 |
| --- | --- | --- |
| 关闭/启用两臂逐方法 fingerprint（去 `usage`/`elapsed_millis`/`limits`） | **6/6 相同**；12 个非读取维度逐步相同；D0 的准备/解码/呈现/记录计数逐步相同 | `reuse-selected-class-read/evidence/implementation.md` |
| 紧预算冷臂（无 store、`entry_bytes=size-1`） | 停在 `EntryBytes`，如实 | 同上 |
| 紧预算热臂（同预算、命中） | 完成，发布物与宽松臂相同，三个读取维度全 0；**完成有账可查** | 同上 |
| 命中后被 `ResultItems` 停下的请求 | 报 `Partial{BudgetExceeded{ResultItems}}`、0 item，不伪造 Complete | 同上 |
| 同一预算的第二次请求 | 在 `ClassHeaders` 处被拒，usage 不变（不借用旧额度） | 同上 |
| 取消 | facade 与入口自身都以 `cancelled` 拒绝，不计命中（反例：命中不 `poll` → 被取消的请求被服务） | 同上 |
| 三类终止点 | cancelled / `EntryBytes` 限额 / 流中途损坏 / CRC 不符各一个 store，**保留集合都停 0**；同几何的完整读取保留 1 条 191 B（非空对照） | 同上 |
| 顺序/背压/取消（调度面） | 由 bulk 的 `bulk_recovery_{serial,workers,backpressure,cancel,delivery,retention,handover}` 38 项承担 | `add-parallel-bulk-recovery` |

**没有为收益删除 origin/顺序/语义 diagnostics/coverage/规则/source map**：`domain` 指纹与语义平面在两臂与 1/N 对照中逐项相同（bulk 的差异白名单只有 `elapsed_millis` 与其字节后果）。

## 4.3 回退演练

| 情形 | 演练结果 |
| --- | --- |
| 无 store | 直读；读与直读逐维相同 |
| 容量拒绝（`none`） | 16 次拒绝 / 0 命中 / 0 保留；结果与直读相同 |
| 身份不符（异 snapshot、异 digest、异 length、异 variant） | 错误码与无 store 臂**逐字相同**，`hits` 不动 |
| 取消 | 入口与 facade 各自拒绝，不计命中 |
| 异声明条目（`over(format+1)`） | discard 计数增加、`hits` 不动、结果同直读 |
| 发布回退 | 语义回归时撤回 `reuse-selected-class-read` 的独立提交（无开关、无旁路需要清理） |
| 串行降级与共享生产者失败 | 不适用：本层无共享生产者与后台预热；串行/并行由 bulk 的既有门禁覆盖 |

## 4.4 后准入回归（在 `reuse-selected-class-read` 之后重跑 W1–W5 与 W6a）

命令（`run-baseline.py`，每样本独立进程、10 次交错、failure 记数）：

```
python3 evidence/run-baseline.py --tag fixture-postadmission --repeats 10 --features test-support \
  --out postadmission-fixture-raw.jsonl <20 个配置，见 baseline-fixture-summary.txt>
python3 evidence/run-baseline.py --tag bcprov-postadmission --repeats 10 --features test-support \
  --out postadmission-bcprov-raw.jsonl <18 个配置，取自 baseline-bcprov-raw.jsonl 的 config 列>
```

结果：**fixture 200 样本、bcprov 180 样本，failures 均为 0**（`postadmission-*.log`）。

前后对照（`compare-campaigns.py`，按 **(configuration, sample)** 分组——harness 在本轮之间新增了 6 个样本，按配置整组比对会把 harness 自身的增长记到引擎头上，故不那样做）：

| 语料 | 读维度移动的测量 | **工作维度移动的测量** | 两侧样本不同的测量（harness 增长） |
| --- | --- | --- | --- |
| fixture | 1（`w1-second-request`：`archive_entries`/`read_bytes`/`entry_bytes` 下降） | **0** | 6 |
| bcprov | 1（同上） | **0** | 6 |

即：被准入的组合进入后，**除了"被停止重复的那次读取"以外没有任何计数移动**——这正是该变更 allowed 的主张面；冷/热、一次性扫描、容量退化（`tiny`）、首屏与总量（W4/W5）、W6a 的三档 sink 与四种 worker 数全部读出相同的工作维度。

**显式启用决定**：`reuse-selected-class-read` **保持启用**（它就是默认行为，无开关）。依据是上一层三条：契约等价、工作量下降、回退逐条演练通过；**时间层未证实**（本轮不是 ≥10 次交错的计时对照，窗口份额上界仍是 0.5%），因此不附任何加速主张。

**未测（不得读作已测）**：尾延迟（10 样本不构成 p95）、`retained_bytes`→RSS 换算（同档 RSS 离散度远大于驻留差且不单调）、W6b（无需求）、O3 的 item 级停止（子 spec 未立）。
