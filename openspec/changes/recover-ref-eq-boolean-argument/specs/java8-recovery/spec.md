## ADDED Requirements

### Requirement: 相等比较结果直接作 boolean 调用实参可呈现

系统 SHALL 在调用实参的值由 `if_acmpXX`/`if_icmpXX` 比较双臂（iconst_1/iconst_0 汇合）产生且形参为 boolean 时，以 `a == b` / `a != b` 比较表达式呈现该实参。既有布尔位（赋值/return/条件）证明与非比较来源的 0/1 int SHALL 零变化或保持拒绝。

#### Scenario: 引用相等实参恢复

- **WHEN** `b.append(t == t.intern())` 形态三方 Java 8 重编运行
- **THEN** 语句完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 边界不变

- **WHEN** 输入为既有布尔位形态或非比较来源的 0/1
- **THEN** 输出与本变更前逐字一致
