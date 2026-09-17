## Why

以 P0 完成为进入条件。本 change 把有界内存快照、顶层物理 entry 定位和 classfile 检查组织成不依赖 CFG/SSA/反编译的结构查询产品，并明确物理视图与运行时视图的边界。Task 1.1 的公共 query/view/identity 模型已实现；nested/MR provider、consumer 扫描和查询执行仍未实现。

## What Changes

- 增加 X0/X1 查询编译器，覆盖代码、metadata、bootstrap 和标准资源 consumer。
- 增加嵌套容器候选、MR-JAR 物理/运行视图和 Boot 布局策略；物理 entry 先完整枚举，再按显式 RuntimeProfile 选择。
- 增加结构引用证据、coverage、Unknown/Partial、分页和取消结果，保证预算不足不伪造 NoMatch 或 Complete。
- 保持 X1 不构建 CFG、SSA、Java AST，也不读取无关方法 Body。

## Capabilities

### New Capabilities

- `structural-xref`: 对代码、metadata、bootstrap 和资源 consumer 生成精确结构引用。
- `artifact-views`: 在嵌套归档、MR-JAR、Boot 布局中分离物理视图和显式运行视图。
- `query-api`: 提供无反编译依赖的查询编译、证据、覆盖范围、预算和分页契约。

### Modified Capabilities

- `analysis-contracts`：物理定义身份显式区分 standalone CLASS root 与 archive entry，并把 nested origin 表示为外层 entry 到子容器的有向链；不得为 standalone CLASS 伪造 ZIP entry，也不得只保留无法复核边的容器 ID 列表。

P0 的 `artifact-snapshots`、`classfile-inspection` 和其余 `analysis-contracts` 作为输入契约；P0 仍只负责顶层 locator，不在本 change 中扩张为递归或运行时选择。

## Impact

影响 `jarde` 的 query、artifact view、resource scanner 和 result model，以及 `jarde-cli` 的查询 JSON。继续复用 P0 评估的 noak、rawzip、flate2、blake3、serde、thiserror、clap，不预设新依赖。阶段门槛为 A01–A08、A14、A17、A18 对应的测试和文档；definition/dispatch 解析属于后续 P2。当前只有 1.1 值类型契约已实现，不能解释为结构查询产品已经可用。
