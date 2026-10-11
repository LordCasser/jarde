# 公开 API 验证真实失败

v4 Java lib 360/0/0、reader census 1/0/0 实际通过。随后公开测试编译失败：失败诊断读取 TargetCandidates 上不存在的 usage 字段。尚未执行 updater/CLI。v5 仅修正此测试字段，绑定 v4 相同生产/fixture inputs，不重复库/census；公开测试实跑 2/3/0，正例四方法和指定物理方法 late Stop/pre-cancel 通过。

v5 两个 range 测试把目标有效 range 的整类输出与 all 比较，其他短 member 返回既有 jre_evidence_range_invalid；第三 X4 最终 Builder 拒绝误要求 Region fallbacks 非空。对应最小测试修订待实跑，不将此失败写成产品成功。late Stop 实际 AnalysisSteps usage/limit 1883，目标 asciiToLowerCase 的真实位置 Some(72)，目标无部分 text/map/news/content。

两次 finally cargo clean 实际 exit0、target absent。原始 stdout/stderr、退出码、输入闭包和启动器保留；后续 fingerprint/新 CLI 未执行。
