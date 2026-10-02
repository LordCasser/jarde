## Why

[super 默认调用巡查](../../evidence/java-syntax-2026-10-02/super-default-patrol/README.md)确认：限定 super 默认调用 `A.super.name()`（Java 8 接口菱形冲突调解的核心形态——`A.super.name() + B.super.name()` 与单限定）整方法拒绝；interface-special 通道已存在（build.rs:21191 专属拒绝），是"合法限定符 + 唯一默认绑定"的证明选择失败。两事实（限定符=直接超接口、目标=default 方法）在同 jar 快照内均可物理读取（调用类 header interfaces + 接口成员标志——snapshot-hierarchy-widening 已建快照 header 读取先例）。

## What Changes

- interface-special 证明接入快照内跨类事实：限定符接口 ∈ 调用类 header 的直接 interfaces ∧ 目标方法在该接口成员表为非 abstract 实例方法时，按 `X.super.m(args)` 呈现（参数/结果走既有 invocation 通道）；菱形双限定同判据逐个证明。
- F1$Diamond（`AB`）/F1$Reabstract$Impl（`I:A`）恢复且行为一致；普通 super/this 调用、普通接口调用、抽象限定（目标无体）保持现拒绝或既有行为；接口不在快照内保持拒绝并登记。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：限定 super 默认方法调用可呈现，菱形冲突调解类完整恢复。

## Impact

`crates/jarde-java`（build.rs interface-special 证明选择）与快照成员读取复用及测试；判据为快照 header/成员事实，无新机制。既有 super/this/接口调用通道零回退。
