复核状态（2026-09-17）：接手时是 **9/11**，不是交接摘要中的 8/11；重新打开 2.1、2.2、3.1 后为 **6/11**。修复进展：**2.1、2.2、3.1、3.3 均已修复/完成并通过独立只读复核（Approve），当前 10/11**；剩余 3.4（文档、完整 CI、归档）。R1–R4 的反例、源码位置及范围见 [design](design.md#2026-09-17-复核与推进顺序)，实施与复核证据见 [verification](verification.md#2026-09-17-修复轮r1--r4)。2.2 实施期间另发现并修复了方法 `Signature` 的 `Result`/`ThrowsSignature` 假失败（合法 javac 输出被判 malformed）。本轮只修改实现、测试与验证记录，不提交或推送工作树。

## 1. Query model and views

- [x] 1.1 定义 query relation、consumer schema、PhysicalView/RuntimeView、LoadDomain 和有向 origin chain；将物理定义身份迁移为显式 standalone root/archive entry，并用 API 类型测试覆盖 A06/A07
- [x] 1.2 实现显式 artifact-tree 的 bounded nested/Boot provider、nested entry 重读与 `nested_depth` 高水位预算，复用 rawzip/flate2 和 P0 snapshot；用 DEFLATED nested fixture 验证 A08
- [x] 1.3 实现按 container、Manifest evidence 和唯一 winning level 派生的标准 MR-JAR selection report，以及有界路径/class Header 合规诊断；用 Java 8/11/17 三视图 fixture 验证 A06；不得引入 XRef/resolver、loader/module resolution、API 等价、verifier 或跨 container 消歧

## 2. Structural XRef

- [x] 2.1 收口 CP candidate 与 code consumer 扫描：完成 R4 的共享 class candidate 判定，损坏/截断 magic 必须带真实 origin 的 diagnostic 与非 Complete coverage，统一 code/metadata/bootstrap 的大小写和裸 `.class` 边界；增加候选损坏与普通资源对照，保留 A01/A02/A17 和既有 code consumer 回归（2026-09-17 修复并独立复核 Approve；证据见 verification 的修复轮 R4）
- [x] 2.2 收口 metadata consumer：完成 R2 的 Record component 注解和 Code 内类型注解、R3 的实际使用 MethodType/MethodHandle/dynamic/bootstrap descriptor 类型；用位置唯一的正例、未使用 CP/BootstrapMethods 反例、单类别请求和组合请求验证 A03–A05；所有正例同时断言 items、精确来源、execution、coverage、diagnostics，不以原始 descriptor literal 代替类型引用（2026-09-17 修复并独立复核 Approve：首轮 Reject 的嵌套落位 B1 与 `Type` 类别触达 bootstrap 可达 descriptor 的 B2 已按修正方向闭合；同轮修复方法 `Signature` 的 Result/ThrowsSignature 假失败；证据见 verification 的修复轮 R2/R3）
- [x] 2.3 实现 deferred bootstrap/condy graph，验证标准 lambda 可追溯、任意 bootstrap 不解析为 runtime target，并覆盖 A04/A05
- [x] 2.4 实现 P1 对 `references_definition`/`may_dispatch_to` 的 `UnsupportedAnalysis` 响应，保留 target relation 与 raw symbol evidence，并将 Base/Sub 候选扩展留给 P2

## 3. Query API and acceptance

- [x] 3.1 修复 R1：游标及其 digest 绑定完整 QueryTarget，升级 QUERY_ENGINE_SCHEMA，拒绝 target/schema/digest 不匹配；覆盖 owner/name/descriptor、literal kind/原始值/float 位模式变化，库与 CLI 均拒绝跨目标续页；同目标在不同页大小/预算下按发布边界不重不漏，维持 A14/A18 和 Unknown/Partial/Complete 契约（2026-09-17 修复并独立复核 Approve；复核要求的预算续页等价用例已补齐，证据见 verification 的修复轮 R1）
- [x] 3.2 在 jarde-cli 暴露查询范围、view、预算和 evidence，验证库与 CLI 结果语义一致
- [x] 3.3 在 R1–R4 回归及独立复核通过后，按 design 验收表建立 P1 版本/打包/身份/对抗语料索引与结构 XRef golden/性质及有界 fuzz 门禁：映射 A01–A08、A14、A17、A18，保留来源/生成命令/编译器版本/摘要；覆盖跨页等价、未使用 CP 不产出 X1、局部未知不伪装否定、预算/取消和嵌套容器组合；声明性 UnsupportedAnalysis 与解析结果必须可区分（2026-09-17 完成并独立复核 Approve：语料索引见 verification 的 3.3 两节，golden/demo 清单与 fuzz 工具、限值、冒烟记录同处；两处门禁削弱点已关闭并各有可证伪实验）
- [ ] 3.4 在 3.3 通过后同步 README、OpenSpec 入口/roadmap/acceptance/proposal/verification 与五维支持矩阵，按 CI 原命令跑 fmt、all-targets/all-features/locked clippy、双固定种子 tests、ignored JDK 25 oracle、示例、依赖树、MSRV 与 supply-chain；检查差异，按可编译边界提交/推送并记录最终候选 SHA 对应的 CI，门禁全部通过后才归档同步主 specs；仓库卫生改动单独提交，P1 仍不包含 Decompiler

## 执行顺序与收口规则

1. 先 3.1（R1），再 2.1（R4）、2.2（R2/R3）；每步保留最小复现，先确认旧实现失败，再修复并独立复核。共享 query schema/classfile reader 各保持单一所有者，避免并发改同一契约。
2. 再 3.3；复用已有 250 项测试与 ECJ 历史语料，不另建重复 fixture 框架。缺口按 design 的验收映射补齐。
3. 最后 3.4；旧 CI run 只能证明其对应 SHA，不能替代本工作树的证据。不要机械按历史 agent 任务拆分当前相互依赖的 reader/xref/query 文件；提交边界须能独立编译验证。
4. 重复物化与性能优化移交 P5；dead-code 清理、跨 API aggregate 优先级统一另行小变更。若 3.3 证明它们造成漏扫描或假 Complete，再以具体反例纳入当前正确性修复，不提前重构。
