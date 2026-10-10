# Switch proof and Builder scope evidence

`copy-manifest-root-v2.json` 保存 110 个已完成输入/执行文件的原始字节、路径、长度和 SHA256；不覆盖 v1，也不收录仍运行中的完整 workspace 回归。root 实际归档调用在 `/private/tmp/jarde-conditional-scope-archive-invocation-root-v2`，脚本在 `/private/tmp/jarde-archive-conditional-scope-evidence-root-v2.py`。

Builder v1/v2 的部分 Region 输入先在 lexical ownership 处拒绝；v2 还保留重复测试 helper/缺失 `Limits` import 的编译失败。v3 完整库实测 342 passed、1 failed：真实 cycle 与 unknown-terminal 测试通过，新增 Builder 的 Endless Loop 包装没有在 body 中携带其实际 header，故未到达目标消费检查。v4/v5 只改测试诊断，定位相同失败。root 修正包装，使 body 包含实际 Straight header，并保持原块多重集合。v6 实际 fmt/fmt-check 和两个指定单测都通过：真实 legacy clone 路径拒绝 1/0/0；Builder 正例、无 active switch、错误 branch/join、跨内层 loop/switch 的消费边界 1/0/0。各 selective 摘要为 343 filtered，当前库总计 344；这不替代最新完整库验收。

scope 包装与 unknown-terminal 解码事实别名是明确标注的 metamorphic 输入，不声称它们是合法物理 switch class。legacy fixture 是已有 P2 `jsr_class(50)` 同字节冻结的完整 114B class，SHA256 `5ef72ab6afa95cd945cb53123155ca99f86f4c99c9ad7fa78c2d58e044e04fa5`、BLAKE3 `dac91477a63e8c8e43acbf4e56916bab8a7415f862314f47eb5e904ef1adc019`。本片通过实际 Reader 分析得到两条 BCI7 clone 路径及完整 Call/Return 行，再核 caller 传入不同路径时证书拒绝；没有声称该 legacy class 已通过 JVM 运行。

两次 Clippy 失败原稿保留：第一次是 report 的 unnecessary_lazy_evaluations，第二次是测试 helper 的 let_and_return；root 已作最小语法修正，当前来源的完整 Clippy 尚待后续实际验收。中间 cargo clean 实际删除 1094 files / 781.5 MiB，随后构建另有记录。冻结 Luna replay adapter v2 只是静态私稿，runner/collector/verifier 是否通过以 root 后续实际记录为准。

所有 Rust 调用复用已 pin 的 v9 守卫，5 GiB free / 1 GiB target / 一秒检查。该归档只证明列出的实际观察，不接受整 CF12、完整新 CLI 或产品 CI。
