## Why

CLI 现在只有一个 JSON 请求入口：18 个必填预算字段，加上库形状的环境、物理方法身份与 stage 列表。想“看看这个方法”的用户必须先把引擎的内部请求拼出来，而且没有文本输出、诊断与正文的分离、输出文件或稳定退出状态。即使 `add-task-oriented-operations` 在库侧统一了选择、预算、环境与呈现，命令行上的任务链仍然拼不起来——每个后续宿主（GUI、MCP）还会各自再实现一遍渲染与状态约定。

## What Changes

- 增加任务链命令（列类、列方法、看引用、打开代码），每条命令直接调用 `add-task-oriented-operations` 的对应库操作；CLI 不实现分析、选择或分组逻辑。
- 同一请求支持文本与 JSON 两种输出，两者由同一份库报告派生、语义一致。
- 诊断与正文分离：正文走标准输出或输出文件，诊断走独立字段或标准错误；诊断不得混入代码正文。
- 增加输出文件选项：内容与标准输出模式一致，写入前沿用 `output_bytes` 预算与既有停止语义。
- 明确退出状态：成功、用法/输入错误、需要调用方选择（名称歧义）、执行未完整（Partial/Cancelled/Stopped）各自稳定且可区分；执行未完整不得以 0 退出。
- 发现面沿用 `bind-prefixed-load-roots` 增加的显式 artifact-tree 枚举，不新增第二套目录扫描；现有 JSON operation schema 保持可用。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `analysis-contracts`：任务导向 CLI 的薄适配契约、文本/JSON 输出、诊断分离、输出文件与显式退出状态。
- `artifact-views`：CLI 导航只使用库列举与显式 artifact-tree 枚举，不引入第二套发现面。

## Impact

前提为 `add-task-oriented-operations` 的库操作与 `bind-prefixed-load-roots` 的显式 tree 枚举已交付；CLI 仍只依赖 `jarde` 的公开面，不依赖内部 crate。影响 `crates/jarde-cli` 的参数与输出层、文本/JSON golden 与集成测试，以及 README 与支持矩阵中的使用说明；使用现有 `clap`/`serde_json`，不新增 crate 或依赖。

依赖顺序：本 change 在 `add-task-oriented-operations` 之后实现；tree 发现面带来的验收项必须在 `bind-prefixed-load-roots` 落地后才能完成，重叠部分不同时实现。`bound-container-lookup` 只影响底层访问成本，不改变本 change 的契约，不构成前置。

不包含：GUI/MCP 宿主、交互式 TUI、批处理或整 artifact 恢复、自动 classpath 推断、配置文件持久化、颜色/分页等终端装饰，以及任何性能或加速承诺。
