## Why

`recover_for_class_source` 的测试调用仍有两参数旧形式，导致测试目标无法编译。显式选择泛型 sidecar 收集与否可恢复测试构建，并让调用意图符合当前 API。

## What Changes

- 修正 class initializer 与 P3 测试中的调用参数，按测试是否需要泛型返回/构造器候选选择 `true` 或 `false`。
- 不改变生产代码、恢复行为或已记录的枚举变更。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

无。本项只修复测试调用与构建，不改变系统行为。

## Impact

影响 `crates/jarde-java/tests/class_initializer_candidates.rs`、`crates/jarde-java/tests/p3_patterns.rs` 和本 change 的 OpenSpec 记录；不涉及生产代码或依赖。
