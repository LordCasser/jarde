## Context

见 proposal.md - Why。API 的布尔参数决定同次 recovery 是否收集泛型返回与构造器候选；初始化器、bridge 和 enum sidecar 不依赖该选项。

## Goals / Non-Goals

**Goals:**
- 让受影响测试用显式参数表达是否需要泛型候选。
- 恢复相关测试目标的编译能力。

**Non-Goals:**
- 修改生产 API、候选收集实现或任何枚举 switch 规划/roadmap。
- 给没有泛型候选断言的 initializer、bridge、enum 测试额外启用候选证明。

## Decisions

- class initializer 测试及 P3 中只验证 bridge/enum sidecar 的共享 helper 传 `false`。这些用例不检查泛型候选，传 `true` 会启动不相关的 proof 工作。
- 若后续测试显式验证泛型返回或构造器 sidecar，应在其调用处传 `true`；本次搜索没有此类断言。
- 这是测试构建修复，没有行为需求变更，因此 change 声明 `skip_specs: true`，不虚构产品 capability spec。

## Risks / Trade-offs

- [Risk] 新测试可能需要泛型 sidecar → 修改该测试的调用参数为 `true`，并明确断言候选。
