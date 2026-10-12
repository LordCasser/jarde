# Structured List 产品自身 CI 独立验收

产品 `ca45c0ed2193fa8709eeba1196ef325f1804f63e`，运行 [38225064059](https://github.com/LordCasser/jarde/actions/runs/38225064059)。root 实际采集 3 条 gh 命令，4 jobs / 52 steps 全部成功。双 seed 各 364 targets，3502 passed / 0 failed / 98 ignored；逐目标、具名结果及顺序匹配 fresh 本地同 seed，4822 动态源码输入匹配 focus/workspace/final/live/product Git，最终 CLI SHA `d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f`。

原 capture-root-v1 属于旧 b155 产品，未复用。本次 root-v2 为 fresh 实采。v8 实跑因 tuple/list 容器比较错误拒绝，v9 实跑因 CI ignored 原因文字与本地规范化状态比较错误拒绝；两次 stdout/stderr/argv/rc 完整保存。v10 只修容器比较并规范化精确 ignored 状态，原始 reason 另存；实际 exit 0，未放宽测试数量、名称、顺序、状态和来源要求。无产品修改或测试重跑拼接。

`summary.json` 为原 root 验收结果加归档信息；原结果、完整 CI 原始日志与工具、两次失败、mismatch 定位保存于压缩包。CI 验收不等于最终文档 HEAD 干净 main 审计，该任务仍独立完成。
