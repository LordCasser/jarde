## 1. 命令骨架与输出

- [ ] 1.1 为任务链命令（列类、列方法、看引用、打开代码）增加参数层，参数只做 friendly 输入与预算覆盖的传递，全部调用 `add-task-oriented-operations` 的库入口；验证相同输入下 CLI 的 JSON 报告与库报告逐字段一致（仅 `elapsed_millis` 可不同），CLI 不额外读取 artifact（A13、A16）。
- [ ] 1.2 实现文本与 JSON 两种输出同源渲染：文本模式的字段可逐项对应 JSON 报告字段；JSON 保持库 serde 形状，不引入 CLI 专有语义（A13）。
- [ ] 1.3 实现诊断与正文分离：文本模式把正文写标准输出、诊断与 usage 写标准错误；JSON 模式保留独立 diagnostics/coverage/execution 字段；用带诊断的恢复 fixture 验证诊断不进入代码正文（A13）。

## 2. 输出文件与退出状态

- [ ] 2.1 实现输出文件选项，与标准输出走同一序列化路径与 `output_bytes` 检查；验证文件内容与标准输出一致，预算不足或取消时不写出部分成功文档（A14）。
- [ ] 2.2 实现显式退出状态：成功 0、用法/输入错误、歧义需要选择、执行未完整各自不同；用四条 fixture 路径按状态断言，确认执行未完整即使有可靠前缀也不以 0 退出（A14）。

## 3. 发现面与任务链闭环

- [ ] 3.1 让导航命令只使用库列举与显式 artifact-tree 枚举：验证普通枚举不递归、tree 的 Partial/Cancelled 与坏 child 与库一致、CLI 不自行解包或生成 roots；涉及 prefix root 的验收项在 `bind-prefixed-load-roots` 落地后完成（A07、A08、A14）。
- [ ] 3.2 用受控 fixture 完成“列类 → 列方法 → 引用 → 恢复”的任务链闭环：后续命令接受前一命令返回的物理身份而不要求手工重拼；验证读取范围与库操作一致、未请求的方法不读取 Body（A16、A17）。

## 4. 文档与门禁

- [ ] 4.1 更新 README 与支持矩阵中的 CLI 用法（文本/JSON、诊断分离、输出文件、退出状态表）和任务链示例；明确既有 JSON operation schema 继续可用、未包含批处理/GUI/自动 classpath（A13、A14）。
- [ ] 4.2 运行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked`（两个固定 seed）、`cargo test --test p3_execution_comparison --locked -- --ignored` 与 `openspec validate --all --strict --no-interactive`，记录固定 SHA 与结果后同步/归档。
