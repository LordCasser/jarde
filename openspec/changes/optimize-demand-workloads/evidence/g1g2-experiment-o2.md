# 任务 3.3–3.4：O2 读取复用的单因素实验与决定

## 1. 实验的问题与设计

**问题**（O2 的候选，§2 唯一准入项）：一次操作已经为它**选定**的物理定义执行过一次受信读取之后，同一
snapshot、同一 store 上再次选中同一定义时，能不能复用那次读取的字节与它自己建立的 digest，而不重做
`entry` 定位后的字节读取、CRC/长度校验与摘要计算？

**单因素**：两臂是**同一二进制**、同一构建、同一语料、同一 store 容量与同一工作负载，唯一变量是
`JARDE_OPTIMIZE_READ_REUSE=on`（原型补丁 `g1g2-o2-prototype.patch`）。变量未设时原型**不查也不记**
（`definition_read_reuse_enabled()` 在普通构建里根本不存在，在带 `test-support` 的构建里默认 `false`）。

**隔离的其它变量**（design §2/§3.3 要求逐项固定，读数见 §4）：

| 变量 | 如何固定 |
| --- | --- |
| 容器层（O1/O5） | 两臂都挂同一个已交付的容器产品；`container_hits`/`directory_parses`/`nested_materializations` 逐项相同（W1/W2/W5 各档），差异只出现在读取本身的计费维度 |
| CP/Header 层 | 两臂都命中同一 HEADER 产品：W1 第二次请求的 `class_bytes` 两臂都是 **0** |
| 算法/分析 | 同一构建产物、同一 `RecoveryEvidenceRequest::essential()`、同一 stages 表 |
| 批处理/并发 | 两臂都单 worker、都不进 bulk（`w5-sweep` 是 bulk 形状，读数为对照组，见 §4） |
| 预热状态 | 引擎里没有预热/预取入口（§2 o6 已在 `src/**`+`crates/*/src/**` 核对 `prefetch`/`warmup` 零命中）；唯一的「先前状态」是 store 的保留，而它正是本实验的变量 |
| 语料/容量 | fixture（15824 B，blake3 `145f7903…`）与 bcprov-jdk15on-152.jar（2,903,072 B，sha256[:16] `5329ddefb3c92927`）；容量档 `none`/`tiny`/`e1`/`e4`/`roomy` |

**样本与交错**：每个格子 10 个独立进程，同一次重复内 `off`/`on` **交替**（220 个进程，`g1g2-o2-raw-*.jsonl`）。
这满足「时间主张须 ≥10 次交错」的判据；`§2` 的 5 样本不满足该判据的问题在本轮不存在。

## 2. 可复现命令与产物

```text
# 建立原型（两个补丁按序 apply，再单独加观测口）
git apply openspec/changes/optimize-demand-workloads/evidence/g1g2-o2-prototype.patch
git apply openspec/changes/optimize-demand-workloads/evidence/g1g2-o2-harness-observation.patch
cargo test --release --test p5_optimize_workloads --no-run \
    --features "test-support jarde-reader/test-support" --locked     # -> 二进制路径
python3 openspec/changes/optimize-demand-workloads/evidence/g1g2-o2-campaign.py \
    --binary <上一步的二进制> --repeats 10                            # -> g1g2-o2-raw-{fixture,bcprov}.jsonl

# 身份负向实验（两臂分别跑）
JARDE_OPTIMIZE_READ_REUSE=on cargo test --test g1g2_o2_retention \
    --features "test-support jarde-reader/test-support" --locked -- --nocapture
```

产物：`g1g2-o2-campaign.log`（汇总表 + 每个进程的墙钟/RSS）、`g1g2-o2-raw-fixture.jsonl` 120 行（6 格 × 2 臂
× 10）、`g1g2-o2-raw-bcprov.jsonl` 100 行（5 格 × 2 臂 × 10）、`g1g2-o2-prototype.patch`
（sha256[:16] `294d7ba90cd8b2c0`；384 插入/38 删除，含 `facts_cache.rs`/`artifact.rs`/`inspect.rs`/`prepared.rs`/
`providers.rs`/`facade.rs`）、`g1g2-o2-negative-tests.patch`（`2d20bce180d09ad1`）。
机器：`macOS-26.6.2-arm64`，load 起始 `[2.7, 3.4, 4.2]`、结束 `[2.6, 3.1, 3.9]`；无 `/tmp`、无网络。

## 3. 正面读数（中位数，n=10，两臂交替；`off → on`）

**端到端（窗口）与请求段**：窗口层没有可区分的差（W1 bcprov 275,427 → 275,186 µs，即 −0.09%），记
**未证实**；有区分度的是请求/序列段：

| 格子（语料/sample） | `off` | `on` | 配对更快的次数 |
| --- | --- | --- | --- |
| W1 `w1-second-request`（bcprov） | 66 µs | **50 µs** | 9/10 |
| W1 `w1-second-request`（fixture） | 54 µs | 61 µs | 4/10（读取太小，见 §6） |
| W1 `w1-cold-method`（bcprov/fixture，对照） | 1364 / 156 | 1366 / 148 | 5/10、4/10（**不变**：冷路径两臂都读） |
| W2 `control_repeat_micros`（bcprov / fixture） | 20 / 14 | **8 / 7** | 10/10、10/10 |
| W3 `w3-per-method-across-classes`（bcprov / fixture） | 2178 / 4186 | **1764 / 3468** | 10/10、10/10 |
| W3 `w3-batch-per-class-across-classes`（bcprov / fixture） | 598 / 869 | 564 / 796 | 10/10、7/10 |
| W5 `w5-round-trip` roomy（bcprov / fixture） | 498 / 367 | **416 / 290** | 9/10、10/10 |
| W5 `w5-sweep`（bulk 形状，对照） | 3,716,990 / 14,124 | 3,701,157 / 14,214 | 5/10、6/10（**未证实**） |

**准备**：`prepare` 段（调用方发现）两臂相同（W1 冷样本中位数 254,964 µs 对 254,644 µs，≈窗口的 92.6%），
`class_preparations` 在 W3 两臂都不变（逐方法 36/77 对 36/77；类形状 4/8）——读取复用**不**减少准备，
那是 O4 按类形状的工作（见 `g1g2-admission.md` §3.3）。

**工作量（计数，中位数）**：

| 格子 | `off` | `on` |
| --- | --- | --- |
| W1 第二次请求 `class_materializations` / `entry_bytes` / `read_bytes` / `archive_entries`（bcprov） | 1 / 2056 / 1165 / 7 | **0 / 0 / 0 / 0** |
| W1 冷请求（同一字段，对照） | 1 / 2056 / 1165 / 2576 | 1 / 2056 / 1165 / 2576（**不变**） |
| W2 `class_materializations`（fixture / bcprov） | 9 / 9 | **8 / 8** |
| W3 `w3-per-method-across-classes` `class_materializations`（fixture / bcprov） | 77 / 36 | **0 / 0**（该臂窗口内全部命中；读取发生在同进程更早的臂里） |
| W5 往返 `class_materializations`（roomy，fixture / bcprov） | 10 / 10 | **1 / 1**（1 次未命中 + 9 次命中） |
| W5 往返命中/咨询/写入（roomy，bcprov） | 0/0/0 | **9/10/1** |

**驻留**：`retained_bytes` bcprov W1 3,685,071 → 3,687,127（**+2,056 B**，即一次 class 读取）；
W2 3,683,015 → 3,691,931（8 个定义 +8,916 B）；fixture W2 36,047 → 44,507（+8,460 B）；
进程 RSS 中位数 fixture 两臂同为 7.5 MiB、bcprov 26.1 MiB（off）对 25.9 MiB（on）——**没有**可区分的
内存代价，也没有任何 RSS 换算声明。

**容量退化（W5 往返，10 个请求）**：

| 容量 | `off`（中位数 µs / 物化 / 拒绝） | `on` | 读到的行为 |
| --- | --- | --- | --- |
| `none`（0/0）fixture | 466 / 10 / 30 | 470 / 10 / **40** | 读取被拒（+10 次拒绝）后照常读取：结果与计数同基线 |
| `tiny`（1 entry/4 KiB）fixture | 424 / 10 / 19 | **356 / 1 / 19** | 只留 1 个产品：命中的是读取本身 |
| `tiny` bcprov | 24,928 / 10 / 19 | **24,826 / 1 / 0** | 容器层被饿死（`container_hits=0`、`directory_parses=28`）而读取复用照旧生效 |
| `roomy`（1<<14 / 1<<27）两语料 | 498 / 10 / 0；367 / 10 / 0 | **416 / 1 / 0；290 / 1 / 0** | 两者都开：往返 416 µs（bcprov）对无保留形状的 24,928–25,268 µs |

## 4. 隔离后的归因

处理组的差**只**落在读取本身的维度上；其余平面逐项相同（中位数，n=10）：

- **容器层状态相同**：W1 第二次请求 `container_hits` 2 → 2、`directory_parses` 0 → 0、
  `nested_materializations` 0 → 0；W2（bcprov）8 → 8、11 → 11、0 → 0；W5 roomy 20 → 20、8 → 8、0 → 0；
  W5 tiny 0 → 0、28 → 28、0 → 0。即：读取复用**没有**换来额外的目录解析或容器物化。
- **CP/Header 层相同**：W1 第二次请求 `class_bytes` 两臂都是 0（HEADER 命中两臂相同）；W1 冷请求两臂都是
  2056；W3 各臂 `class_bytes` 不变。
- **一处计数确实不同且必须解释**：W3 逐方法臂的 `container_hits` 在 fixture 上从 134 降到 77（bcprov 同臂
  36 → 36 不变）—— 命中路径不再做那次 `entry` 定位，而 OFF 臂每次请求都要定位一次；两臂
  `directory_parses` 都是 0，所以差别是一次**便宜**的命中咨询被跳过，而不是容器工作被删除
  （也就没有把 O1/O5 的收益记到本条上）。
- **`class_materializations` 的口径**：原型让 D0 的计数器只统计**本次请求自己执行**的读取；命中不计数，
  因此上表的 1 → 0 表示「这次请求没有做受信读取」，而不是「工作消失」——W1 `w1-cold-method` 两臂都是 1，
  说明冷路径没有被绕过。

## 5. 负向实验与结果同一性

- **结果同一性（24/24 相同）**：对每个格子、每个 sample 比较**去掉 `usage` 与 `elapsed_millis`** 的结果指纹
  （W1 的 `result_domain`、W2/W3 的 `result_sequence_domain`、W3 手臂的 `arm_domain`（成员解码投影）、
  W5 的 `round_trip_domain`）；24 组比较全部相同，0 组不同。命中确实改变了计费（`entry_bytes` 等），
  但**没有**改变任何被交付的东西。
- **跨 snapshot 不命中（判别性负向，真跑两臂通过）**：两个归档在**同一坐标**上持有**同一份 class 字节**
  （digest 相同），只差一个无关 entry 的字节，因此是两个 snapshot。armed 臂计数
  `(咨询/命中/写入/拒绝/驻留/条目) = (1,0,1,0,1147,0)`（第一个 snapshot 的读取）→ `(2,0,2,0,2294,0)`
  （第二个 snapshot 的**读取仍是未命中**）→ `(3,1,2,0,2294,0)`（回到第一个 snapshot 的定义，命中）；
  两臂结果与直读相同。这条会失败于「只绑坐标 + digest、不绑 snapshot」的实现——镜像 identity 正是本轮要
  钉的规则。
- **容量拒绝回落（真跑通过）**：`FactsCapacity::none()` 下 `retained_bytes=0`、`entries=0`、命中 0、
  两次咨询都在读之前被拒绝，结果与 roomy 臂逐字段相同（去掉 charges）。
- **未成为默认策略（可验证）**：开关默认关闭；**普通构建里连开关都不存在**（`cfg(not(feature = "test-support"))`
  直接 `false`）；带 `test-support` 的构建不设环境变量时同样不查不记。本阶段结束时原型已从工作区还原，
  `git diff` 为空、三个补丁 `git apply --check` 通过。
- **仓库既有门禁**：还原后的工作区 `cargo test --workspace --all-targets --all-features --locked`
  **110 套 / 1592 passed / 0 failed / 18 ignored**（`g1g2-workspace-test.log`）；`cargo fmt --all` 无改动；
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 零告警。

## 6. 噪声、未证实与限制

- **时间层只在小请求形状上可区分**：bcprov 的第二次请求（66 → 50 µs）与 10 次往返（498 → 416 µs）在 9–10/10
  的配对上更快；fixture 的第二次请求（54 → 61 µs）**在离散度内**、`w5-sweep`（−0.4%）也在离散度内。
  窗口层的端到端差（W1 275,427 → 275,186 µs）**未证实**。
- **没有测到的**：(a) 一次受信读取自身的独立耗时与 `s`（O2 的前置缺口仍未关闭）；(b) 命中路径的
  `h` 分量（查找/拷贝/锁）；(c) 紧预算下的完成度差异——命中少计费，理论上会让紧预算请求走得更远，
  本轮两臂都用声明的大预算，**未测**；(d) `retained_bytes` 与 RSS 的换算；(e) 一个 store 服务两个
  snapshot 的**同一 snapshot 身份**（相同字节）情形：本轮只测了不同 snapshot 必须不命中，
  相同内容的两个 snapshot 会共享同一个 snapshot 身份，属未测。
- **口径**：`w5-sweep` 走 bulk 路径，本原型不触及它（这也是对照组的意义）；W3 的臂 (b)–(e) 共用一个
  store 且按顺序运行，命中的是**同一进程更早的臂**已执行的读取，因此计数读作「该臂窗口内没有再做读取」，
  而不是「读取被删除」。

## 7. 任务 3.4 的决定与复核

**决定：实施（有条件）**，由一个独立 change（`reuse-selected-class-read`，登记见
`g1g2-registration.md`）承担。依据是工作量层与契约层，不是时间层：读到的重复工作被删除（W1 第二次 1→0、
W2 9→8、W3 逐方法 77→0、W5 往返 10→1），24/24 结果指纹不变，容量档位给出干净的回落，驻留是每定义一份
class 字节并与既有双界共用；design §3 明确「结构性访问范围修正可以由不必要工作证据准入」。

**条件（写进该 change 的验收，不得省略）**：

1. `facts-cache` 与 `artifact-snapshots` 的 delta 按登记表定义（尤其要改述现文 scenario「随后仍需读取所选
   class」），不把新的 key 维度藏进「缓存透明」。
2. 只允许**工作量**与**契约**主张；不得发布端到端加速或吞吐结论（§3 窗口读数未证实）。紧预算完成度差异
   必须在 G3 的冷/热对照里如实报告。
3. 回退责任逐条演练：无 store、满容量、异声明条目、取消/耗尽、身份不符（登记表 §1）。
4. 不新增依赖、线程、`unsafe`、全局状态或淘汰策略；D0 计数器必须只统计本次请求执行的读取。

**能力/维护/许可复核**：

- 需要的现有能力**已经存在**：`FactsCache` 的 entries/retained_bytes 双界与满则拒绝、`Arc` 句柄共享、
  幂等重写、`FactsIdentity`（format/registry）声明与不匹配丢弃、`ArtifactSnapshot` 的不可变字节与
  `ClassBytesId` 受信摘要、`Budget::facts_cache()` 的唯一附着点。原型没有引入任何新类型族、依赖或平台特性，
  实现只用 `std`（`HashMap`/`Arc`/`Mutex`/`OnceLock`），许可集合不变（无新 crate）。
- **边界缺口（继承既有裁定，不是本项新开）**：解析结构/`PreparedClass` 不能保留（自引用或 `unsafe` 被禁）；
  更高层 store 与淘汰仍暂缓（O5 条件）；`retained_bytes`→RSS 未建立换算。
- **维护成本**：多一个 store 产品 + 一个快照侧入口 + 一处计数器口径；风险集中在 identity（已用判别性负向
  测试钉住）与计数（D0 口径已在原型中改为「本次请求执行」）。

**没有原型成为默认策略**：原型不在仓库里（三个补丁），开关默认关闭且在普通构建中被编译掉，两臂是同一
构建的两种配置；`cargo test --workspace` 在还原后的树上与基线一致（110/1592/0/18）。
