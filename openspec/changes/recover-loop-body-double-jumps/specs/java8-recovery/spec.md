## ADDED Requirements

### Requirement: 循环体内两条循环跳转边可证明并呈现

系统 SHALL 在循环体内存在恰两条循环跳转边（break/continue 的任意标签与目标组合）时，按各边目标语义（离开循环 vs 回 header）归类证明并正确呈现 `break`/`continue` 语句。单跳转全矩阵、break 与非循环控制组合及既有循环通道 SHALL 逐字不变；三条及以上跳转或与守护边交叠的形状 SHALL 保持既有拒绝或登记。

#### Scenario: 双守卫循环恢复

- **WHEN** `for (…) { if (k == 1) break; if (k == 0) continue; … }` 三方 Java 8 重编运行
- **THEN** 两条跳转语句完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有形态不变

- **WHEN** 输入为单跳转（任意目标深度/标签）、break+return 或三跳转
- **THEN** 前两者与本变更前逐字一致；后者保持既有拒绝
