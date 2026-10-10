# One-arm loop baseline independent verifier v1

该 verifier 只读取 `baseline-root-v1` 的已封闭证据、冻结工具和输入源；它不运行 collector、CLI、JDK、JADX、Cargo 或 Git。脚本及其验收输出都独立于 baseline 目录。输出为 `independent-acceptance-luna-v1.json`，存在时拒绝覆盖。

验收对象是“观测完整、结论有边界”的 baseline：两份原始 JDK class / Runner 与四份 JADX 全类重编运行成功，全部原始三流和命令均从 raw 记录复核。Jarde 四个完整源码编译腿保留真实失败和空 runtime；不把 method report 单独可读误当成全类编译成功。

五个物理方法按 javap 身份、顺序、flags 和指令 BCI 独立核对。`prefixWhile`、`loopAndTail`、`takenArm` 的 explanation-only fallback 按原文和 BCI 注释逐项核验。`noPrefix` 是 structured Java body，但其 source map 缺少物理 `goto` BCI 14；verifier 明确要求并报告这个已观察差距，而不把不完整映射伪装成全覆盖。default/all 的完整类文本、各方法文本和 source map 必须一致。

因此，verifier 成功只表示 baseline 证据及其真实结果被独立复核，不表示 Jarde 全类生成可编译或此功能已通过。
