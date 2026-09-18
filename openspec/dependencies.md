# 技术栈与依赖选型

核对日期：2026-09-17。本文件记录选型决策、准入门槛及已完成的 P0 证据；未表示后续阶段或所有候选库已通过行为验收。版本来自 crates.io 元数据及下载的 crate 源码；实施时使用明确版本和 Cargo.lock，升级后重新执行相关门槛。

## 技术栈边界

生产核心使用 Rust 2024，MSRV 1.88 已由本地单作业检查及 Linux x86_64 CI 实际验证。当前仍为库 `jarde` 和薄适配器 `jarde-cli` 两个 crate。2026-09-18 依据已存在的生产依赖，规划独立的 [layer-jarde-crates](changes/layer-jarde-crates/design.md)：新增 reader、query、jvm 三包，根包作为门面，Java 包随 P3 第一个实际恢复闭环创建；不是每个模块各建一个包，也不预建 common/core。前提是当前 P2 修正和 3.4 验收通过，拆包完成后继续 3.5/4.x。

目标生产依赖为 query→reader、jvm→reader+query、facade→三包、CLI→facade；query 不依赖 jvm 或 petgraph。现有第三方版本/features 按所有者迁移，不因拆包升级；reader/query 的独立消费、单一身份/预算、A17 和 fuzz/CI 路径是实际验收项，不提前宣称编译提速。

公共 API 同步、可取消，不强制 Tokio、线程池或数据库。CLASS/JAR/WAR 为必需输入；目标代码、bootstrap、JNI 和 launcher 均不执行。JDK、javap、其他反编译器只用于受控测试 oracle，用户依赖不自动联网下载。纯 Rust 要求覆盖生产依赖链，不能只检查顶层 crate 名称。

## P0 首选依赖

| 用途 | 评估版本及发布日期 | 选择理由 | 许可证 / feature 边界 |
| --- | --- | --- | --- |
| classfile、MUTF-8、instruction decoder | [noak 0.7.0](https://crates.io/crates/noak/0.7.0)，2026-07-10 | 借用式 Class/attribute、保留原始 MUTF-8、支持延迟 Code 解码；与按需架构相符 | MIT OR Apache-2.0；生产准入仍需下述边界回归 |
| ZIP/ZIP64 容器 | [rawzip 0.5.1](https://crates.io/crates/rawzip/0.5.1)，2026-07-13 | 逐项目录遍历、raw name、wayfinder 与完整性校验可组合；预算可在分配完整目录前介入 | MIT；自身无依赖、无 unsafe；不替代调用方的压缩比/递归/输出限制 |
| DEFLATE | [flate2 1.1.10](https://crates.io/crates/flate2/1.1.10)，2026-08-28 | 成熟压缩生态；与 rawzip 的结构读取分离，无需自写 inflater | MIT OR Apache-2.0；关闭 default features，仅启用 `rust_backend`，检查 feature 合并未引入 C 后端 |
| 内容摘要 | [blake3 1.8.7](https://crates.io/crates/blake3/1.8.7)，2026-08-20 | 强内容身份用于快照、class bytes 和可选缓存，不用 path/mtime/CRC 代替 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception；启用 `pure`，不引入汇编实现要求 |
| 类型化序列化 | [serde 1.0.229](https://crates.io/crates/serde/1.0.229)，2026-07-18；[serde_json 1.0.151](https://crates.io/crates/serde_json/1.0.151)，2026-07-20 | 库结果和 JSON 适配共享模型，无需自写编码器 | MIT OR Apache-2.0；serde derive；Java 精确字符串使用 UTF-16/raw bytes 字段，不能 lossy 转成 JSON string 身份 |
| 错误类型 | [thiserror 2.0.20](https://crates.io/crates/thiserror/2.0.20)，2026-08-08 | 结构化错误派生，保留错误原因和分类 | MIT OR Apache-2.0；库 API 使用类型化错误，不只返回字符串 |
| CLI 参数 | [clap 4.6.7](https://crates.io/crates/clap/4.6.7)，2026-09-14 | 复用参数解析、help 和错误展示 | MIT OR Apache-2.0；只进入 CLI crate，库不依赖 CLI |
| 对抗性/性质测试 | [proptest 1.11.0](https://crates.io/crates/proptest/1.11.0)，2026-03-24；[tempfile 3.27.0](https://crates.io/crates/tempfile/3.27.0) | 随机/最小化失败输入与测试目录生命周期 | MIT OR Apache-2.0；仅 dev-dependencies；持续 fuzz harness 另行配置 |

活跃度判断不只看发布日期，还需核对上游变更、测试语料和问题响应。noak/rawzip 的近期版本和适配性足以进入首选评估，但不能用 star、下载量或“纯 Rust”替代本项目正确性验收。[noak 上游](https://gitlab.com/frozo/noak)、[rawzip 上游](https://github.com/nickbabcock/rawzip)、[flate2 上游](https://github.com/rust-lang/flate2-rs)。

## 比较与未选方案

| 候选 | 评估结果 | 适用位置 |
| --- | --- | --- |
| [ristretto_classfile 0.33.0](https://crates.io/crates/ristretto_classfile/0.33.0) | 活跃且覆盖广，包含读写/验证；其拥有式对象模型和 MSRV 1.97.1 不如 noak 贴合首期借用式轻量读取。尚未证明可无损满足全部字符串和按需边界 | 可作为独立 classfile 测试 oracle；不得引入 ristretto_vm/classloader 的执行或下载行为 |
| [zip 8.6.0](https://crates.io/crates/zip/8.6.0) | 成熟高层接口，但审阅的目录索引按 raw path 建立 IndexMap；不能直接把 name-index 当作保留所有重复物理 entry 的接口。默认压缩/加密 features 也超出初期需求 | 常规 ZIP 应用合适；本项目首选 rawzip，避免为同名 entry 再自行解析目录 |
| [rc-zip 5.4.1](https://crates.io/crates/rc-zip/5.4.1) / rc-zip-sync 4.4.2 | 可保留 entry 列表、支持 ZIP64 和多 I/O 模式；高层接口会先物化目录，raw name 与逐项预算接入不如 rawzip 直接 | 若未来 I/O provider 需求改变，可重新评估；不是质量不合格的结论 |
| zip 9.0.0-pre3 | 当前查询可见预发布版本 | 无必要不选 prerelease，不能把 cargo info 的 latest 等同稳定推荐 |
| ASM/JADX 等 Java 实现 | 不能作为生产核心依赖，否则引入 JVM 并改变架构 | 仅测试/算法参考；移植源码前单独核对许可证，不因有算法参考就复制代码 |

## 后续阶段优先评估

| 阶段与职责 | 候选 | 准入条件 |
| --- | --- | --- |
| P2 CFG/SCC/支配关系/拓扑排序 | [petgraph 0.8.3](https://docs.rs/petgraph/0.8.3/petgraph/algo/index.html)，发布于 2025-09-30，**已准入**（2026-09-18，证据见 `changes/p2-jvm-ir/verification.md` 的 3.1 节） | 优先复用通用图算法；验证内存权重、确定性、异常边和遍历预算。JVM Frame、returnAddress、SSA origin/effect 仍由语义层负责，不能把普通图算法当 verifier。准入后的硬约束：feature 固定 `default-features = false, features = ["std"]`；SCC 只用迭代的 `kosaraju_scc`（`tarjan_scc` 递归会 abort）；所有输出按 (物理定义, BCI) 自排序（`immediately_dominated_by` 跨进程顺序不定）；`simple_fast` 前自校验 root 归属；多出口合成 super-exit；阶段级不可取消，靠规模上界（`max_blocks` 默认 16 384 / 硬上限 65 535）与支配阶段 195 B/block 记账 |
| P3 Java 文本排版 | [pretty 0.12.5](https://docs.rs/pretty/0.12.5/pretty/)，发布于 2025-09-26 | 优先复用文档组合、分组和断行，验证 source map、注释/转义和输出预算；不自行实现通用 pretty-print 算法 |
| P3 Java 语法检查 oracle | [tree-sitter-java 0.23.5](https://github.com/tree-sitter/tree-sitter-java)，发布于 2024-12-21 | 含生成的 C parser，不符合纯 Rust 生产链；最多作为可选测试工具。语法解析也不证明类型检查或语义等价，且需核对目标 Java release 覆盖 |
| P5 缓存、并行、持久索引 | 实测后选型 | 在出现真实瓶颈时评估有界缓存库、Rayon、SQLite/Rust 原生存储等；必须检查纯 Rust 约束、权重淘汰、取消、损坏回退和许可。当前不锁定产品或格式，也不预先自研 |

只有通用库无法满足已经声明的 JVM 语义或边界时才新增专用实现。发现依赖问题时依次考虑正确使用现有 API、上游修复/升级、局部有测试的补丁，最后才替换依赖或实现缺失部分；禁止另起一套完整 ZIP、DEFLATE、MUTF-8 或图算法。

### ASC / droidsaw 复用复核（2026-09-18）

本轮核对固定的本地 ASC checkout 与 droidsaw-common 2.0.0 发布源码，不代表对最新上游的全面评测；修订和接口证据见 [P2 design §6.1](changes/p2-jvm-ir/design.md)。当前不引入 common 整包：SSA 仍需 JVM Frame、异常逻辑 predecessor、origin/effect 与预算适配，Region 的单 handler 接口不直接承载完整 JVM 异常结构；其 Rust 1.93 要求及未按子模块裁剪的依赖是额外工程成本，不是永久禁用理由。

通用 SSA 在转换输入后可复用，ASC checked 路径也已有预算/取消/内存预留，不能据“DEX vs JVM”或“它没有资源治理”直接排除。当前吸收职责分离、独立小图对照与通用输出先行的路线，不复制 DEX 语义、不 fork 整包、不先建跨项目共享 crate；以后用同一组异常、块顺序、资源停止探针比较薄适配和私有实现后再决定。该决定与 Jarde 内部 workspace 分层是两个问题。

## 上游准入门槛与已知关注点

以下是依赖准入与持续验证清单；每项完成度以分项说明为准，尚未覆盖的部分留在对应 tasks：

1. **noak CP/字符串**：P0 2.3 已覆盖无效 tag/index、Long 末槽、MUTF-8 NUL、补充字符、孤立 surrogate 和非法编码；同一局部预检同时约束 Double 双槽。回归确认 0.7.0 会接受缺少合法后继保留槽的末位 Long/Double，因此适配层在 noak 前拒绝该边界，不能把构造成功等同 JVMS 合法。P0 3.2 又以真实 Header descriptor 路径覆盖 encoded NUL、surrogate pair 和孤立 surrogate，并用 256-case UTF-16 unit 性质测试核对 raw/UTF-16/escaped 稳定性；这些测试验证结构保真，不冒充 descriptor 语法验证。
2. **noak instruction cursor**：P0 2.4 已覆盖 wide、相对 Code 起点的 switch padding、high/low 边界、`i32::MAX` table key、截断、reserved opcode、CP index、异常表顺序、局部预算/取消及首错停止。P0 3.2 进一步要求任意 0–64 byte 指令向量都通过公共方法入口返回 Complete 或保留可靠前缀的 method-local Partial，并核对 raw opcode/span、stop/diagnostic code 和预算；非法零项 `tableswitch`、malformed `wide` 及 synthetic `jsr/jsr_w/ret` 均有定向回归。固定 ECJ 4.6.1 历史目标语料覆盖 45.3–52.0，其中 45–48 的 finally codegen 保留真实 `jsr`/`ret` 边界。适配不迭代存在 i32 上界递增风险的 `TablePairs`；noak 成功事件之后的 checked 宽度薄适配同时由下一 BCI、最终 code_length 和 operand 类别回归约束。测试专用 OpenJDK 25 Class-File API oracle 已对 `tableswitch`、`lookupswitch` 与 CP-bearing fixed-width 指令交叉检查：runtime `25.0.4+7`，动态 fixture 固定为 52.0，SHA-256 `04ea6ad5115a0c17d4bd604ef262f8110e417efa8725f39c9de9641565c2b345`；oracle scope 仅为 instruction boundary，不是 verification。
3. **Header 延迟性**：P0 2.3 已用非法 Code 内容验证 Header 只读取外壳，并为未知 class/field/method attribute 保留完整及 content span；名称和 descriptor 同时保留原始 MUTF-8、UTF-16 units 与安全转义显示，不以 lossy 文本作为身份。P0 3.2 已补齐 Code 嵌套未知 attribute 的长度/截断回归，证明 Header 不进入 Code 子属性而 bytecode 请求在读取该结构时拒绝非法长度；attribute 合法位置、基数和版本语义仍由后续 capability registry 负责。
4. **rawzip 容器**：P0 3.1 已覆盖 CLASS 与 JAR/WAR 路径入口、重复 raw name/同内容不同 origin、EOCD 与实际 entry 数不符、中央/局部头冲突、data descriptor、前置脚本、STORED/DEFLATED、CRC/size、加密/未知压缩、重叠数据区间和源替换快照。ZIP64 使用确定性生成的 248-byte 小型 fixture，classic sentinel、ZIP64 EOCD/locator、64-bit entry hint、物理 spans 和读取摘要均经真实 `ArtifactSnapshot` 入口复核；不依赖 65535-entry 或多 GiB 测试数据。复用库的解析与验证接口，不把高层成功等同全部输入合规。
5. **预算与 I/O**：P0 3.3 已验证 artifact open、中央目录枚举、entry locator/read、class Header 和单方法 bytecode 在同一请求中的累计 Budget，以及入口预取消和内部循环协作取消。DEFLATE 按实际展开字节计 `EntryBytes`，压缩输入计 `ReadBytes`；目录记录逐项计 `ArchiveEntries`；snapshot、临时物化和结果缓冲分别计费。Header 不计 `CodeBytes`，方法请求只计所选 Code shell 和可靠指令前缀。P1 分别验证 STORED 子范围访问和 DEFLATED nested 有界物化，不承诺任意 nested 零拷贝。
6. **供应链**：P0 3.4 已固定 lockfile，并在 Linux x86_64 运行 stable 与 MSRV 1.88 CI、`cargo-deny 0.20.2` 的 advisories/licenses/bans/sources 门禁、feature tree 和生产依赖边界检查；本地 Linux aarch64 使用相同 lockfile 补充验证。当前生产 feature 保持 BLAKE3 `pure` 与 flate2 `rust_backend`，未引入 JVM、网络、async、图 IR 或数据库 runtime。后续依赖升级必须重新执行这些门槛；冷/热完整结果一致性仍由引入缓存的 P5 负责。

## 可复核源码版本

以下 commit 来自对应 crate 发布包的 `.cargo_vcs_info.json`，作为后续核对入口，不使用会漂移的默认分支作为唯一依据：

| 发布包 | commit |
| --- | --- |
| noak 0.7.0 | `2ea894274abd4da742a161dfb6aaf447c95c7c9b` |
| rawzip 0.5.1 | `571e479673646848ec16b12213b7600d639971d8` |
| flate2 1.1.10 | `ed93d4fc60eaf876c6aded741bf992d524551930` |
| ristretto_classfile 0.33.0 | `37da1643d75a170029655c1506d4e28f2756f4ff` |
| zip 8.6.0 | `771dfc534d2614158af5497ea3dff4d4208d7db1` |
| rc-zip 5.4.1 | `373fa9bbbdef30d006e2b025e5be8c8ea1bc49dd` |

版本规则以 [JVMS 8 §4.1](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.1) 和 [JVMS 26 §4.1](https://docs.oracle.com/en/java/javase/26/docs/specs/jvms/jvms-4.html#jvms-4.1) 为交叉检查入口；release/feature Registry 实施时固定各代规范，不能只提高 major 上限。
