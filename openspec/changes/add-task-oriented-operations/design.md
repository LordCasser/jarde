## Context

见 [proposal](proposal.md)。现有库面已经齐备但过于底层：`Engine::analyze_method` 与 `Engine::recover_method` 要求 `MethodAnalysisRequest`（environment + `PhysicalMethodId` + `Vec<AnalysisStage>`），`Engine::resolve_symbol`/`declaration_references` 要求 `ResolutionEnvironment` 与 `LoadDomain`/`LoadRoot` 图，`Engine::query` 要求完整 `QueryRequest` 与 `PhysicalView`；CLI 的 `RequestLimits` 把 18 个维度全部设为必填。`RecoveryReport` 已有 `content`（三值闭合），`XrefItem` 已有 relation/derivation/certainty/resolution/evidence，`MethodAnalysisReport` 已发布阶段结果与读取记录——缺的是把选择、预算、环境、组织和呈现收到库里的一层。

## Goals / Non-Goals

**Goals:** 让“给 artifact 与一个目标 → 得到结果”成为库内一次调用；让配置、stage、身份与呈现都可复核；让后续宿主只做渲染与协议适配。

**Non-Goals:** 不新增 crate、依赖、持久状态或后台服务；不发明 `Session`/`Workspace`/`Project`；不做自动 classpath 推断、依赖下载、GUI/MCP、批量整 artifact 恢复；不实现 WAR 布局策略（依赖 `bind-prefixed-load-roots`）；不承诺性能或加速。

## Decisions

### 1. 目标选择收敛到一处，复用导航规则

选择接受 friendly 名称或既有物理身份，产出唯一 `PhysicalMethodId`/`PhysicalDefinitionId`。名称路径复用 `add-artifact-navigation` 的匹配与歧义规则；身份路径只做归属校验。选择这一形状而不是每个操作各写一份解析，是因为歧义语义分叉后，库与 CLI 会在“究竟选了哪一个定义”上给出不同答案（A07）。备选方案是引入 `Session` 持有目标——被明确排除，它会把生命周期、状态与并发问题提前引入，而现阶段的缺口只是选择与装配。

### 2. stage 由操作声明，控制权留在底层入口

每个操作按固定 pass 表选择自己需要的最小 stage 集合（例如“类视图按需方法体”为 SSA/effects 及恢复所需前置），并在报告中发布实际执行的 stage 集合。`Engine::analyze_method` 的显式 stage 列表保持原样，作为底层控制；`ir_pass_prerequisite_missing`/`ir_pass_order_invalid` 等既有校验不变。备选方案是隐藏 stage 或默认跑满全表——前者剥夺底层控制，后者把预算浪费在调用方从未请求的 pass 上，并改变既有停止语义。

### 3. 有界默认预算 + 少量覆盖，并发布生效配置

默认预算沿用项目原则（每个输入处理都要有界），覆盖只暴露少量高频维度（例如 output 与 elapsed 类维度），未知维度或非法值直接是输入错误。报告发布生效的 `Limits` 与 `UsageSnapshot`，使“我到底用了什么配置”不需要从 CLI 参数反推。备选方案是保持 18 字段必填（即当前可用性成本）或提供无限默认（违反预算契约）。

### 4. 环境策略只在显式声明上构造

三个策略分别对应 standalone root、该 snapshot 的 root container、以及调用方列出的 roots 顺序；它们只组装既有 validator 能接受的声明，不读宿主、不读 Manifest `Class-Path` 生成 roots、不激活扫描到的嵌套库。WAR/Boot 布局的 roots 组织延后到 `bind-prefixed-load-roots` 落地之后，并且只允许使用显式 root prefix；任何布局策略都不得声称等价于容器加载（A07/A08）。备选方案是按 layout 检出自动补 roots——这正是 `bind-prefixed-load-roots` 明确拒绝的方向（“只有 layout 检出、没有显式 prefix”仍不能补 root）。

### 5. 类视图是一份读取 + 一条列举 + 一个预算

类视图在实现上只做三件事：一次类 Header 读取、一次成员列举、按需读取被请求的方法 Body；所有子结果共享同一个 `Budget` 与同一个复用面，逐方法保留自己的阶段结果、coverage、execution 与诊断。它不是对既有单方法入口的循环调用：循环会重复类读取、把预算切成互不相关的几份，并直接违反 A16 的按需计数。`open` 本身不做读取，重复操作在同一不可变 artifact 上复用已取得的类事实（方向与 `bound-container-lookup` 一致，跨请求复用按其容量与开关边界）。备选方案是预读全部 Body——违反按需边界；或每方法独立请求——本决策要排除的那个。

### 6. 引用组织建立在既有报告之上

组织层读取 `QueryReport`/`DeclarationRefReport` 的 `XrefItem`，按 owning method 分组并保留 relation/derivation/certainty/resolution/evidence 与物理位置；常量池候选、结构引用与解析到声明保持三类。备选方案是新增统一 reference 类型——那会先把三类语义压平，再靠字段恢复出原本的分类，且与 `structural-xref`/`demand-resolver` 的既有边界冲突。

### 7. 恢复呈现读取报告，不读文本

呈现读取 `RecoveryReport.content`、`quality`、`execution`/停止原因并按规定顺序表达。`content` 已由提交结构判定，展示层再扫文本就是第二个判定源，会重复 `expose-recovery-content` 明确拒绝的做法。备选方案是按文本是否非空或含关键字推断——已有验收要求这种推断不改变分类。

### 8. 复用与依赖

复用 `Engine`、`ArtifactSnapshot`、既有身份类型、`Budget`/`Limits`、resolver 环境 validator、固定 pass 表、查询报告与恢复报告；没有缺失的通用能力，不需要新依赖或新协议。层次上全部落在现有 crate：选择、预算、组合与组织不引入第五个 crate，也不把 CLI 类型带进核心。

## Risks / Trade-offs

- 默认预算与调用方预期不符 → 覆盖项数量刻意少且生效配置随结果发布；测试断言预算中断返回真实 Partial/Cancelled 与终止维度（A14）。
- 组合操作把按需读取边界做丢 → A13/A16 计数测试加变异（逐方法重读 Header、重复列举必须变红）。
- 环境策略演变成隐式 classpath → 专项 scenario 与 negative fixture 覆盖 layout 检出、Manifest `Class-Path` 与嵌套库的存在都不生成 roots。
- 引用分组掩盖未决候选 → 分组保留 resolution/coverage，Partial 不被补全（A14）。
- 新入口与既有入口语义分叉 → 库/CLI 逐字段比较与既有回归同时作为门禁，旧入口不改语义。

## Migration Plan

新增入口与报告，既有 `Engine::analyze_method`/`recover_method`/`query`/`resolve_symbol`/`declaration_references` 保持语义与签名；示例与 `add-task-oriented-cli` 在实现后迁移到任务导向入口。无持久数据迁移，无兼容层；回滚只需移除新增入口与其测试。
