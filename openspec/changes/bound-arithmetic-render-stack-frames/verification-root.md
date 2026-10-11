# Root 验收记录

当前产品 bafb61a5ec58b49f0040ea3a55420a56801bbe9c 的自身 CI 38119159950 已由 root 实际独立接受：4 jobs/52 steps，双 seed 各359目标、3444 passed/0 failed/97 ignored，完整具名结果绑定 fresh95批4693输入及132条构建输入。证据见 [stack 自身 CI](../bound-arithmetic-render-stack-frames/results/own-product-ci-root-v1/README.md)。本片实现及CI条件已闭合，进度 6/6；本轮交付文档的提交推送/clean checkpoint 仍须按实际 Git 终审记录确认。e041 的真实 stack overflow/SIGABRT 原样保留，不覆盖为成功，不宣称整个 CF12/71 单元完成。

以下为此前验收阶段的历史记录。

当前 5/6的本地构建和初步报告比较已执行；产品尚未提交，本片和三个父片的自身 CI 交付任务继续开放。e041b7635 的 CI 38116415118 确实失败，不能覆盖为成功。

最小修复只移动既有 Arithmetic 分支并抽取其余 Definition 分派。共享前置与 MAX_VALUE_DEPTH=24 不变；构造 site 仍先于执行算术。独立审查指出 site_of 是线性搜索，root 先过滤 Arithmetic 再检查 site，避免给非算术额外扫描。没有新 charge/poll、公开 IR 或恢复机制。

最终源码 build.rs SHA f4660755cf2ebc699dccf8ee7f7c984a94f6695a22a62cf649d4b728c76a1ade，既有测试 SHA 881c186b4d1f16bb4bea58a8f22f72956a7f04a1648f8087740b0666dd6dd54d（精确值以执行 pins 为准）。最终 CLI SHA b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0，0555；实际最终构建五命令的 raw、132 条动态产品输入和前后相同 pins 保存于 results/focused-build-root-v2。lambda 16/0、公开条件恢复 8/0，fmt 和构建通过，peak target 791209023 bytes，未触发守卫；cargo clean 后 target 不存在。独立 build verifier 已实际通过。

初步完整报告重放 59 对，首对为同 LambdaDeep class 的新 2 MiB main-stack 报告与旧默认栈报告，其余覆盖 compound 双模式、CF12 各类双模式、合法拒绝、跨 case continuation、NullThenBuilder。除 elapsed_millis 外所有 JSON 字段相同，包括完整 source/text/maps/身份/quality/usage。独立输入 pin、旧 raw、默认栈基线与逐对 diff 验证已实际通过，严格匹配58个父片旧命令及1个基线，无重复/遗漏；生成的 compound class 以既有独立 verifier 固定2848字节/SHA及old/new class_bytes绑定。59份diff保留，布尔值不能通过时间例外；仍不能据此宣称全部产品已验收。此前的原/JADX/Jarde 运行边界与 CF12 不完整结论保留。

fresh 完整 workspace 与独立 validator 已实际通过：95批/359 targets、3444/0/97、4693 条输入 pins 前后恒同，无 carried execution。OpenSpec strict348/0已通过。CI 同范围 fmt/Clippy（workspace/all-targets/all-features/locked，29既有allow及-D warnings）均实际exit0，源码与fresh workspace恒同；已cargo clean。下一步提交推送精确产品并验收其自身所有 CI job/step/两 seed。使用 5 GiB free 与 target 1 GiB 一秒守卫，完成后 clean。
