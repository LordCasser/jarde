## ADDED Requirements

### Requirement: catch 参数槽复用的消费按行类型呈现

系统 SHALL 在 catch 绑定 store 复用前序局部槽且 handler 体内的读取值定义为该绑定时，以行类型（子句头同一拼写）呈现该消费，使传参调用完整恢复。无槽复用场景、非 handler 读取与既有 rethrow 呈现 SHALL 输出零变化；不可呈现形状 SHALL 保持既有拒绝。

#### Scenario: 槽复用传参恢复

- **WHEN** `try (T r = …) { … } catch (E e) { return tag(e); }`（参数复用资源槽）三方 Java 8 重编运行
- **THEN** handler 体完整呈现，`java -Xverify:all` 正常与注入异常路径与原 class 一致

#### Scenario: 对照与边界不变

- **WHEN** 输入为无槽复用的普通 catch、非 handler 的槽复用读取或 rethrow 形态
- **THEN** 输出与本变更前逐字一致
