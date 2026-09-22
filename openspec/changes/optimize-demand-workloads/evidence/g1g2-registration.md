# 任务 3.2：准入项的登记说明

本文件只登记 §3.1 判为**准入**的方案（O2 的「选定定义的读取复用」），并为其余项写明它们**不能**绕过哪些
协议与预算设计。它不创建 change 目录、不改任务表：可落地的段落按下面的名字与范围登记即可。

## 1. 准入项：O2 选定定义的读取复用

**独立实施 change 的名字与范围（建议登记）**

- 名字：`reuse-selected-class-read`。
- 范围：让「一次操作已执行过的、针对它所选定的物理定义的受信读取」在同一 snapshot 与同一 store 内可被
  下一次选择该定义时复用；复用的是**读取**（字节 + 该读取自己建立的 digest），不含解析结构、不含
  `PreparedClass`、不含任何方法级产品。
- 不属于该 change：容器/CP/Header 层（O1 与 `642e49f` 已交付）、按类准备与批量（O4）、并行与窗口（O7）、
  query cursor 与 coverage（O3，归 `add-demand-driven-core-results`）、证据产品（O8 的 demand-driven 部分）。

**capability delta**

| spec | delta | 内容 |
| --- | --- | --- |
| `facts-cache` | ADDED Requirement | 「已验证的定义读取可在显式界内复用」：读取按**定义身份**（snapshot、物理位置、digest、length、variant）与读取 schema 绑定；只发布完整读取；与容器/CP 层共用同两个界（entries + retained bytes），满则拒绝、不淘汰；命中不计入本次 `archive_entries`/`read_bytes`/`entry_bytes`/`class_bytes`，不重放旧 usage、不重置预算；关闭或拒绝时在剩余预算内走直读 |
| `facts-cache` | MODIFIED Requirement | 「Complete cache identity and invalidation」的身份维度表增加定义读取一层；「Verified container facts are reusable within explicit bounds」的 scenario「Verified backing saves work but does not grant a fresh budget」现文要求「随后仍需读取所选 class」，必须改述为「仍须**定位该 entry 并核对变体与调用方声明的身份**；读取本身可由该定义的已保留读取作答」 |
| `artifact-snapshots` | MODIFIED Requirement | 快照新增一个受信读取的复用入口（`jarde-reader` 与 `jarde-jvm` 之间需跨 crate 调用），语义为「命中即返回该读取的字节与该读取建立的 digest；不命中读零字节、计零费用」 |

`measured-execution` 无需 delta：本项不改工作负载词表、阶段口径或指纹规则。

**直接路径（谁调用）**

1. `crates/jarde-reader/src/inspect.rs::materialize_definition` —— facade 的身份绑定读取
   （`class_view` / `class_source` / `list_members` 的选中读取）。
2. `crates/jarde-jvm/src/providers.rs::read_definition_class` —— 方法分析驱动的读取与 class task 读取
   （W1/W5 的形状）。
   两处都经 `ArtifactSnapshot::{retained_definition_read, remember_definition_read}`；名字搜索的**候选读取**
   （`facade::read_class_declaration`）**不**进这条路径——按现有产品文档，搜索的读取是搜索自己的成本。

**回退责任**

| 触发 | 行为 |
| --- | --- |
| 请求未附加 store（现有 opt-in 状态） | 不查、不记，走直读；本次读取照常计费 |
| 容量不足（entries 或 retained bytes） | 拒绝并计数；本次请求继续消费自己的读取，后续请求走直读；不淘汰、不驱逐 |
| 条目由另一声明（format / registry）写入 | 丢弃该条并计数，走直读；不复用、不转换 |
| 命中 | 仍执行目录定位、变体核对与调用方声明身份的核对；命中只免掉「重读该 entry 的字节与其 CRC/摘要」 |
| 取消 / 预算耗尽 | 命中不得恢复请求、不得提交 Complete、不得重置预算（`facts-cache` 既有 scenario） |
| 语义或身份回归 | 撤回该独立提交；不得借它回退其它正确性修复 |

**明确不做（也是本项不失守的边界）**：不保留解析结构或 `PreparedClass`（其解析视图借用字节，保留需要自引用
或 `unsafe`，design §5 与 §11 禁止）；不跨 snapshot 共享；不做全局 `MethodIr` 缓存；不引入淘汰；不新增依赖、
线程或 `unsafe`；不改输出字节、诊断、coverage 或顺序契约。

## 2. O1 仍由已有 change 唯一负责

O1（容器定向访问、目录/backing 复用、跨请求保留）的实现与行为规格由归档的
[bound-container-lookup](../archive/2026-09-20-bound-container-lookup/proposal.md) 唯一负责（8/8，`b22ea04`）。
本专项只引用其验证（`evidence/o1-container-lookup.md` 的真跑复核），**不**为它新立 change、**不**在
`reuse-selected-class-read` 里重开容器层，也**不**把容器层的收益记到新 change 名下（见 `g1g2-admission.md` §3.1）。

## 3. 其它项：不得绕过的协议/预算设计

- **O3（query 细粒度停止）**：实施归 `add-demand-driven-core-results` D4。前置是 cursor identity、coverage、
  diagnostics 的 delta 子 spec（design §6）；在子 spec 落地前不实现、不实验。禁止把「页满」记成预算耗尽或
  完成、禁止在存在未覆盖范围时把空 items 当完整空结果、禁止静默跳项或重发已发布前缀。
- **O4/O7（bulk 与并行）**：唯一所有者是 `add-parallel-bulk-recovery`。本专项只提供归因（按类准备、
  1/N worker、`take_front` 等待）。禁止普通入口隐式批量、禁止用逐方法新预算突破 batch 总额度、禁止在
  fingerprint 里删 coverage/diagnostics 掩饰调度差异；窗口设计的改动是该 change 的未裁定架构决策。
- **O5（更高层 store 与淘汰）**：暂缓。重开条件（可观察）= 同一 store 上跨 ≥2 个请求复用同一方法/同一
  resolution 的实测复用距离，且驻留可在 `FactsCapacity` 字节界内被证明。本轮读取复用**只**增加一个新
  product 并复用同两个界，不等于重开更高层 store。
- **O6（预热）**：暂缓，触发条件 3 条（宿主显式导航轨迹与实测 `q`、已声明且未满足的前台延迟目标、独立预算/
  取消模型）。禁止用「读取复用已让第二次请求更便宜」当作预热准入证据——那是命中，不是提前支付。
- **O8**：编码/增量摘要/mmap 已否决；CLI 长度固定点暂缓（触发=大输出单文档）。保持：可信 digest 只能来自
  已验证读取、owned immutable bytes 为安全路径、输出计数含分隔符、不发布半个成功 JSON。
- **协议与预算的共同约束**：任何准入项都不得放宽 `Budget` 维度或 `FactsCapacity`、不得跳过身份/变体验证、
  不得把命中当作「本次读取」计费、不得以复用为由扩大 coverage 或跳过诊断。

## 4. 规划未完成的子项不开始生产实现

本阶段（3.1–3.4）**没有**任何产品代码进入仓库：O2 的单因素原型只存在于
`evidence/g1g2-o2-prototype.patch`（sha256[:16] `294d7ba90cd8b2c0`），观测口在
`g1g2-o2-harness-observation.patch`（`ad68bcdf9e7bfe5f`），负向测试在 `g1g2-o2-negative-tests.patch`
（`2d20bce180d09ad1`）；三者都已在实验后从工作区还原（`git diff` 为空，`git status` 只有本目录新增的证据
文件）。O2 的生产实现必须由 `reuse-selected-class-read` 按上表逐项定义 delta 后再开始。
