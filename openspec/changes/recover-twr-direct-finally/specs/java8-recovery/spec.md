## ADDED Requirements

### Requirement: TWR 直接携带的 finally 子句可呈现

系统 SHALL 在 try-with-resources 降低表之后紧跟自洽的 finally 降低表（两副本中段逐指令同形、区间连续、不与资源 suppression 行交叠、正常副本入口为 TWR 正常出口的 goto 目标）时，将该子句按 `try (…) { … } finally { … }` 呈现。纯 TWR、纯 finally 与体内嵌套 finally（inner 形）SHALL 逐字不变；副本断链、交叠或三子句共存 SHALL 保持整方法拒绝。

#### Scenario: direct finally 恢复

- **WHEN** `try (r) { touch(r); } finally { log("f"); }` 三方 Java 8 重编运行
- **THEN** TWR 头与 finally 体完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有形态与负例不变

- **WHEN** 输入为纯 TWR、纯 finally、体内嵌套 finally，或副本断链/交叠/三子句
- **THEN** 前三者与本变更前逐字一致；后者保持既有拒绝
