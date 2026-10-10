# Multiply CI/product verifier v1

该 verifier 供 root 在乘法产品提交、对应 GitHub Actions 证据齐备后运行。目前仅完成准备，未执行，也不代表 CI 已接受。

脚本导入并固定校验已接受的 `verify-int-array-ci-product-luna-v2.py` 与 static CI v12 helper 的 SHA，只复用后者的 `verify_ci`，不复制其千行校验框架。验证范围是新建的 `results/ci-product-v1`；已有 acceptance 文件会导致拒绝覆盖。

命令行要求提交 `PRODUCT_SHA`、`RUN_ID`、candidate metadata SHA、CLI SHA 和 build execution SHA。脚本将它们分别绑定到 `candidate-cli-v2.json`、只读执行权限的 `/private/tmp/jarde-field-multiply-cli-v2`、`validation-build-root-v2/execution.json`，并核对 9 条实际 build 命令、18 份原始 stdout/stderr、磁盘守卫、source-before/source-after pins、source base `977f761d9f68c6cb4de02f42b060de5290a1a947` 与 `uncommitted_field_multiply_product`。10 个 candidate source、2 个 test source 及全部 canonical input 都以 `git show PRODUCT_SHA:path` 返回的实际字节计算 SHA；canonical set 从已提交测试源码里的 include 路径推导，并包含两组基线的 manifest、inventory、source、Runner、outer class 和 `$A` class。

固定 CI helper 会检查四个 job、52 个 step、双固定 seed、Temurin JDK 25、MSRV、fuzz 与 supply-chain 证据。额外检查每个 seed 下 `p3_compound_lvalue_updates` 的 7 个测试均通过，且三个新增 multiply 测试逐名为 `ok`；再调用已接受整数 verifier 的 helper，要求前一片 11 个整数测试也在两个 seed 下逐项通过。workspace 测试通过数从实际日志动态汇总，不硬编码总数。执行时必须传入乘法产品自己的真实 CI run ID；文档检查期间的 CI run 不属于本片验收。

root 冻结 build 与 CI 证据后可按下列形式运行：

```sh
python3 -B openspec/changes/recover-int-field-multiply-updates/results/verify-field-multiply-ci-product-luna-v1.py \
  PRODUCT_SHA RUN_ID \
  --metadata-sha256 METADATA_SHA256 \
  --cli-sha256 CLI_SHA256 \
  --build-sha256 BUILD_SHA256
```

与已接受整数 verifier 的逐行 unified delta 保存在 `verify-field-multiply-ci-product-luna-v1.diff`。未来 CLI、metadata、build 和 CI run 的值尚未猜测或填入。
