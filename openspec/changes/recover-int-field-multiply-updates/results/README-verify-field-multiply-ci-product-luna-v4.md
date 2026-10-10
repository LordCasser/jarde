# 乘法 CI 独立验收脚本 v4

v4 对应 `validation-build-root-v4/execution.json`，固定该 runner 的 SHA-256，并核对执行记录中的 runner 路径、哈希及 build schema。测试 stdout 独立计数、非测试命令空 summary 精确校验、CI 双 seed 与旧整数测试检查均沿用 v3。

结果路径为 `ci-product-v1/acceptance-field-multiply-v4.json`。v3 的预检失败记录和 v1 至 v3 脚本均保持不变。v4 仅准备，未执行；root 应在资源守卫满足后运行新的 build runner，再用真实 build、metadata、CLI 与乘法 CI 证据执行 verifier。
