## ADDED Requirements

### Requirement: 存局部的短路布尔值可作拼接实参呈现

系统 SHALL 在短路链的结果存入布尔局部、且该局部的消费方为拼接链的布尔（或其装箱）append 实参位时，按既有布尔呈现交付该值，方法完整恢复且可重编。直接 return 与存后 return 的既有输出 SHALL 逐字不变；值流断裂或共享消费者未证的形状 SHALL 保持既有退化与诊断。

#### Scenario: 拼接消费恢复

- **WHEN** `boolean hasA = (v & 1) != 0 && (v & 2) != 0; return hasA + ":" + f(v);` 三方 Java 8 重编运行
- **THEN** 短路赋值与拼接完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 判别链前档不变

- **WHEN** 输入为直接 return 短路链或存后 return 局部
- **THEN** 输出与本变更前逐字一致
