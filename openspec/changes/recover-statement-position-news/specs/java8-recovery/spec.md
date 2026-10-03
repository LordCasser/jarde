## ADDED Requirements

### Requirement: 无序敏感实参的语句位构造按表达式语句呈现

系统 SHALL 在语句位构造（结果不被消费）的 new@1 构造证明通过且全部实参为常量、直读或已证消费链（无实参位调用副作用）时，按 `new X(args);` 表达式语句呈现。实参含真实调用的 CST 形状与消费位构造 SHALL 与本变更前逐字一致。

#### Scenario: 语句位构造恢复

- **WHEN** `main` 含 `new B5(); new B5(7);`（ctor 副作用可见）三方 Java 8 重编
- **THEN** 构造语句完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: CST 保护不变

- **WHEN** 输入为实参含真实调用的语句位 new（既有冻结反例）
- **THEN** 输出与本变更前逐字一致（保持拒绝）
