## Context

见 [proposal](proposal.md)。当前 `crates/jarde-cli` 只有一个 `Request`（`input_path` + 18 个必填 limit + `Operation` 枚举），6 个 operation 全部是库形状（`inspect_header`、`inspect_method_bytecode`、`query`、`analyze_method`、`recover_method`、`enumerate`），输出只有一份 JSON（成功 envelope 或错误 envelope），退出状态只有 0/1；对应 fixture 集中在 `crates/jarde-cli/tests/json_cli.rs` 与 `query_cli.rs`。`add-task-oriented-operations` 提供库侧的选择、预算、环境、类视图、引用组织与恢复呈现；`bind-prefixed-load-roots` 提供显式 `enumerate_artifact_tree` operation。本 change 只补命令行层。

## Goals / Non-Goals

**Goals:** 让“列类 → 列方法 → 看引用 → 打开代码”在命令行可直接完成；输出、诊断与退出状态对脚本与人工都可判读；实现保持在薄适配层。

**Non-Goals:** 不新增分析逻辑或第二种发现路径；不做 GUI/MCP、交互式 TUI、批处理或整 artifact 恢复、自动 classpath、配置持久化、终端装饰；不承诺性能。

## Decisions

### 1. 每条命令 1:1 调用库操作

命令的参数只做两件事：把 friendly 输入交给库的目标选择，把预算覆盖交给库的默认预算层。报告是库的，CLI 只渲染。备选方案是在 CLI 里拼 `MethodAnalysisRequest`/环境——这正是本任务要消除的成本，且会让 CLI 与库在歧义、预算与 stage 上分叉。

### 2. 文本与 JSON 同源

两种渲染读取同一份报告结构；JSON 继续使用库的 serde 形状（不新增 CLI 专有字段语义），文本渲染只做展示。这样文本模式的字段集合可以从 JSON 报告中逐个复核，A13/A16 的逐字段比较才有意义。备选方案是为文本单独定义一套结果模型——会带来第二套语义。

### 3. 诊断与正文分离，输出文件走同一预算

文本模式把代码正文写到标准输出（或 `--output` 文件），诊断、usage 摘要与状态写到标准错误；JSON 模式把 diagnostics/coverage/execution 留在结构字段中，正文单独字段。输出文件与标准输出走同一条序列化路径与同一个 `output_bytes` 检查，避免“文件里成功、stdout 里失败”这类分叉。备选方案是复用现有 `write_success` 的 stdout 专用路径——它会把诊断和正文继续耦合在一起。

### 4. 退出状态是契约的一部分

状态集合为：成功（0）；用法/输入错误；歧义需要选择；执行未完整（Partial/Cancelled/Stopped 或 budget 中断）。四者互不相同且稳定，测试按状态断言而不是按文本断言。备选方案是沿用当前的 0/1：脚本无法区分“没找到”与“扫描没跑完”，而 A14 明确要求不把中断伪装为完整。

### 5. 发现面沿用库，不新增扫描

导航命令消费 `add-artifact-navigation` 的列举与 `bind-prefixed-load-roots` 的 `enumerate_artifact_tree`；CLI 不自己遍历 ZIP、不按路径推断 container、不自动生成 roots。跨 artifact 的处理仍要求调用方声明 root/prefix，与库的显式环境契约一致。备选方案是 CLI 自行串起路径与 entry 名——这正是 `bind-prefixed-load-roots` 已经拒绝的隐式身份构造。

### 6. 既有 JSON operation 保持可用

现有 6 个 operation 与其必填 limit schema 不删除、不改语义；任务链命令是新增面。这让已归档验收（如 CLI 逐字段等同库结果）继续成立，也避免在迁移期制造两套互相矛盾的入口。

## Risks / Trade-offs

- 文本渲染与 JSON 漂移 → 两者同源并通过逐字段比较测试；文本字段必须能在 JSON 中对应。
- 退出状态扩展破坏现有脚本 → 现有成功/错误仍是 0/1 的子集语义；新增状态只在任务链命令上生效，并在 README 中列出。
- CLI 变成第二个实现 → 代码评审与测试断言 CLI 只调用库入口（读取计数在库报告中，CLI 不额外读取）。
- tree 枚举尚未落地 → 涉及 prefix root 的验收项在 `bind-prefixed-load-roots` 完成后才勾选，其余任务不阻塞。

## Migration Plan

新增命令与输出模式，保留既有 JSON operation 与 schema；README/支持矩阵补充任务链用法与退出状态表。无持久状态迁移；回滚只需移除新增命令。
