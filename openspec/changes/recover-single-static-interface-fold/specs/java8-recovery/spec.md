## ADDED Requirements

### Requirement: 接口方法引用的 owner 参与折叠成员名锚定

系统 SHALL 在 token 锚定匹配器中接受 `InterfaceMethodRef` 条目的 owner 类名作为折叠成员名的匹配源（健全性校验不变）。既有 Fieldref/Methodref owner 命中、覆盖段不含 CP 索引的形状 SHALL 与本变更前逐字一致。

#### Scenario: 接口调用形折叠

- **WHEN** 家族成员经接口方法引用被外围调用（WCallI 形）三方 Java 8 重编
- **THEN** 家族折叠呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有锚定不变

- **WHEN** 输入为既有 owner 命中形或覆盖段不含 CP 索引
- **THEN** 输出与本变更前逐字一致
