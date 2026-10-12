# 双 seed 工作区运行：汇总失败原始记录

Root 实际运行私有 v3 runner，session 90103 退出 1。两组 seed 各 96 批测试命令均退出 0，fmt/metadata/clean 也退出 0；195 命令没有资源 guard stop，最后 cargo clean 成功、target 不存在，4822 个源码输入前后相同。

原始 runner 收据仍是 **failed**，原因是 required-named-test 汇总把 `diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty` 这个刻意 ignored 的诊断测试当成必须 ok。聚合 totals/summary 字段尚未写入，不能假装原收据 passed。所有原始 stdout/stderr、metadata、guard/resource 记录在 captures 中原样保留。

本目录只记录终结命令和汇总缺陷，尚不代表全部 metadata target/具名结果的独立接受。后续只读验收必须重建本次完整 raw suites，逐 seed 要求全部 364 target 各一次、准确的具名状态，保留原始 SHA/error 并明确区分运行器汇总失败与测试事实。不借旧运行、不混合或重写收据。本地测试也不能代表产品自身 CI。
