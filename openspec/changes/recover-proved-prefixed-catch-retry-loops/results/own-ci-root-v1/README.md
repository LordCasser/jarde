# 本产品 CI：任务合同通过，日志导出缺口保留

产品 `8ea98e2e11d64f02092ac200d2b596511dcf5e40` 的[自身 CI 39278336384](https://github.com/LordCasser/jarde/actions/runs/39278336384)已完成：4 jobs、52 个官方 API 步骤均为 completed/success。两个固定 seed 各396 targets、3856 passed/0 failed/98既有ignored，3954条具名结果与156个精确门；两轮完整目标清单、逐目标摘要和Counter相等。212个采集命令的424条stdout/stderr原流、40份源码及166份fixture的Git-show副本已重新验算并绑定产品。CLI为64754240B、0555、SHA256 `ed74e005c1bc97bcdf4a15f6a27da9e9d8a1ecdf7fe576836820ed2ce323222d`。

root审查并实际复跑Luna contract auditor v3与独立审计v2，两个进程均exit 0。按两项task 3.3的CI、具名结果、旧门及Git来源合同接受；本机Rust/JDK/CLI的数字退出码来自各自真实执行回执，GitHub API只报告status/conclusion，不伪造逐步骤数字退出码。原先更强的“每个API步骤必须有独立ZIP文件”验证器仍为FAILED，未改写成通过。

官方run与attempt日志ZIP逐字相同，794203B、SHA256 `a8bfc7520db8b337c67219b8b8d8f4979ca23d26dd4b4d407cb4d73029b6b25e`，44成员中有38个按job/步骤号/名称精确对应的独立步骤文件、4个整job aggregate及2个system文件。缺14个独立成员：stable的2/3/4/5/7；fuzz的2/3/4/6/7/9/10/20/21。四份按job ID采集的CLI原流中，fuzz流不完整；原ZIP aggregate提供更多正文。fuzz post-checkout与Complete job没有可独立识别的正文，成功状态仅引用官方API。没有从aggregate生成假独立成员，也未重写原ZIP。

CI两seed原始Counter相等；跨native比较须区分表示。native审计指纹为`6bc624d48ecbba140da74106a7123758b3e387c4baa2a3ea2ff337ddf8cba966`，CI helper指纹为`59c07dafe5152f650f3312331bea0f5338aa326cd731d27f7a27d7e66270c885`，不是字面相等。差异是Rust的` - should panic`展示后缀与GitHub遮蔽的`a_result_wei***`。后者绑定产品源码`tests/bulk_recovery_backpressure.rs:613`，同target候选唯一，两个seed均原样遮蔽且status/count不变；它不属于156个精确门。仅统一展示后缀并映射这一唯一源码身份后，3954条tuple精确相等，canonical SHA256为`e346abd2000f6d4a570c75f8faa890e19f825b7069059f71d346436432c36820`。原始指纹、名称及差异均保留。

[原始证据包](evidence.tar.gz)包含874个payload、30076118原始B；归档8062619B、SHA256 `cd6190ccd2d8d4db922c7b30054ab0f2ec2b6b2fc3c448986039827857c218c1`，全部成员已逐bytes/hash/mode重开。[清单](inventory.json)保留原路径和成员对应，[采集回执](capture.json)保留原归档位置。包内保留HTTP502、延迟状态、标准v1至v3拒绝、严格ZIP拒绝、未用于真实采集的备用工具各版本、root合同决定、两份实际复跑回执及独立审计。root决定位于`jarde-last-four-ci-root-decision-v1-20261020/decision.json`；私有v3合同审计与独立v2复跑分别位于`jarde-last-four-ci-contract-review-root-rerun-v3-20261020/`和`jarde-last-four-ci-independent-rerun-root-v1-20261020/`。

OpenSpec的`validate --all --strict`与上述CI日志ZIP完整门是两种检查。语法产品验收不声称52份独立日志齐全；CF09/71/612仍为partial，Loop6仍为原始Smali/Dex输入边界。
