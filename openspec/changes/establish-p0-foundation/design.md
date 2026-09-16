## Context

架构来源为 `JVM_Rust_Engine_Final_Architecture.md`，具体规划范围见 proposal。当前仅交付文档，没有实现、需要兼容的代码或公开 API。

## Goals / Non-Goals

**Goals:** 以两 crate、少量逻辑模块实现 P0；外部依赖只通过内部适配进入，公共结果使用本项目类型。解析与显示保持原始身份、范围和状态。

**Non-Goals:** 不提前创建空的 query/ir/java crate，不实现后续阶段伪 API，不宣称完整 dialect validation 或 JVM verifier。

## Decisions

1. 根库包含 `artifact`、`classfile`、`model`、`budget`、`engine`；CLI 只编排库调用和 JSON 序列化。无需 async runtime、数据库、JVM 或动态插件 ABI。
2. 首选 noak 0.7.0 的借用式 Reader、MUTF-8 和指令解析，采用前必须通过 [准入测试](../../dependencies.md)。Header 遍历 attribute 外壳，不解码未请求的 Code 指令；保留原始 attribute slice 的 class 内偏移。项目补充版本/预算/coverage，不复制整个 parser。公开 Bytecode 游标在首个解码错误处停止，不尝试同步到后续 opcode；上游长度、索引或迭代器问题优先修复/升级上游，必要时才做有回归样本的薄适配。
3. 使用 rawzip 0.5 逐项枚举中央目录、保留 raw name 和 ordinal，并用库的 CRC/size verifier 读取 entry。flate2 仅启用 rust_backend。zip 8.6 的路径索引可能合并重复项，rc-zip 默认一次物化整个目录；两者不如 rawzip 适配 P0 的预算边界。
4. P0 使用有上限的内存快照，打开时取得独占字节副本并计算 BLAKE3。路径打开读前/读后检查元数据；后续读取仅用快照字节。元数据检查只能发现可检测的并发修改，不能证明恶意并发写入期间曾存在该完整文件版本；强一致调用应传入已固定的字节源。接受首次 O(artifact bytes) 拷贝，未来按实测再引入私有 backing file；不使用可变文件 mmap 冒充 snapshot。
5. P0 枚举归档根的所有物理 entry，包括 WAR 路径及未激活的 MR entries；嵌套 JAR 显示为 container candidate、明确递归未请求。不按布局过滤物理事实。递归策略、Boot layout、MR runtime view 属于 P1。
6. 请求单独持有 Budget，检查输入/entry/class/attribute/code/结果条数、累计读取与输出、协作取消和 elapsed time。结果缓冲上限与输入上限显式记录；不声称进程 RSS 或硬超时保证。
7. Strict 拒绝已识别的版本违规和未支持 dialect；Forensic 可返回边界可靠的已读结构并标记诊断。45–52 建立逐版本 fixture，53–71 在 P0 仅结构探测；preview/未来版本明确为未支持 dialect。遵循 JVMS 的历史 minor 规则：45–55 的 minor 不能一律强制为 0，56+ 只能为 0/65535；Java 8 的 runtime 版本接受区间单独检查至 52.0。结构完整、dialect 检查程度、verification 独立，不把 45.x 的 65535 当成 preview。
8. 初始技术栈为 Rust 2024，MSRV 暂定 1.88；只创建库和 CLI 两个 crate。用编译期依赖图保证 Query 不依赖 Decompiler；通用图算法、文档排版和未来缓存均先评估现成库。Cargo.lock、CI 及五维支持矩阵属于实施任务，当前不生成空工程。

## Risks / Trade-offs

- 上游 decoder 不承担完整 JVMS validation → 支持矩阵明确分层；添加截断、索引、switch/wide 和属性边界回归。
- 内存快照限制大 WAR → 可配置上限；超限显式错误，后续 backing store 独立 change。
- 无法解释的 attribute 仍可能含引用 → 保留 span 和 uninterpreted 列表；P0 结果不标为 X1 完整。
- 取消为协作式 → 在 I/O 分块和解析循环检查；第三方单次解析受 class/input size 上限约束，不宣称硬时限。
- 上游未公开指令结束位置 → 复用解码事件或推动上游补足；不能再维护一套 opcode 长度表。发生错误时输出最后可靠的扫描边界，未完整发布的指令必须计入未覆盖范围。

## Migration Plan

无旧实现迁移，不设置向后兼容负担。本次完成规格、依赖评估和 OpenSpec validation 即结束；后续实施时再完成代码及测试，全部实施任务验收后才能归档并同步主规格。当前不发布 crate，不将规划档案归档为已实现能力。
