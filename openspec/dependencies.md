# 技术栈与依赖选型

核对日期：2026-09-16。本文件是实施前的选型决策和准入门槛，未表示引擎或所有候选库已通过行为验收。版本来自 crates.io 元数据及下载的 crate 源码；实施时使用明确版本和 Cargo.lock，升级后重新执行相关门槛。

## 技术栈边界

生产核心使用 Rust 2024，MSRV 初选 1.88，由未来 CI 实际验证。首期仅库 `jarde` 和薄适配器 `jarde-cli` 两个 crate；artifact、classfile、model、query、resolver、IR、Java 输出先保持逻辑模块边界，随真实编译依赖再拆包。

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
| P2 CFG/SCC/支配关系/拓扑排序 | [petgraph 0.8.3](https://docs.rs/petgraph/0.8.3/petgraph/algo/index.html)，发布于 2025-09-30 | 优先复用通用图算法；验证内存权重、确定性、异常边和遍历预算。JVM Frame、returnAddress、SSA origin/effect 仍由语义层负责，不能把普通图算法当 verifier |
| P3 Java 文本排版 | [pretty 0.12.5](https://docs.rs/pretty/0.12.5/pretty/)，发布于 2025-09-26 | 优先复用文档组合、分组和断行，验证 source map、注释/转义和输出预算；不自行实现通用 pretty-print 算法 |
| P3 Java 语法检查 oracle | [tree-sitter-java 0.23.5](https://github.com/tree-sitter/tree-sitter-java)，发布于 2024-12-21 | 含生成的 C parser，不符合纯 Rust 生产链；最多作为可选测试工具。语法解析也不证明类型检查或语义等价，且需核对目标 Java release 覆盖 |
| P5 缓存、并行、持久索引 | 实测后选型 | 在出现真实瓶颈时评估有界缓存库、Rayon、SQLite/Rust 原生存储等；必须检查纯 Rust 约束、权重淘汰、取消、损坏回退和许可。当前不锁定产品或格式，也不预先自研 |

只有通用库无法满足已经声明的 JVM 语义或边界时才新增专用实现。发现依赖问题时依次考虑正确使用现有 API、上游修复/升级、局部有测试的补丁，最后才替换依赖或实现缺失部分；禁止另起一套完整 ZIP、DEFLATE、MUTF-8 或图算法。

## 上游准入门槛与已知关注点

以下是待实施验证项，尚未完成：

1. **noak CP/字符串**：无效 tag/index、Long/Double 末槽、MUTF-8 NUL、补充字符、孤立 surrogate、非法编码和 descriptor 字节边界。0.7.0 的 CP 双槽读取需要额外关注最后保留槽；禁止直接假定构造成功即 JVMS 合法。
2. **noak instruction cursor**：wide、相对 Code 起点的 switch padding、负表长、high/low 边界、`i32::MAX` table key、截断和 reserved opcode。审阅发现 `TablePairs` 的 key 递增涉及 i32 上界；必须建立回归并避免调用可能溢出的遍历路径，必要时推动上游修复。首个错误后不得继续解码。
3. **Header 延迟性**：无效 Code 不影响可读 Header 外壳；未知 attribute 保留位置；子属性长度、位置/基数/版本验证由 capability registry 明确声明。MUTF-8 descriptor 不能靠 lossy 字符迭代恢复精确身份。
4. **rawzip 容器**：重复 raw name、同内容不同 origin、EOCD 与实际 entry 数不符、中央/局部头冲突、ZIP64、data descriptor、前置脚本、CRC/size、加密/未知压缩、重叠数据区间。复用库的解析与验证接口，不把高层成功等同全部输入合规。
5. **预算与 I/O**：按实际展开字节限制 DEFLATE，校验声明大小；目录逐项计数；同一请求累计计算 snapshot/临时物化/结果缓冲。P1 分别验证 STORED 子范围访问和 DEFLATED 有界物化，不承诺任意 nested 零拷贝。
6. **供应链**：固定 lockfile；MSRV/stable CI；许可证与 RustSec 检查；`cargo tree -e features` 检查传递 feature 与 C/JVM 运行依赖；完整结果冷/热一致。检查通过之前不标记依赖通过生产验收。

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
