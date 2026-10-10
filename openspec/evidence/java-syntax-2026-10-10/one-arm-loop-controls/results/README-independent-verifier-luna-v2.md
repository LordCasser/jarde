# One-arm loop baseline independent verifier v2

本版尚未执行。v1 在实际验收中因读取不存在的 `execution_policy.default_source_map_completeness_required` 字段而失败；原失败记录、v1 脚本和 v1 输出均保留。v2 根据已生成 manifest 的真实结构，逐项要求完整 `execution_policy` 对象与已记录合同一致，不通过缺省值、忽略未知项或放宽条件处理。

验收结果固定写入 `independent-acceptance-luna-v2.json`，存在时拒绝覆盖。其它校验与 v1 相同：保留 noPrefix 缺少的物理 `goto` BCI 14 来源映射，明确记录 Jarde 四个整类编译失败、三个 missing-return diagnostics 和无 runtime。验证成功只表示 baseline 证据及观察到的 gap/failure 被独立复核，不表示功能通过。
