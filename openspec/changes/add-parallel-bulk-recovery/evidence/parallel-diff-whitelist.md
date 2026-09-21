# 1 与 4 worker 的固定差异白名单（任务 4.5）

对象：bcprov（2,430 类 / 15,003 方法），同一次 CLI `export`（tree scope、同一 roots 文档、同一 release 构建），只改 `--jobs 1` 与 `--jobs 4`。

| 比较项 | 结果 |
| --- | --- |
| 方法键集（容器链 + entry raw name + name + descriptor） | 完全相同（15,003） |
| 交付顺序 | 完全相同（逐条比较） |
| 每个方法的正文（sha256 前 16 位，逐方法比较） | **0 处差异** |
| 方法计数、outcome 分桶、`traversal_complete`、aggregate 状态 | 相同 |
| 差异维度 | 只有 `elapsed_millis`（4,291 → 4,220 ms）与 `output_bytes`（375,509,222 → 375,509,233，+11 字节） |

**白名单（新并行资源记录允许的差异）**：`elapsed_millis` 与其直接的字节后果 `output_bytes`。后者不是独立差异：每条记录的 JSON 里带有该次运行的耗时字段，位数变化即字节数变化，因此 JSONL 的字节长度按设计不属于语义指纹。除此之外没有第三项。

**总账核对**：`final.summary.execution.usage` 与流内记录的 `usage` 由同一次操作的总账产生（`tests/bulk_recovery_delivery.rs` 把"返回总账 = 流内总账 + 终态记录自身"钉成等式）；`tests/bulk_recovery_workers.rs` 断言 1 与 N worker 的逐方法 fingerprint/key/outcome/text 与 summary 指纹（忽略 worker 数与 usage）一致；`tests/p1_budget_ledger.rs` 断言累计维度求和、高水位取最大、entry usage 折入且不重置。

**复现**：`jarde-cli export --input <bcprov> --policy plain-jar --release 8 --jobs {1,4} --scope '{"kind":"artifact_tree","root_container":"root"}' --roots @roots.json --output <out>`，然后按上表逐项比较（脚本见本轮会话；比较口径与 `tests/bulk_recovery_workers.rs` 相同）。

**不在白名单内、也不允许被忽略**：scope/身份、content/quality/coverage/execution、source map、规则与诊断、方法顺序。任何一项在 1 与 N worker 间不同都视为失败。
