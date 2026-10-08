## Why

handoff 第一项 `new PriorityQueue<>((a,b)->b-a)` 在主线 ddfd05f8 仍整方法拒绝。2026-10-08 双编译腿巡查证明，同一缺口覆盖 Runnable、Comparator、Callable、原始类型 SAM 和多参数构造；根因是构造形状扫描拒绝参数依赖链中的 invokedynamic，既有 lambda 证明和调用实参类型适配尚未获得呈现机会。

## What Changes

- 构造实参允许其物理 SSA 依赖链中的动态调用，由已有 lambda 呈现器验证 bootstrap、capture、SAM 与适配。
- 包含动态实参的构造表达式要求 allocation 到构造调用及唯一消费者保持同一异常处理范围；不移动或重复创建与调用的副作用。
- 双 javac 腿的全类编译/执行比较与拒绝面测试，重新验收 legacy LG 锚。
- 前置条件：现有 new@1 的闭合参数区间、唯一消费与 lambda@1 的完整证明成立。非目标：任意 bootstrap 支持、可空绑定接收者的新证明、跨块构造、弃置动态实参构造、泛型 Signature 投影改进。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 组合已有构造与函数表达式的恢复能力，保留求值时机、类型与拒绝边界。

## Impact

主要修改 `crates/jarde-java/src/init.rs`，复用 `build.rs` 的 lambda_expr/new_expr/invocation_argument 和类级 helper 投影。无新 crate、IR、外部依赖、owner 白名单或公开 API。证据在 `openspec/evidence/java-syntax-2026-10-08/lambda-constructor-arguments/`；泛型返回 Signature 拒绝另行登记，不混入实现。
