## ADDED Requirements

### Requirement: TWR 包围 catch 子句体可含调用语句

系统 SHALL 在 TWR 的包围 catch 子句体语句为"非 void 调用后紧随丢弃其结果"（既有丢弃调用判据）时接受该子句。return 形与 void 调用形既有行为 SHALL 逐字不变；结果被消费或体含分支的形状 SHALL 保持既有拒绝。

#### Scenario: 调用语句 catch 体恢复

- **WHEN** `try (r) { … } catch (E e) { log.append("E"); }` 三方 Java 8 重编运行（含注入异常路径）
- **THEN** 子句体完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为 return 形 catch 体、void 调用形，或结果被消费/体含分支
- **THEN** 前两者与本变更前逐字一致；后者保持既有拒绝
