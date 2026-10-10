# 乘法 CI 独立验收脚本 v2

v2 保留 v1 的 CI、产品 pin、冻结 CLI、build guard 和回归检查，并在 build 验收中从每条测试命令的冻结 stdout 原始字节重新解析 `test result: ok. N passed; N failed; N ignored;` 记录。解析出的非空记录必须全部 `failed == 0`，逐项等于 build JSON 的 `test_summary_check.actual`；脚本中固定了预期条目的命令还必须同时等于固定 expected。验收结果也保存这次独立解析出的计数。因此 build JSON 中的 `ok` 布尔值不再单独代表测试通过。

文件只准备，尚未执行。v1 与其证据保持不变；v2 将结果写到 `results/ci-product-v1/acceptance-field-multiply-v2.json`，目标文件已存在时拒绝覆盖。执行仍需传入乘法产品自己的真实 `PRODUCT_SHA`、`RUN_ID` 和 metadata、CLI、build 的 SHA-256。不会用文档检查期间的 CI run 代替乘法产品 CI。

语法检查可以用 `python3 -c` 编译脚本源码完成；这不执行验收或工具链。root 应在真实乘法产品 CI 和冻结 build 证据齐备后运行 verifier。
