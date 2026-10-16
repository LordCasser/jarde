# 条件返回 finally 归档

产品提交 `51de58710291cd3a3587f7040eec6f972cb26229` 对应自身 CI `38863891656`。归档及后续文档提交保持该产品的编译源码闭包；不将其声称为后续文档提交的自身 CI。

实际 apply 收据确认六项任务全部完成。java8-recovery 保留全部旧字节，追加一个 requirement、八个 scenario（20/132 → 21/140），analysis-contracts 不变。显式同步且核对后，正常 archive 使用 `--skip-specs` 避免重复应用。固定 1.11 与本机 1.14 的归档后 strict 均通过全部 363 项（345 个活动 change、18 个 spec）。本目录保存实际命令及原始输出。

完整源码恢复验收限于 LooplessFinally 有界样例：两种 javac 输入和 Java 8/23 重建回放均通过。较大的 FY/FX 完整类仍存在 missing-return 重编译失败，没有重建运行成功记录。下一片 noCall 循环恢复尚未完成。
