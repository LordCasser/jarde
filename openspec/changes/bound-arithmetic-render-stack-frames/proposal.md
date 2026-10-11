## Why

产品 e041b7635 的 CI 38116415118 在超深 lambda helper 回归中以 stack overflow/SIGABRT 结束。root 用同一冻结 CLI、300 次直线加法和 2 MiB 主线程栈复现；回溯显示 `Builder::render_value` 在进入深度守卫之前已分配约 48 KiB 的通用分派栈帧，连续算术递归耗尽余量。

## What Changes

- 保持现有值深度界、优先级、拒绝、预算、类型与物理来源语义；把普通算术递归放在轻量入口，隔离庞大的非算术定义分派函数。
- 用既有超深 lambda reader/API 回归显式运行于 2 MiB 线程，验证同一输入返回真实报告、保留 helper rename。
- 保存实际 CI 失败、同 CLI 原始复现与回溯，完成相关回归及新产品 CI。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

无。此变更修复既有 `java8-recovery` 中 “Recovery recursion is bounded and a stop is published” 的实现缺陷，保持现有公开行为契约；设置 `skip_specs: true`，不另造要求或复制整条规范。

## Impact

范围为 `crates/jarde-java/src/build.rs` 的私有渲染函数与既有 `tests/p3_immediate_functional_receivers.rs` 回归。无新增依赖、公开 IR、pass 或线程调度；不提高栈大小，不降低深度界，不混入常量名恢复、通用迭代 AST 或其他递归消费者改造。本片是当前三项父片自身 CI 关闭的前提，不预设其通过。
