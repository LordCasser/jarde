## Context

架构来源为 `JVM_Rust_Engine_Final_Architecture.md`，具体规划范围见 proposal。建立本 change 时仓库仅有规划文档；当前实施状态以 `tasks.md`、实际代码和验证记录为准，仍没有需要向后兼容的既有公开 API。

## Goals / Non-Goals

**Goals:** 以两 crate、少量逻辑模块实现 P0；外部依赖只通过内部适配进入，公共结果使用本项目类型。解析与显示保持原始身份、范围和状态。

**Non-Goals:** 不提前创建空的 query/ir/java crate，不实现后续阶段伪 API，不宣称完整 dialect validation 或 JVM verifier。

## Decisions

1. 根库 crate `jarde` 包含 `artifact`、`classfile`、`model`、`budget`、`engine`，并保持宿主无关；`jarde-cli` 只编排库调用和 JSON 序列化。已实现的 `Engine` 无内部可变 session 状态，调用方为每个请求持有同一 `Budget`，entry target 必须携带枚举所得完整物理 metadata，不按 ordinal/path 隐式扫描。CLI 采用单请求 JSON 协议，1 MiB 控制面上限，成功结果原样封装 Engine report，Partial/Cancelled 不改写为失败；JSON transport 先计数、再检查和计费，并显式返回 `response_bytes`。未来 `reverse-engine` 集成由独立 adapter/backend crate 单向依赖 `jarde` 并映射宿主协议，核心不得依赖 CLI、MCP、`reverse-engine-backend-api` 或宿主状态类型。无需 async runtime、数据库、JVM 或动态插件 ABI。
2. 首选 noak 0.7.0 的借用式 Reader、MUTF-8 和指令解析，采用前必须通过 [准入测试](../../dependencies.md)。Header 遍历 attribute 外壳，不解码未请求的 Code 指令；保留原始 attribute slice 的 class 内偏移。项目补充版本/预算/coverage，不复制整个 parser。实施已确认 noak 0.7.0 会接受占用常量池最后声明槽位、却没有合法保留后继槽的 Long/Double；`classfile` 适配在进入 noak 前用局部 slot/payload 边界预检拒绝该输入，不另建完整 CP parser。公开 Bytecode 游标在首个解码错误处停止，不尝试同步到后续 opcode；指令语义仍由 noak 解码，项目只在 noak 成功后用 checked 宽度薄适配补足其未公开的消费终点，并以后一条 noak BCI、最终 code_length、switch/wide 边界回归和 JDK Class-File API oracle 交叉检查，不能把该适配扩张为平行语义 parser。其他上游长度、索引或迭代器问题优先修复/升级上游，必要时才做有回归样本的薄适配。
3. 使用 rawzip 0.5 逐项枚举中央目录、保留 raw name 和 ordinal，并用库的 CRC/size verifier 读取 entry。flate2 仅启用 rust_backend。zip 8.6 的路径索引可能合并重复项，rc-zip 默认一次物化整个目录；两者不如 rawzip 适配 P0 的预算边界。
4. P0 使用有上限的内存快照，打开时取得独占字节副本并计算 BLAKE3。路径打开读前/读后检查元数据；后续读取仅用快照字节。元数据检查只能发现可检测的并发修改，不能证明恶意并发写入期间曾存在该完整文件版本；强一致调用应传入已固定的字节源。接受首次 O(artifact bytes) 拷贝，未来按实测再引入私有 backing file；不使用可变文件 mmap 冒充 snapshot。
5. P0 枚举归档根的所有物理 entry，包括 WAR 路径及未激活的 MR entries；嵌套 JAR 显示为 container candidate、明确递归未请求。不按布局过滤物理事实。递归策略、Boot layout、MR runtime view 属于 P1。
6. 请求单独持有 Budget，检查输入/entry/class/attribute/code/结果条数、累计读取与输出、协作取消和 elapsed time。P0 3.3 已通过公共 Engine 入口验证同一 Budget 从 open 到 inspection 的累计计费与阶段失败原子性，并由 artifact/classfile 私有确定性 seam 覆盖枚举、locator/read、CP preflight、handler 和 instruction 中途取消；这些 seam 不扩张为公共 API。目标 class 尚未物化或 Header/Code 结构尚未建立时，预算/取消为顶层 `Err`；ZIP 根已建立后的枚举及 Code 已建立后的 handler/instruction 扫描保留可靠前缀和 Partial/Cancelled report。Header 不计 `CodeBytes`，单方法请求只解码并计费所选 Code，未请求方法的非法 Body 不影响结果。JSON CLI 的 P0 请求协议不暴露 `CancellationToken` 注入，只继承核心预算/结果语义并支持 elapsed limit；此支持边界在 3.4 矩阵中显式记录。结果缓冲上限与输入上限显式记录；不声称进程 RSS 或硬超时保证。
7. Strict 拒绝已识别的版本违规和未支持 dialect；Forensic 可返回边界可靠的已读结构并标记诊断。45–52 建立逐版本 fixture，53–71 在 P0 仅结构探测；preview/未来版本明确为未支持 dialect。遵循 JVMS 的历史 minor 规则：45–55 的 minor 不能一律强制为 0，56+ 只能为 0/65535；Java 8 的 runtime 版本接受区间单独检查至 52.0。结构完整、dialect 检查程度、verification 独立，不把 45.x 的 65535 当成 preview。当前 Header 公共入口已实现 Strict/Forensic 版本 gate，并以 `version_only` scope 明确尚未执行 CP tag、flags、attribute 位置/基数或 opcode dialect 验证；未来 major 优先标记 future release，不根据未知 major 的 65535 minor 推断 preview。
8. 初始技术栈为 Rust 2024，MSRV 暂定 1.88；只创建库和 CLI 两个 crate。P0 的生产模块仅含 artifact、budget、classfile、engine、model 和 error，公共 Body 结果为原始 `InstructionFact`/exception-handler facts；生产依赖图不含 query、resolver、CFG/SSA/AST/IR、Decompiler、JVM、网络、async 或数据库运行时。A17 的构造计数首次由 P1 在真实 X1 入口承担，P0 不为证明不存在的能力引入空 IR 或假 counter。后续仍用编译期依赖图保证 Query 不依赖 Decompiler；通用图算法、文档排版和未来缓存均先评估现成库。Cargo.lock、CI 及五维支持矩阵按实施任务逐项建立，不以空 crate 冒充后续能力。

## Risks / Trade-offs

- 上游 decoder 不承担完整 JVMS validation → 支持矩阵明确分层；添加截断、索引、switch/wide 和属性边界回归。
- 内存快照限制大 WAR → 可配置上限；超限显式错误，后续 backing store 独立 change。
- 无法解释的 attribute 仍可能含引用 → 保留 span 和 uninterpreted 列表；P0 结果不标为 X1 完整。
- 取消为协作式 → 在 I/O 分块和解析循环检查；第三方单次解析受 class/input size 上限约束，不宣称硬时限。
- 上游未公开指令结束位置 → 当前只在 noak 成功解码后使用局部、checked 的宽度适配，并用下一 BCI、最终 code_length、全类别回归和 JDK oracle 防止漂移；若 noak 后续公开消费终点则删除该适配。发生错误时输出最后可靠的扫描边界，不在错误后继续猜测 opcode。

## Migration Plan

无旧实现迁移，不设置向后兼容负担。代码与测试按 `tasks.md` 逐项实施；只有全部实施任务验收后才能归档并同步主规格。当前不发布 crate，也不将尚未完成的阶段归档为已实现能力。
