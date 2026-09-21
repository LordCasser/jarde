## Why

jarde 的核心价值是以明确身份和可核对边界回答当前问题。现有读取、分析阶段和方法恢复已经按需分层，但普通组合入口仍有同一类事实的重复准备；结构查询先收集整个范围或单元，再截取页面；恢复报告固定构造所有详细证据。这使“只列成员”“取一个方法”“追问某处来源”的成本与结果体积超过实际需要。

信息是否有用取决于当前问题。定位身份、实际答案、影响结论的缺口与停止必须随结果交付；内部算法依赖不能省略；完整映射、规则轨迹等明细则应显式选择。仅把已经生成的 JSON 字段隐藏不能实现这一目标，也不能以减少证据为由放宽恢复规则。

## What Changes

- **BREAKING**：在现有恢复请求中增加类型化证据选择，普通恢复默认 Essential，完整审计显式 All；区分未请求、完整空结果、部分交付和未执行。核心身份、语义平面、可定位缺口、coverage/execution 和既有预算报告契约始终保留。
- 将内部恢复计划与对外证据产品分开。未选明细不构造；局部证据按当前方法 BCI 选择并保留必要 origin 闭包。可信正文先提交，后续证据中止不得删除已提交正文，也不得把整体请求报告为完成。
- 为正文提供绑定物理方法、环境、规则/输出配置及确切文本的产物身份。追问证据使用同一恢复管线，在新预算内复用或重建必要事实并核对产物；不引入持久 AST 注册表。
- 贯通普通目标选择、类视图、class-source、直接恢复及同类 callee 的 prepared 交接，同一操作使用已经持有的可信事实；未选择方法不提前解码，跨请求复用仍受显式有界 store 管理。
- **BREAKING**：将结构查询的 provider 与 consumer 改为可停止的逐项推进，升级 cursor 为绑定查询身份的结构位置；页满停止后续工作，不重造已经发布的匹配前缀。未访问后缀保持未知，正常页满与预算停止分别表达。
- 增加可证伪的工作量、生命周期、语义一致性和性能验收；足额 Essential/All 必须有相同正文与恢复决定，减少输出不能代替算法提速的证据。

## Capabilities

### New Capabilities

- `demand-driven-results`：规定按问题选择计算与证据、必要结果、产物绑定、事实复用生命周期，以及分开测量计算和交付的验收契约。

### Modified Capabilities

- `analysis-contracts`：将证据选择/交付状态与既有语义、覆盖、执行平面分开，保留解释答案所需的核心事实。
- `query-api`：规定 provider/consumer 的增量停止、细粒度续扫、游标验证和逐页实际覆盖。
- `source-maps`：区分内部 origin 与公开映射物化，明确局部证据、确切产物绑定及映射停止语义。
- `java8-recovery`：允许按需交付详细证据，同时保持全部恢复前提、语义和完整证据模式的原验收要求。

## Impact

涉及 `jarde-reader`、`jarde-query`、`jarde-jvm`、`jarde-java` 和根库 facade 的现有入口与报告。CLI、class-source、bulk、测试和示例同步传递证据选择；完整审计调用显式选择 All。不新增协议实现、通用 Session、全局索引、查询语言、另一套 IR 或无界保留机制，不维护长期平行兼容路径。

实现所有权分开：[add-parallel-bulk-recovery](../add-parallel-bulk-recovery/design.md) 继续负责 worker、背压、总账与活动容器任务交接；[optimize-demand-workloads](../optimize-demand-workloads/design.md) 负责工作负载、归因和优化准入；本 change 唯一负责普通 prepared 交接、可选证据产品及增量结构查询。已存在的共享 CP/Header 与 analysis report/MethodIr 分离直接复用，不重复立项。

本次只交付规划，实施为 **0/32**。数组参数槽宽、append(int) 消费 char 的 T5 反例和其它既有正确性边界独立登记，不混入本 change 或标成已修复。详细阶段、源码依据、风险与 D01–D12 验收见 [design](design.md)，实施清单见 [tasks](tasks.md)。主 specs 待真实实现与验收完成后再同步。
