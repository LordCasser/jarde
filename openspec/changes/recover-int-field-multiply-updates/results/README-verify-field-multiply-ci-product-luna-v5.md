# 乘法 CI 独立验收脚本 v5

v5 对应 `validation-build-root-v5/execution.json` 和 runner v5 的固定 SHA-256；验收输出写入 `ci-product-v1/acceptance-field-multiply-v5.json`。其余 CI、build stdout 测试计数、空 summary、source pin 与守卫核验沿用 v4。

v3/v4 的失败记录和 verifier 保持不变。此版本只准备，尚未执行；root 应在资源充足时运行 runner v5，再使用实际 build、metadata、CLI 与乘法 CI 证据执行 verifier。
