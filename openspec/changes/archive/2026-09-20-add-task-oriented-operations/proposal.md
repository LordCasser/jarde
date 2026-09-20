## Why

CLI 的唯一入口是一份 JSON 请求：调用方要填满 18 个预算字段，再自己拼出环境、物理方法身份与 stage 列表。库示例展示同一成本——`PhysicalDefinitionId`、`PhysicalMethodId` 与 loader 环境都由调用方手工构造。结果是每个宿主（今天的 CLI，之后的 GUI/MCP）都会重复实现同一套装配逻辑，而这些事实属于引擎：用户想说的是“打开这个 artifact，看这个方法”，不是“替我拼出 P2 请求”。测量的可用性成本已经写在现有入口形状里，继续逐宿主补适配只会把同一逻辑复制到更多地方。

## What Changes

- 统一目标选择：任务导向操作接受 friendly 名称或既有物理身份，在一处完成选择；歧义返回候选且不执行，不静默取第一个；不属于本次 artifact 的身份是输入错误。
- 每个操作自行选择所需 stage 并在报告中发布实际执行的 stage 集合；`Engine::analyze_method` 的显式 stage 列表保持为可用的底层控制，语义不变。
- 有界默认预算加少量覆盖项；结果发布本次实际生效的完整 `Limits` 与 usage，预算中断沿用既有停止语义。
- 环境策略限定为三种显式形状：单 `.class`、plain JAR、显式 classpath；不推断 classpath。显式 WAR 布局策略不在本阶段，且依赖 `bind-prefixed-load-roots`：它只组织 `WEB-INF/classes/` 与嵌套库的加载位置，不声称复现容器的真实加载行为。
- 类视图：declaration + fields + method list + 按需 bodies，复用一次类读取与一次成员列举、共享一个总预算，逐方法结果独立保留；不是逐方法重扫重读的循环。`open` 保持轻量，复用方向沿用 `bound-container-lookup` 在同一不可变 artifact 上的重复操作。
- 引用结果保留 jarde 既有的语义分离：常量池出现、结构/指令引用、以及在给定声明环境下解析到声明的引用是不同发现，按 owning method 组织而不压平。
- 恢复结果先给交付内容（`contains_statements`/`explanation_only`/`not_produced`，由 `RecoveryReport::content` 提供），再给 quality 与停止原因；展示层不重新分析文本。
- 构建在 `Engine`、`ArtifactSnapshot` 与既有身份类型之上；不发明 `Session`/`Workspace`/`Project`。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `analysis-contracts`：任务导向操作的输入契约（目标选择、stage 选择、有界默认预算与生效配置）与组合操作/类视图的平面契约。
- `demand-resolver`：少量显式环境策略及其 roots/loader 声明与不推断边界。
- `structural-xref`：引用结果按 owning method 组织且保留推导类别与解析状态。
- `java8-recovery`：恢复呈现以交付内容、quality、停止原因的顺序表达。

## Impact

前提为 P1 物理身份与查询报告、P2 resolver/IR 与固定 pass 表、P3 恢复报告（含 `content`）已交付；`XrefItem`/`QueryReport`/`DeclarationRefReport` 与 `RecoveryReport` 已携带所需证据。影响根 facade 入口、`jarde-jvm` 的请求校验与环境构造、查询结果组织、恢复呈现及其测试与示例；不新增 crate、依赖、持久状态或后台索引，也不改变既有入口的语义。

依赖关系：本 change 消费 `add-artifact-navigation` 返回的物理身份与歧义规则，并为 `add-task-oriented-cli` 提供全部库操作；实现顺序为 `add-artifact-navigation` → 本 change → `add-task-oriented-cli`。WAR 布局策略依赖 `bind-prefixed-load-roots`，跨请求复用依赖 `bound-container-lookup`，二者落地前不实现或验收重叠部分。

不包含：自动 classpath 推断、宿主或网络依赖解析、`Session`/`Workspace`/`Project` 抽象、GUI/MCP 宿主、整 artifact 或批量恢复、新的缓存层、并行/索引，以及任何性能或加速承诺。
