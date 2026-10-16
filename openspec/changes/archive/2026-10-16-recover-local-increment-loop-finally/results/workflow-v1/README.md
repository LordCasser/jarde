# 正常同步与归档

apply实际5/5 all_done后正常同步：新增java8-recovery一条requirement/八场景，旧21/140的全文字节为新文档前缀；当前22/148，SHA256 00300b7fb22623b58b3a194420995ad28cd1d0d01c7f111e44be99fe90b1f57e。analysis-contracts原文SHA256 668af66f52a79c199abc188834995c5d197453ac9d5659eadd6a1e688c211684逐字未动。

实际archive recover-local-increment-loop-finally --yes未使用skip-validation/skip-specs，报告Specs already in sync，移动到2026-10-16-recover-local-increment-loop-finally。归档后OpenSpec1.11与1.14 strict均363 passed/0 failed；全部实际argv/原始流/退出收据保留。本分支没有加入下一片active计划，不能把其它工作树item计数套到本次结果。
