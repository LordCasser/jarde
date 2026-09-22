# jarde `d16419c` vs jadx 1.5.6 —— 批量导出轮（含读取复用验证）

测量日期 2026-09-22 · 语料 `/Users/lordcasser/workspace/vulnerability/vulhub`（12/12 逐字节校验，`corpus5.json`） ·
口径 `openspec/benchmark-protocol.md` · 原始样本同目录

> **候选**：`d16419c`（独立 worktree `/tmp/jarde-bench-v6`，`--release --locked`，独立 `CARGO_TARGET_DIR`），
> 相对上一冻结候选 `ba2076a` 的引擎差异是 `15df69a`（定义读取复用，7 文件 +515/−55）。
> `jarde-cli` sha256[:16] `42fd43334f82e104`。机器独占，负载见 `load.log`；page cache 未控制（逐配置预热），
> 时间列是**形状证据**。

## 0. 本轮引擎变化：读取复用（独立复现）

我用自己的探针（一个进程内、同一 snapshot、同一 facts store、两次恢复同一个定义）核对，两臂都是
各自的库、各自的 target：

| corpus | 候选 | 第二次读同一定义（`archive_entries` / `read_bytes` / `entry_bytes` / `class_bytes`） | 归一化文档 blake3 |
| --- | --- | --- | --- |
| bcprov | `ba2076a` | 7 / **1165** / **2056** / 0 | `e0d57178…` |
| bcprov | `d16419c` | **0 / 0 / 0 / 0** | `e0d57178…`（相同） |
| S2-009 | `ba2076a` | 413 / **1263** / **2659** / 0 | `423fa00f…` |
| S2-009 | `d16419c` | **0 / 0 / 0 / 0** | `423fa00f…`（相同） |

（归一化 = 序列化后递归删除 `usage`/`elapsed_millis`/`limits`，与变更自己的对照口径一致。）
**结论：第二次读取的工作量降到 0，而文档不变。** 三个臂（essential / 第二次 / all）在两候选上
归一化哈希逐一相同，`method_bodies` 也都为 1——省掉的是重复读取，不是产出。

端到端（`demand_workloads`，5 个 workload × 2 语料 × 10 次交错，进程级）：所有格的
`total_micros` 比值落在 0.97–1.00（下表为逐语料/workload 的中位比），与变更自述的窗口份额上界
**0.5%** 量级一致；单次遍历的 `sweep` 不变（bcprov 0.997×、S2-009 1.001×）。

| corpus | workload | ba2076a 中位 µs | d16419c 中位 µs | 比 |
| --- | --- | --- | --- | --- |
| bcprov | expand | 1354 | 1336 | 0.987× |
| bcprov | nav-class | 1276 | 1260 | 0.987× |
| bcprov | nav-decl | 1312 | 1314 | 1.002× |
| bcprov | recover-one | 1347 | 1341 | 0.996× |
| bcprov | sweep | 789996 | 787891 | 0.997× |
| s2-009 | expand | 1057 | 1021 | 0.966× |
| s2-009 | nav-class | 585 | 575 | 0.983× |
| s2-009 | nav-decl | 646 | 630 | 0.974× |
| s2-009 | recover-one | 828 | 824 | 0.995× |
| s2-009 | sweep | 6219700 | 6227560 | 1.001× |

## 1. 批量导出（协议 C/D，24 格 × 10 次交错）

| corpus | sink | workers | runs | wall median s | wall min | wall max | RSS median MiB | bodies (median) | declared | exit codes | counts stable |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bcprov | discard | 1 | 10 | 3.423 | 3.385 | 3.513 | 73.5 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | discard | 2 | 10 | 2.603 | 2.551 | 2.737 | 86.9 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | discard | 4 | 10 | 2.246 | 2.204 | 2.280 | 88.1 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | discard | 8 | 10 | 1.855 | 1.843 | 1.866 | 123.2 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | encode | 1 | 10 | 3.737 | 3.684 | 3.777 | 80.2 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | encode | 2 | 10 | 2.676 | 2.623 | 2.795 | 91.1 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | encode | 4 | 10 | 2.318 | 2.269 | 2.371 | 93.3 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | encode | 8 | 10 | 1.940 | 1.925 | 1.971 | 128.7 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | write | 1 | 10 | 4.045 | 4.019 | 4.083 | 80.3 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | write | 2 | 10 | 2.741 | 2.676 | 2.943 | 94.2 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | write | 4 | 10 | 2.421 | 2.354 | 2.745 | 92.6 | 14495 | 15003 | 0×10 | 10/10 |
| bcprov | write | 8 | 10 | 2.022 | 2.006 | 2.423 | 132.3 | 14495 | 15003 | 0×10 | 10/10 |
| s2-009 | discard | 1 | 10 | 11.584 | 11.447 | 12.070 | 91.6 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | discard | 2 | 10 | 9.024 | 8.818 | 9.359 | 94.0 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | discard | 4 | 10 | 7.511 | 7.406 | 7.784 | 98.0 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | discard | 8 | 10 | 6.477 | 6.444 | 6.600 | 108.1 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | encode | 1 | 10 | 14.178 | 14.080 | 14.616 | 90.7 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | encode | 2 | 10 | 9.438 | 9.245 | 9.685 | 93.3 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | encode | 4 | 10 | 8.140 | 7.949 | 8.282 | 99.7 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | encode | 8 | 10 | 7.429 | 7.299 | 7.553 | 105.5 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | write | 1 | 10 | 15.827 | 15.090 | 20.467 | 88.8 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | write | 2 | 10 | 10.721 | 10.009 | 11.944 | 92.7 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | write | 4 | 10 | 9.223 | 8.680 | 10.152 | 97.8 | 53247 | 57180 | 0×10 | 10/10 |
| s2-009 | write | 8 | 10 | 8.628 | 8.215 | 9.991 | 106.4 | 53247 | 57180 | 0×10 | 10/10 |

## 2. 并行度

| corpus | sink | w1 | w2 | w4 | w8 | w8/w1 | w8 vs w1 (min–max overlap) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bcprov | discard | 3.42 | 2.60 | 2.25 | 1.85 | 1.85x | disjoint |
| bcprov | encode | 3.74 | 2.68 | 2.32 | 1.94 | 1.93x | disjoint |
| bcprov | write | 4.04 | 2.74 | 2.42 | 2.02 | 2.00x | disjoint |
| s2-009 | discard | 11.58 | 9.02 | 7.51 | 6.48 | 1.79x | disjoint |
| s2-009 | encode | 14.18 | 9.44 | 8.14 | 7.43 | 1.91x | disjoint |
| s2-009 | write | 15.83 | 10.72 | 9.22 | 8.63 | 1.83x | disjoint |

## 3. CLI 导出（产品形态）

| corpus | jobs | runs | wall median s | wall min | wall max | RSS median MiB | JSONL lines (median) | exit codes | effective jobs | status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bcprov | 1 | 10 | 3.945 | 3.885 | 4.291 | 82.9 | 19865 | 4×10 |  |  |
| bcprov | auto | 10 | 2.182 | 2.154 | 2.349 | 136.6 | 19865 | 4×10 |  |  |
| s2-009 | 1 | 10 | 15.973 | 15.569 | 16.782 | 93.6 | 71582 | 4×10 |  |  |
| s2-009 | auto | 10 | 8.664 | 8.526 | 9.821 | 120.9 | 71582 | 4×10 |  |  |

## 4. 退化形状

| corpus | runs | exit codes | lines written (median) | wall median s |
| --- | --- | --- | --- | --- |
| bcprov | 3 | 4×3 | 71 | 0.02 |
| s2-009 | 3 | 4×3 | 25 | 0.02 |

## 5. jadx 1.5.6（同轮同机，RSS 只取 product 配置）

| artifact | default median s | default min–max | `-j 1` median s | RSS median MiB | files | exit codes |
| --- | --- | --- | --- | --- | --- | --- |
| hello | 0.41 | 0.40–0.41 | 0.40 | 172.9 | 0 | 2×5 |
| evil | 0.94 | 0.94–1.13 | 0.95 | 193.4 | 1 | 0×5 |
| weblogic_decrypt | 0.95 | 0.94–0.95 | 0.96 | 194.6 | 8 | 0×5 |
| s2-001 | 2.64 | 2.63–2.70 | 4.14 | 1579.0 | 1525 | 0×5 |
| s2-005 | 3.25 | 3.23–3.25 | 5.76 | 2406.4 | 2101 | 0×5 |
| bcprov | 2.70 | 2.69–3.26 | 4.72 | 1811.1 | 2419 | 3×5 |
| s2-012 | 3.29 | 3.24–3.29 | 5.30 | 2562.8 | 2308 | 0×5 |
| s2-007 | 3.30 | 3.29–3.30 | 5.54 | 2186.5 | 2308 | 0×5 |
| s2-015 | 3.32 | 3.29–3.34 | 5.33 | 2417.8 | 2386 | 0×5 |
| s2-008 | 3.30 | 3.28–3.36 | 5.61 | 2201.7 | 2309 | 0×5 |
| s2-013 | 3.26 | 3.26–3.38 | 5.84 | 2487.8 | 2308 | 0×5 |
| s2-009 | 5.67 | 5.52–5.98 | 10.52 | 3795.3 | 7163 | 3×5 |

## 6. 交付构成

| corpus | exit | stream bytes | records | classes prepared | declared | delivered | produced | explanation only | not produced | no body | refused | oversized | execution |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bcprov | 4 | 390.8 MB | 19865 | 2430 | 15003 | 15003 | 12045 | 2447 | 3 | 508 | 0 | 0 | partial (ir_frame_inconsistent) |
| s2-009 | 4 | 3609.6 MB | 71582 | 7200 | 57180 | 57180 | 44502 | 8725 | 318 | 3635 | 0 | 0 | partial (resolution_definition_unbound) |

## 7. 正确性门禁与定向复现

| 工具与动作 | min | 中位 | max | 峰值 RSS | n |
| --- | --- | --- | --- | --- | --- |
| jarde `recover` 一个方法体（bcprov 的 `org/bouncycastle/asn1/ASN1Object`） | 10.37 ms | 10.49 ms | 20.44 ms | 14.2 MB | 10 |
| jadx `--single-class` 同一个类 | 840.99 ms | 856.67 ms | 870.07 ms | 455.7 MB | 10 |

| fixture | 方法 | content / quality | 产物正文 | 贴回后编译 | 行为一致 |
| --- | --- | --- | --- | --- | --- |
| Shapes | arr | contains_statements / fallback | `int i = 0;` | 不通过 | - |
| Shapes | r15 | contains_statements / fallback | `if (count >= threshold) { }` | 不通过 | - |
| Shapes | r8 | contains_statements / fallback | `x = x + 1;` | 不通过 | - |
| T5 | prefixed | contains_statements / structured | `return "x" + (int) c;` | 通过 | 一致 |
| T5 | value | contains_statements / structured | `return "" + (int) c + "!";` | 通过 | 一致 |

（门禁明细：600 抽样 → 可比 424 → 一致 421（99.3%）；3 个不一致是探针自身的 `int` 参数槽接反；
`does_not_compile` 11/600。jadx 臂可比 36、一致 33，3 个不一致已定位为重载绑定问题，不作工具归因。）

## 8. 与上一轮（`ba2076a`）的同格差

24 个批量格的中位比落在 **0.98–1.06×**（大多在 1.00 附近）：批量导出每个类只读一次，
复用变更在这条形状上没有可测收益——与它自己"只作工作量主张、不作加速结论"一致。

## 9. 未测

- page cache 初态未控制（只逐配置预热）；A/B 逐方法形状未重跑（属 `8807fa5`）；
- 零容量 store 与超大类倾斜未跑（CLI 未暴露 store 配置）；导出流的证据选择未作对照臂。

## 10. 发布

对外稿见 [`docs/benchmark.md`](../../../docs/benchmark.md) 与 `docs/benchmark_all.png`（README 的
`## Benchmark` 段落指向它们）。
