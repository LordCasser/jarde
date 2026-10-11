# CF11产品验收与clean审计证据

目标head参数：`b155877ee9a3b1894b053e70ffbf1cbf0a9512fa`。此清单只陈列本次执行收集的原始证据和其SHA；CI、full workspace、lint与clean audit结果以各自execution/receipt为准，本README不预写通过状态。

冻结CLI：`/private/tmp/jarde-cf11-side-exits-candidate-cli-root-v10`，SHA256 `09f4eac277ee9ddfd60a627f7b132877841f4859bb2c655fc21cb8d25ac197db`，mode `0555`。CLI构建receipt和4707项source pins随verification材料保留；clean audit逐项核对live文件及expected Git tree路径，并要求worktree干净。

归档脚本按字节复制输入目录，不删除来源。原始证据与归档副本的大小和SHA记录在 `archive-inventory.json`。本目录不包含目标代码执行或额外工具链运行。

Root独立验收已实际执行：acceptance/jarde-cf11-own-product-ci-acceptance-root-v1.json 为 accepted，精确产品 b155877ee9a3b1894b053e70ffbf1cbf0a9512fa / CI 38144212651，4 jobs/52 steps，双seed各360目标/3455-0-97。全部named outcomes与fresh workspace相同；两个GitHub脱敏public labels只绑定唯一同status本地结果。完整4707 inputs与产品Git/live/冻结CLI一致，38项明确列名增量测试及既有控制已核。原始调用exit/stdout/stderr见invocations。clean-audit是产品提交后实际干净checkpoint，不冒称后续文档HEAD已经审计；关闭文档推送后将独立执行outside-repo final audit。
