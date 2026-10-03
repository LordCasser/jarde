## ADDED Requirements

### Requirement: 外层 continue 与内层循环退出共享 latch 时可证明

系统 SHALL 在满足三条 CFG 结构事实时接受循环 latch 共享：内层循环块集完全包含于外层循环体、内层循环退出边目标等于外层 latch 块、外层 continue 边目标等于同一 latch 块；归属按边语义分派（continue 与内层退出均回外层 header）。单层 continue、无跳转嵌套、同一循环体内双跳转（既有片）与全部既有循环形态 SHALL 逐字不变；内层退出目标非外层 latch 或三层以上共享 SHALL 保持既有拒绝或登记。

#### Scenario: 双层过滤循环恢复

- **WHEN** `for (i…) { if (cond) continue; for (j…) { s += j; } }`（纯 int，无标签与带标签两形）三方 Java 8 重编
- **THEN** 双层循环与 `continue` 完整呈现，`java -Xverify:all` 逐路径与原 class 一致（`13`/`13`）

#### Scenario: 真实业务形体恢复

- **WHEN** Map 缓存双层 for-each + 前缀过滤 `continue` + 内层 `break`（`Svc.lookup` 形）三方重编
- **THEN** 方法体完整呈现（泛型投影按既有通道如实处理），运行逐路径与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为单层 continue、无跳转嵌套循环、同体双跳转（既有片）或内层退出目标非外层 latch
- **THEN** 前三者与本变更前逐字一致；后者保持既有拒绝
