# 2.2 数组构造器辅助方法证明验收

2026-09-26，root 审读 `Builder` 到 lambda planner 的现有同类 `ClassMembers` 传递，以及精确成员选择、完整无 handler Code、逐条 BCI/类型/长度参数和预算停止的窄证明。只有同类的 `private static synthetic lambda$...` 实现方法满足 `iload_0`、一维目标数组分配、`areturn` 的完整形状才保存数组类型证明；缺成员表仍可得到普通 lambda 计划，但没有数组证明。本步没有发射数组构造引用。

root 独立运行 `CARGO_TARGET_DIR=/tmp/jarde-root-integration-target cargo test -p jarde-java --lib --locked`：190/190 通过，包含成功、缺失、错误类型、副作用、handler 和预算控制；`p3_immediate_functional_receivers` 为 2/2，`p3_lambda_adaptation` 为 7 通过、2 个 JDK 测试原有 ignored。后者另两个预算断言失败，与 2.1 时干净基线上的失败相同；排除它们后复跑 7/7 通过。`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-boxed-sam-adaptations --strict` 通过。全包测试的两项既存预算/类标志失败及 Clippy 的既存 lint 已在 [2.1 验收](verification-2.1.md)和代理记录中分离，未据此放宽本步证明。

后续真实整类接线发现这里的构造测试只用通用 `iload`（0x15），而冻结 javac helper 实际用紧凑 `iload_0`（0x1a）；原 2.2 代码因此不会在真实 fixture 授予数组证明。此缺口已随 [2.3a/2.3b 集成](verification-2.3ab.md)在相同严格 Code 门内修正并重放，不把当时人工测试的通过误记为真实链路已闭合。
