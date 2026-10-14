# Proposal

## Why

当前物理方法 driver 与 callee 先取第一个同名同 descriptor 成员；当它是 abstract/native 的无 Code 声明时直接返回 no-body，绕过 reader 对重复成员的歧义拒绝。交换重复记录顺序就改变执行结果，违背完整成员表中的唯一选择契约。

## What Changes

- direct/prepared driver 与 callee 在判断 Code 和请求方法体前确认 raw name/descriptor 唯一。
- 对所有重复组合统一使用既有 `classfile_method_ambiguous` 输入错误，不静默选择首条，也不把重复无 Code 声明报告为确定的 no-body。
- 补充 abstract/native 与有 Code 声明的双顺序、两条均无 Code、两条均有 Code，以及唯一/不同 descriptor/缺失控制。
- 明确歧义在方法体准入前拒绝：完整 Header 证据保留，不消费 body、Code 或 IR 额度。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `analysis-contracts`：补充底层物理方法 driver/callee 的重复声明与 no-body 判定顺序、错误及用量契约。

## Impact

主要涉及 `crates/jarde-jvm/src/engine.rs`、`callee.rs` 和 `tests/p3_prepared_input.rs`；保留 reader 解析所得的全部物理成员和既有重复解码门。前置是当前完整成员表、prepared locator 与显式运行环境；不新增成员 ordinal 到公开 identity，不改变 resolver 的独立候选模型，不扩展恢复能力或增加依赖。共享工具链和提交由 coding root 串行管理，规划完成不代表实施通过。
