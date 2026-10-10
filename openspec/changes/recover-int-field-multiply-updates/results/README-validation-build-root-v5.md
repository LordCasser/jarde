# 乘法验证 build runner v5

v5 在验证 `--source-base` 格式后、检查输出路径或创建目录之前，先检查磁盘可用空间是否至少 20 GiB、`target` 是否不超过 1 GiB。预检不满足时以清楚的 `SystemExit` 消息退出，不创建 validation 输出；可释放空间后直接重试同一 v5 runner。启动命令前的原有资源守卫、逐命令监控、执行记录和冻结流程保持不变。

v5 使用独立 `validation-build-root-v5` 和 schema v5，CLI、metadata 路径仍使用候选 v2。不会覆盖 v3/v4 证据。runner 仅准备，未执行。
