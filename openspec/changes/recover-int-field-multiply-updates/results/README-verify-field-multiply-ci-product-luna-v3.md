# 乘法 CI 独立验收脚本 v3

v3 对应 `validation-build-root-v3/execution.json` 与其 v3 schema，并将验收结果写到 `ci-product-v1/acceptance-field-multiply-v3.json`。它保留 v2 从 build stdout 独立解析测试汇总的逻辑：测试命令必须有非空记录、全部 `failed == 0`，并逐项匹配 execution JSON 中的记录及固定预期值。

root 发现通过但不运行测试的命令也会由 runner 写入空 `test_summary_check` 对象；v3 对命令 0、1、8 精确核对该对象，而不错误地要求其为 `null`。verifier 还固定核对 runner 脚本 SHA-256，并要求 execution JSON 中的 runner 路径和哈希与之完全相等。

两份脚本均只准备，未执行。v2 runner 的失败 raw、v1/v2 verifier 均不修改。CI verifier 仍需传入真实产品 SHA、run ID 和 metadata、CLI、build SHA，并仅接受乘法产品自己的 CI run。
