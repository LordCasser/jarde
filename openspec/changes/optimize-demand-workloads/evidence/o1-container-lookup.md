# O1（任务 2.1）：容器定向访问、目录/backing 复用与跨请求保留的归因

本条不重复实现 O1。实现与行为规格由已归档的
[bound-container-lookup](../archive/2026-09-20-bound-container-lookup/verification.md) 唯一拥有（8/8，`b22ea04`）。
本轮的工作是：在**冻结基线 `ba2076a`** 上复核它的计数证据，把 B/D/C/W/F 五条路径在 W1/W2/W5 上归因，
并确认 sibling/深度/重复名/容量场景齐全。

## 1. 调查的问题与假设

**问题**：指定 container 的定向查询在当前基线上真的不展开未搜索 sibling、并在事实仍被保留时不重建目录吗；
在 W1/W2/W5 里，哪一份读数属于「定向访问（D）」「冷 store 读（C）」「命中（W）」和「容量不足（F）」？

## 2. 测量形状（引用与新测分开）

| 来源 | 形状 | 样本 | 本轮状态 |
| --- | --- | --- | --- |
| 归档 change 的门禁 | `tests/p5_container_lookup.rs`（26 容器/24 sibling 的 WAR、两层嵌套链、flat JAR 4/96 sibling、重复 `p/S.class`、容量不足行） | 29 passed / 1 ignored | **本轮重跑**（见 §6） |
| G0 冻结基线 | `evidence/g0-{workloads,metrics}.md` + `baseline-*-raw.jsonl`，W1/W2/W5 各 10 个独立进程 | 20/18 配置 × 10 | **引用**，不重测 |
| 本轮新测 | `investigation-fixture-raw.jsonl`、`investigation-bcprov-raw.jsonl`（各 5 个独立进程/配置，`--features test-support`，release），叠加原有 W1/W2/W5 与新增容量档 `none/tiny/e1/e4/roomy` | 75 / 65 行 | **新测**，表见 `investigation-*-tables.txt` |

语料：仓库内 fixture（15824 B，blake3 `145f7903…`）与 bcprov-jdk15on-152.jar（2,903,072 B，sha256[:16]
`5329ddefb3c92927`）。机器 load average 记录在 JSONL 每行上（fixture `[4.1, 5.2, 5.0]`，bcprov 同级）。

## 3. 必要工作证据（p/s/h 的计数面）

数字取自 `investigation-*-tables.txt` 的 `cache` 段（中位数，n=5）。口径：**store 的报告是「到该样本为止」的累计读数**，
所以同进程相邻样本之差才是那一次请求自己的读数；跨配置比较取同一位置的读数。

| 路径 | 读数（W1 冷单方法，fixture / bcprov） | 归因 |
| --- | --- | --- |
| C（冷 + roomy store） | `directory_parses` 21 / 11，`container_consultations` 24 / 2，`hits` 1 / 1，`nested_materializations` 32 / **0**，`retained_bytes` 17935 / 3685071 B | 定向路径只解析它真的需要的那几个容器；bcprov 是 flat jar，所以嵌套物化恒为 0 |
| W（同 snapshot 第二次同请求） | 两次读数之差：`consultations` +2 / +2，`hits` +2 / +2 | 第二次请求的容器事实全部由保留回答；`directory_parses` 不增长 |
| D（定向 vs 整树参照） | 归档门禁：整树 26 容器/24 sibling = 26 次解析 + 25 次嵌套物化（7866 B）；定向 = 2 次解析 + 2 次物化（728 B） | 未搜索 sibling 的物化为零；本轮重跑打印同上 |

W2（同 snapshot、连续 8 个类 + 1 次控制重问）：`container_consultations` 35、`hits` 11、`misses` 24、
`containers` 2、`retained_bytes` 36047 B（fixture）。**每个新类的首问是一条 miss，控制重问是 hit**——
这就是「目录与 backing 在保留期间不重建」的可观察形式。

W5（sweep + 10 次往返，同一个 store）的四档对照，bcprov：

| capacity | `container_hits` | `container_misses` | `directory_parses` | `refused_capacity` | round-trip `sequence_micros` |
| --- | --- | --- | --- | --- | --- |
| roomy | 17538 | 1 | 11 | 0 | 469 |
| e4 | 17538 | 1 | 11 | 0 | 525 |
| tiny（1 entry/4 KiB） | 0 | 17539 | 31 | 19（+2 B） | **24971** |
| none（0/0） | 0 | 17539 | 31 | 31 | **25268** |

- **F 的代价是「重复解析目录」而不是「整次扫描更慢」**：四档的 sweep 段几乎相同（3.70–3.72 s，区间互相覆盖），
  差别只出现在复用上（往返 0.47 ms 对 25.3 ms，≈54×）。这与 G0 的结论一致，本轮在 `none` 档上把下界补齐。
- `e1`（1 entry / `1<<27` B 字节界）在 fixture 上是**部分**复用：`hits` 208/412、`refused_capacity` 11——容量按 entry 数先满，
  后续插入被拒而不驱逐（见 O5 §5）。

**缺哪个计数**：`FactsReport` 只按「读到的记录」计账，没有「按 raw name 命中而不物化」的独立计数；
`container_hits` 与 `nested_materializations` 的差是当前能给出的最接近的口径。W1/W2 在冻结配置里固定 `roomy`，
所以 **F 在 W1/W2 上没有被这组工作负载覆盖**（见 §8）。

## 4. 理想上界

- **D 对整树的删除量**：归档 fixture 上每次定向查名从 26 次目录解析 + 25 次物化（7866 B）降到 2 + 2（728 B），
  即 `nested_materialized_bytes` 减少 7138 B/次；这个差是**结构性删除**，不依赖耗时。
- **W 对 C 的删除量**：bcprov 往返的 `directory_parses` 11（roomy）对 31（tiny），即保留期每小时省 20 次目录解析；
  往返总时长 469 µs 对 25,268 µs，**在 W5 的已声明形状上**保留把这段降到 1.9%，但这条只适用于「同 store 重复访问」，
  不能外推为端到端比例（sweep 段不含它）。
- 拿不到的部分：`p`（该部分占某个端到端目标的比例）在 W1/W2 上**没有可用的分母**——请求段是 137 µs/1.415 ms，
  而 prepare 是 813 µs/256.3 ms 的调用方发现成本，两者不同源，不能相除。

## 5. 资源与语义代价

- 驻留：bcprov 保留 1 个容器（3,685,071 B 的 backing 权重）+ 1 个 class 事实；W4 整页查询保留 2430 个条目、
  8,687,490 B。RSS 中位数 26.1 MiB（W1）到 128.1 MiB（W5 roomy），与 G0 的 122–160 MiB 同量级。
- 契约风险（引用归档裁定，不在本轮重开）：STORED 走 owned backing 而不是借用区间；目录/校验 schema 是编译期常量，
  所以「schema 变化导致不命中」只能靠重新构建触发；前缀语义不在该 change 内。
- 不变量：未搜索 sibling 的物化为零、保留期间不重建目录/父 backing、重复 raw name 不合并、
  cold 完整目录与 CRC 检查保留——四条都由归档门禁持有并在本轮重跑通过（§6）。

## 6. 负向实验与结果

本轮**真跑**（`cargo test --test p5_container_lookup --locked --features test-support -- --nocapture`，29 passed / 0 failed / 1 ignored），
其中三条正是反向场景，打印原文：

```text
local lookup: directories parsed 2, nested materialized 2 (728 bytes), sibling libraries in the fixture 24; the second materialization is the read's own fallback, because a zero-capacity store kept nothing
local lookup complete over a damaged sibling; whole-tree enumeration: partial with 1 diagnostic(s): ["entry_integrity"]
flat JAR with 96 sibling archives: directories parsed 1, nested materialized 0, archive_entries 194
capacity-starved row: complete, same semantic fingerprint, retentions 0 refusals 10 directories parsed 8
```

- **负向 1（未搜索 sibling 损坏不误伤）**：目标之外的 sibling 被破坏后，定向查询仍 `Resolved`（2/2），
  而显式整树枚举如实报 `partial` + `entry_integrity`。即「定向」没有把兄弟的损坏当成自己的结论。
- **负向 2（容量不足不改变结果）**：`capacity-starved row` 与同初态的 roomy 行给出**同一个** semantic fingerprint，
  只多了 `refusals 10` 与 `directories parsed 8`；内容变化（`a_changed_content_chain_or_declaration_misses`）
  与伪造 origin（`a_forged_container_origin_is_refused`）各自失败于自己的门禁。
- **负向 3（sibling 规模）**：4 sibling → 96 sibling，`directory_parses` 都是 1、`nested_materialized` 都是 0，
  唯一变化是 `archive_entries` 10 → 194（目录记录本身）。规模不带来额外物化。

场景齐全性：**sibling**（4/96/24 档）、**深度**（两层嵌套链 `a_two_level_chain_reaches_every_ancestor_once`）、
**重复名**（`duplicate_physical_entries_keep_their_ordinals_direct_and_warm`：2 候选、ordinal [0,1]、
暖路径 +0/+0）、**容量**（上一节的 `entries/bytes` 双界与拒绝）四类都在，无缺口。

## 7. 处置建议（建议即可，不做准入结论）

- **D/C/W 三条：维持已交付，不新立实现**。它们的工作计数由归档 change 的门禁持有，本轮复核无新缺口。
- **F：维持「满则拒绝」的现策略**；等 §3 的证据（e1 只保留 1 个 entry 时命中 208/412）被更高层缓存需求
  真正要求时再谈淘汰（那是 O5 的处置）。
- **B（历史整树原始路径）：不可归因**。`providers.rs::tree_candidates` 在当前源码中已不存在
  （G0 §2 的 `git log -S` 记录），旧数字对应的是已删除的入口；**不引用旧数字**，也不为它重建路径。
- 该条**不暂缓**任何东西：O1 无待实施项。

## 8. 未确认项

- **F 在 W1/W2 上没有档位**：冻结的 W1/W2 固定 `Capacity::Roomy`，本轮没有为了归因去改这两条工作负载
  （改它们会移动 G0 基线读数）。F 的归因取 W5 的四档与归档的零容量探针，W1/W2 的 F 记**未测**。
- **未测进程 RSS 与 `retained_bytes` 的换算**：归档 change 明确没有 RSS 声明；本轮仍未做，
  只有 `retained_bytes`（权重代理）与进程 RSS 两个平面分列。
- 目录解析/物化计数是**工作计数**，不是耗时；本文件的任何「省了 X」都不读成墙钟收益。
