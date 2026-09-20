# 重新 benchmark 协议（2026-09-20 复核后）

本文件登记下一次 jadx-vs-jarde 测量的参数与判据，**测量尚未完成**。上一次测量（`cd6f2f0`，历史报告路径 `/tmp/jarde-bench/jadx-vs-jarde-benchmark.md`，原始数据已丢失）的脚本未完整保留；本文件也尚未固定方法抽样清单、全部运行命令与 harness，因此不能单凭协议声称可复现或 G0 已完成。正式运行前须保存这些资产及摘要，不依赖 `/tmp` 历史文件。

## 冻结基线

- **`85828c4` 改记为历史比较臂**：它是本次 review 的行为基线，仍存在 [R1/R2 停止语义缺口](completion-review.md)，不能称为“最终已验收引擎”。修正后的当前候选 SHA 为 **`8586356`**（`preserve-task-operation-stops` 的固定提交：名称选择的停止传播与类视图顶层汇总），其本机门禁与反例对照见该 change 的归档验证记录。
- 正式测量在每个声明 SHA 的独立 worktree 构建，给它独立的 `CARGO_TARGET_DIR`，记录构建参数与二进制摘要。历史臂可用 `git worktree add /tmp/jarde-bench-base 85828c4`；当前候选必须使用实际验收 SHA。两侧行为差异单列，不能归因成性能收益。
- **每份记录同时登记该引擎版本的诊断 code 词汇**：诊断码 SHALL 标识「该引擎版本在本次运行中记录了哪个事实」，MUST NOT 被读作跨版本可比的量。同一个码在不同版本的含义变化（例如声明事实交接前 `jre_declaration_class_not_in_run` 在单个 artifact 上出现 10,720 次、交接后语料全局 0 次，而 `jre_declaration` 出现在每个 artifact 上）MUST 被登记为词汇/事实来源的变化；消费方、比较报告与文档 MUST NOT 把某个码的出现次数下降直接读作质量或覆盖改善，比较 MUST 同时声明两个引擎版本与各自的 code 词汇。
- 每个 change 的实现提交与其 CI 结果：

| 批次 | Change | 实现提交 | CI |
| --- | --- | --- | --- |
| 正确性 | close-recovery-correctness-gaps | `fd0aae8` | 35490735745 success |
| benchmark 1 | expose-recovery-content | `7d095ce` | 35494183220 success |
| benchmark 2 | carry-declaring-class-evidence | `618de49` | 35495275106 success |
| benchmark 3 | bound-container-lookup | `b22ea04` | 35496369587 success |
| benchmark 4 | bind-prefixed-load-roots | `fbcf06b` | 35497694448 success |
| 易用性 1 | add-artifact-navigation | `e0c83b6` | 35499237358 success |
| 易用性 2 | add-task-oriented-operations | `2428752` | 35501294803 success |
| 易用性 3 | add-task-oriented-cli | `85828c4` | 35502305001 success（归档验证记录，本轮未重查远端） |
| 停止传播修正 | preserve-task-operation-stops | `8586356` | 见该 change 的归档验证记录 |

## 语料与每次计时前的门禁

12 个 artifact（vulhub checkout，只读）。**每次计时前校验**：若字节与下表不符，冻结记录必须显式说明语料已变，而不能沿用旧比较。

| artifact | 类型 | 字节 | sha256[:16] 期望 |
| --- | --- | --- | --- |
| nacos/CVE-2021-29442/evil.jar | jar | 2854 | `9217f55601a388e5` |
| weblogic/weak_password/decrypt/weblogic_decrypt.jar | jar | 19309 | `bb5357d7f0dade90` |
| weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar | jar | 2903072 | `5329ddefb3c92927` |
| weblogic/weak_password/web/hello.war | war | 1927 | `b47d85c9b6ec0e2c` |
| struts2/s2-001/S2-001.war | war | 3296352 | `c2e74165ae8b645b` |
| struts2/s2-005/S2-005.war | war | 3272437 | `1c92a38a3618a44b` |
| struts2/s2-007/S2-007.war | war | 3384417 | `9b9b5e37ea721ed9` |
| struts2/s2-008/S2-008.war | war | 3384218 | `fa139b2d545b6ba3` |
| struts2/s2-009/S2-009.war | war | 12272179 | `dda30ca7a2587391` |
| struts2/s2-012/S2-012.war | war | 3383421 | `73d40e42f7866332` |
| struts2/s2-013/S2-013.war | war | 3383341 | `c8f5c7e5c4140392` |
| struts2/s2-015/S2-015.war | war | 3510089 | `23274e700381f13f` |

旧运行的 sha256 只保留前 16 位，只能核对历史摘要前缀一致，不能恢复旧完整摘要。新 campaign 必须计算并保存完整 SHA-256、精确 artifact/方法清单、抽样与排除规则；前缀相同不能代替新样本的完整身份记录。

## 工具

以下是旧运行的工具/启动行为记录；新运行须实际记录路径、版本、JVM 与线程配置，不假定当前环境相同。

- jadx **1.5.6**（`/opt/homebrew/bin/jadx`），JVM OpenJDK 23.0.1。
  - **计时臂**：默认（6 线程）与 `-j 1`。
  - **join 臂**（只为对账类/方法集合，不计时）：`--no-res --output-format json --no-inline-methods --no-inline-anonymous --no-move-inner-classes`。
  - jadx 1.5.6 启动期会拉起 `localhost:8085` 的 MCP server：**计时期间任何时刻只允许一个 jadx 进程**。
- jarde：冻结 worktree，`--release` 构建。

## jarde 测量形状（三档）

1. **按需单请求**（进程级）：记录 `jarde-cli` 对单方法/entry 的 wall time，以及 jadx `--single-class` 的类级 wall time。两者交付单位不同，必须分列，不直接宣称同口径加速；要作同口径对照，须另行固定相同类/方法集合与输出工作。
2. **逐 artifact sweep**（主表）：**每个 artifact 一个 snapshot**，**每个请求一个新 `Budget`**；容器声明见下。
3. **可选全量**：只在 directed access 使 WAR 全量变得可行时做，作为**同一次 campaign 内的第二个覆盖臂**（旧抽样规则作可比锚点 + 全量作新信息），不做两次独立运行。

## 环境声明是承重件（必须记录并哈希）

前缀 load root 现在由调用方**显式声明**，引擎不推断。因此每个 artifact 的完整声明必须逐项记录并哈希，否则 `archive_entries`/`read_bytes` 的下降无法归因到定向访问——可能只是声明变了：

- 每个 loader 的 roots，逐条：`container` origin（snapshot + container chain）与 raw `prefix`（普通 JAR 用空 prefix；WAR 的 `WEB-INF/classes/` 必须显式写）；
- `RuntimeProfile`：`java_release = 8`、`MultiReleasePolicy::Disabled`、`LayoutMode::Generic`；
- `LoadDomain`：loader 名、`parent_loader`、delegation、roots 顺序、module mode；
- 是否挂 `FactsCache`（默认 off；如启用，记录容量配置与预热步骤）。

## 记录列（固定）

**每请求分布**（p50/p90/max，逐 artifact）：`archive_entries`、`entry_bytes`、`read_bytes`、`class_bytes`、`class_headers`、`method_bodies`、`analysis_steps`、`result_items`，以及 wall µs。

**新增列**（本次首次启用，旧数字不在同一口径）：

- `RecoveryReport.content`：`not_produced` / `explanation_only` / `contains_statements`——**主分类器**，结构性事实；
- 容器计数：目录解析次数、nested 物化次数与字节、retained weight 与容量拒绝（cache 启用时）；
- 旧的 token/调用序列/字符串相似度比率**保留**供与旧数字对照，但必须标注「分类列口径已变：旧数字来自去注释 token 启发式，新数字来自引擎的结构分类」，不得把两者当同一口径。

**结果平面**：representation / quality / syntax / compile / semantic / verification / coverage / execution 各自独立记录；拒绝原因按码计数、按 artifact 分列。

**计数与比较行的声明**：每个计数字段 SHALL 被读作「本次请求在其声明范围与请求形状下实际做了多少工作」的代理，MUST NOT 被当作跨请求形状可比的单位。比较行 MUST 声明请求形状、声明的 roots/profile（含 multi-release/layout 策略）与引擎版本，并明确该 scope 下这个数「数的是什么」；MUST NOT 跨形状直接比较计数，也 MUST NOT 用未声明形状的数字得出收益或回归结论。

## 重复与负载

- 正式 G0/G1 对照：每工具/配置至少 **10 个独立重复**，交错执行并重建声明初态，保存全部有效原始样本和负载；进程内重复 **6 次**仅作 warm 观察，不能充当独立样本。`--version` 启动参照 **5 次**单列。尾延迟结论另定足额样本。
- 实验前登记主指标、退化/内存界限与统计方法，发布中位数及区间，不事后挑样本/改阈值。旧记录的“非空载只认 >2×”只是粗略观察规则，不构成显著性或性能验收标准；无法区分噪声时如实标记未证实。
- `archive_entries`/`read_bytes` 是实际工作的计费代理；retained weight 是驻留代理，**不是** RSS（RSS 要单独测量，不与计费混谈）。

## join 与对账

- join 键：jadx 的 `dex` 字段（`<war>:<nested.jar>:<entry.class>`）对 jarde 的容器链 + entry raw name；方法键 `name + descriptor`。
- 函数级对账必须扣掉 jadx 折叠的 `<clinit>` 与省略的隐式 `<init>`，并单列该差异，**不自动计为任一引擎的失败或正确**。
- jadx 不是正确性证明；一致性指标也不是。行为等价只用受控、可编译的 fixture 做编译/执行对照。

## 预期变化（用于核对，不作为结论）

冻结基线与 `cd6f2f0` 之间的四项引擎变化，会在同一语料上表现为：

1. 定向 container 访问：单请求的 `archive_entries` / `read_bytes` 应显著低于「一次全树」的水平；
2. 显式 prefix root：`WEB-INF/classes/...` 的 `resolution_definition_unbound` 应在声明了 prefix 之后可绑定（**未声明时仍 unbound**，这正是对照）；
3. 声明类事实同源交接：`jre_declaration_class_not_in_run` 应大幅减少，但 body 质量不因此自动提升；
4. `content` 分类：替代 token 启发式给出结构性分类。

任何一项与预期不符都要如实记录，并把归因所需的声明/配置一并写出。
