## 1. Query model and views

- [x] 1.1 定义 query relation、consumer schema、PhysicalView/RuntimeView、LoadDomain 和有向 origin chain；将物理定义身份迁移为显式 standalone root/archive entry，并用 API 类型测试覆盖 A06/A07
- [ ] 1.2 实现 bounded nested/Boot provider，复用 rawzip/flate2 和 P0 snapshot；用 DEFLATED nested fixture 验证 A08
- [ ] 1.3 实现 MR-JAR 版本选择与不合规归档诊断；用 Java 8/11/17 三视图 fixture 验证 A06

## 2. Structural XRef

- [ ] 2.1 实现 CP candidate 与 code consumer 扫描，验证未使用 `Methodref` 不生成调用并满足 A01/A17
- [ ] 2.2 实现 metadata、annotation、signature、catch、inner/nest、module 和资源 consumer，按类别 fixture 验证 A03
- [ ] 2.3 实现 deferred bootstrap/condy graph，验证标准 lambda 可追溯、任意 bootstrap 不解析为 runtime target，并覆盖 A04/A05
- [ ] 2.4 实现 P1 对 `references_definition`/`may_dispatch_to` 的 `UnsupportedAnalysis` 响应，保留 target relation 与 raw symbol evidence，并将 Base/Sub 候选扩展留给 P2

## 3. Query API and acceptance

- [ ] 3.1 实现 query compiler、分页、取消、Unknown/Partial/Complete 与 coverage JSON，验证 A14 和稳定续页场景
- [ ] 3.2 在 jarde-cli 暴露查询范围、view、预算和 evidence，验证库与 CLI 结果语义一致
- [ ] 3.3 建立 P1 版本/打包/身份/对抗语料和结构 XRef golden/fuzz 测试，验证 A01–A08、A14、A17、A18，并验证 P1 不把未支持的解析关系误报为已解析
- [ ] 3.4 更新五维支持矩阵并执行 cargo fmt、clippy、test 与 OpenSpec strict validation，确认本 change 仍未包含 Decompiler
