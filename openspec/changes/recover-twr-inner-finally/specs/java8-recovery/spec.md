## ADDED Requirements

### Requirement: TWR 正文内的显式 finally 按嵌套 try/finally 呈现

系统 SHALL 在 try-with-resources 的受保护正文内存在自洽的内层显式 finally 降低（两行 catch-all 表与中段清理均在正文区间内、不与资源 suppression 行交叠）时，将该正文形态按嵌套 `try { … } finally { … }` 呈现于 TWR 块内。纯 TWR 与纯 finally 家族 SHALL 逐字不变；内层不自洽或交叠形状 SHALL 保持整方法拒绝。

#### Scenario: TWR×finally 复合恢复

- **WHEN** `try (a) { try (b) { body } finally { mid } }` 三方 Java 8 重编运行
- **THEN** 嵌套结构完整呈现且清理序忠实，`java -Xverify:all` 逐路径与原 class 一致（`body[b]mid[a]`）

#### Scenario: 纯域与负例不变

- **WHEN** 输入为纯 TWR、纯 finally，或内层行与 suppression 交叠
- **THEN** 前者与本变更前逐字一致；后者保持既有拒绝
